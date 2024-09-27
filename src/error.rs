use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Application {
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("UI error: {0}")]
    Ui(#[from] color_eyre::Report),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<Box<dyn StdError>> for Application {
    fn from(error: Box<dyn StdError>) -> Self {
        Application::Unexpected(error.to_string())
    }
}

impl Application {
    pub fn user_friendly_message(&self) -> &str {
        match self {
            Application::Config(_) => "There was an issue with the application configuration",
            Application::Ui(_) => "An error occurred in the user interface",
            Application::Network(_) => "There was a problem connecting to the server",
            Application::JsonParse(_) => "There was an issue processing the server response",
            Application::Unexpected(_) => "An unexpected error occurred",
        }
    }
}

pub type AppResult<T> = Result<T, Application>;
