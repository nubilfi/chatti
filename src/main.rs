mod config;
mod error;
mod logging;
mod ui;

use config::Config;
use error::{AppResult, Application};
use futures_util::StreamExt;
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, instrument};
use ui::{Action, ChatUI};

/// The main entry point of the Chatti application.
///
/// This function sets up logging, loads the configuration,
/// initializes the UI, and manages the main application loop.
#[tokio::main]
async fn main() -> AppResult<()> {
    color_eyre::install()?;
    let _guard = logging::setup()?;

    let config = Config::load()?;
    let mut chat_ui = ChatUI::new()?;
    let client = reqwest::Client::new();

    while let Some(message) = chat_ui.run()? {
        let (tx, mut rx) = mpsc::channel(100);
        let client_clone = client.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            if let Err(err) = process_message(&client_clone, &config_clone, &message, tx).await {
                error!(?err, "error occurred in process_message");
            }
        });

        chat_ui.start_new_response();
        process_response(&mut chat_ui, &mut rx).await?;
    }

    Ok(())
}

/// Processes a user message by sending it to the API and streaming the response.
///
/// # Arguments
///
/// * `client` - The HTTP client for making API requests.
/// * `config` - The application configuration.
/// * `message` - The user's message to be processed.
/// * `tx` - A channel sender for streaming the response.
///
/// # Returns
///
/// Returns a `Result` indicating success or an `Application` error.
#[instrument]
async fn process_message(
    client: &reqwest::Client,
    config: &Config,
    message: &str,
    tx: mpsc::Sender<Result<String, Application>>,
) -> AppResult<()> {
    let response = client
        .post(&config.api_endpoint)
        .json(&json!({
            "model": config.model,
            "messages": [{"role": "user", "content": message}],
            "stream": config.stream,
            "temperature": config.temperature,
        }))
        .send()
        .await?;

    if !config.stream {
        // Handle regular (non-streaming) response
        let json: serde_json::Value = response.json().await?;
        if let Some(content) = json["message"]["content"].as_str() {
            tx.send(Ok(content.to_string()))
                .await
                .map_err(|e| Application::Unexpected(e.to_string()))?;
        }
        return Ok(());
    }

    // Handle streaming response
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);

        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&buffer[..pos]).to_string();
            buffer.drain(..=pos);

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(content) = json["message"]["content"].as_str() {
                    tx.send(Ok(content.to_string()))
                        .await
                        .map_err(|e| Application::Unexpected(e.to_string()))?;
                }

                if json["done"].as_bool().unwrap_or(false) {
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

/// Processes the streamed response and updates the UI.
///
/// # Arguments
///
/// * `chatti` - A mutable reference to the `ChatUI` instance.
/// * `rx` - A mutable reference to the receiver channel for the streamed response.
///
/// # Returns
///
/// Returns a `Result` indicating success or an `Application` error.
async fn process_response(
    chat_ui: &mut ChatUI,
    rx: &mut mpsc::Receiver<Result<String, Application>>,
) -> AppResult<()> {
    let mut full_response = String::new();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Some(Ok(content)) => {
                        full_response.push_str(&content);
                        chat_ui.update_response(&content);
                        if let Some(action) = chat_ui.update()? {
                            if action == Action::CancelRequest {
                                chat_ui.add_response("request cancelled".to_string());
                                return Ok(());
                            }
                        }
                    }
                    Some(Err(err)) => {
                        error!(?err, "error occurred while receiving response");
                        chat_ui.add_response(format!(
                            "error: {}, For more details, please check the log file at: {}",
                            err.display_message(), logging::get_log_file_path().display()
                        ));
                        break;
                    }
                    None => {
                        if !full_response.is_empty() {
                            chat_ui.add_response(full_response);
                        }
                        break;
                    }
                }
            }
            () = tokio::time::sleep(Duration::from_millis(100)) => {
                if let Some(action) = chat_ui.update()? {
                    if action == Action::CancelRequest {
                        chat_ui.add_response("request cancelled".to_string());
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}
