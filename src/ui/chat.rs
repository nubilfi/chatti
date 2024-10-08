//! Provides the main chat user interface functionality.

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{stdout, Stdout},
    time::{Duration, Instant},
};

use super::input_handler::InputHandler;
use super::renderer::Renderer;
use super::state::{InputMode, State};

/// The main structure for the chat user interface.
pub struct Interface {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub ui_state: State,
    pub input_handler: InputHandler,
    pub ui_renderer: Renderer,
}

impl Interface {
    /// Creates a new `ChatUI` instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `ChatUI` instance or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails to set up the terminal or create the UI components.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        let ui_state = State::new();
        let input_handler = InputHandler::new();
        let ui_renderer = Renderer::new();

        Ok(Interface {
            terminal,
            ui_state,
            input_handler,
            ui_renderer,
        })
    }

    /// Runs the main chat UI loop.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<String>` with the user's message, or `None` if the user quits.
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues with event polling or drawing the UI.
    pub fn run(&mut self) -> Result<Option<String>> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            self.draw()?;

            if self.ui_state.quit {
                return Ok(None);
            }

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if self.ui_state.input_mode == InputMode::Waiting {
                if event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Esc {
                            self.ui_state.input_mode = InputMode::Normal;
                            self.ui_state.messages.pop();
                        }
                    }
                }
                continue;
            }

            if let Event::Key(key) = event::read()? {
                match self.ui_state.input_mode {
                    InputMode::Normal => self
                        .input_handler
                        .handle_normal_mode(&mut self.ui_state, key.code),
                    InputMode::Editing => {
                        if let Some(message) = self
                            .input_handler
                            .handle_editing_mode(&mut self.ui_state, key.code)?
                        {
                            return Ok(Some(message));
                        }
                    }
                    InputMode::Waiting => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    /// Checks if the user has requested to quit the application.
    ///
    /// # Returns
    ///
    /// `true` if the user has requested to quit, `false` otherwise.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.ui_state.quit
    }

    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            self.ui_renderer.render(f, &mut self.ui_state);
        })?;

        Ok(())
    }

    /// Updates the UI and checks for user input.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<Action>` if an action needs to be taken.
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues with event polling or drawing the UI.
    pub fn update(&mut self) -> Result<Option<super::state::Action>> {
        self.draw()?;

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc && self.ui_state.input_mode == InputMode::Waiting {
                    self.ui_state.input_mode = InputMode::Normal;
                    return Ok(Some(super::state::Action::CancelRequest));
                }
            }
        }

        Ok(None)
    }

    /// Updates the current response in the UI.
    ///
    /// # Arguments
    ///
    /// * `new_content` - A string slice containing the new content to be added to the response.
    pub fn update_response(&mut self, new_content: &str) {
        self.ui_state.update_response(new_content);
    }

    /// Adds a complete response to the UI.
    ///
    /// # Arguments
    ///
    /// * `response` - A string containing the complete response to be added.
    pub fn add_response(&mut self, response: String) {
        self.ui_state.add_response(response);
    }

    /// Prepares the UI for a new response.
    pub fn start_new_response(&mut self) {
        self.ui_state.start_new_response();
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
    }
}
