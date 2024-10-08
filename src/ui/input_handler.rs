//! Handles user input for the chat UI.

use super::state::{InputMode, State};
use color_eyre::Result;
use crossterm::event::KeyCode;

/// Handles user input for the chat UI.
#[derive(Default)]
pub struct InputHandler;

impl InputHandler {
    /// Creates a new `InputHandler` instance.
    #[must_use]
    pub fn new() -> Self {
        InputHandler
    }

    /// Handles input in normal mode.
    ///
    /// # Arguments
    ///
    /// * `ui_state` - A mutable reference to the current UI state.
    /// * `key` - The key code of the pressed key.
    pub fn handle_normal_mode(&self, ui_state: &mut State, key: KeyCode) {
        match key {
            KeyCode::Char('q') => ui_state.quit = true,
            KeyCode::Char('?') => ui_state.show_toggle = !ui_state.show_toggle,
            KeyCode::Char('e') => ui_state.input_mode = InputMode::Editing,
            KeyCode::Up => ui_state.scroll_up(),
            KeyCode::Down => ui_state.scroll_down(),
            _ => {}
        }
    }

    /// Handles input in editing mode.
    ///
    /// # Arguments
    ///
    /// * `ui_state` - A mutable reference to the current UI state.
    /// * `key` - The key code of the pressed key.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<String>` with the user's message if Enter was pressed.
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues updating the UI state.
    pub fn handle_editing_mode(
        &self,
        ui_state: &mut State,
        key: KeyCode,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Enter => {
                let message: String = ui_state.input.drain(..).collect();
                ui_state
                    .messages
                    .push(("user".to_string(), message.clone()));
                ui_state.input_mode = InputMode::Waiting;
                ui_state
                    .messages
                    .push(("system".to_string(), "Generating...".to_string()));
                ui_state.horizontal_scroll = 0;
                ui_state.horizontal_scroll_state = ratatui::widgets::ScrollbarState::default();
                Ok(Some(message))
            }
            KeyCode::Char(c) => {
                ui_state.input.push(c);
                Ok(None)
            }
            KeyCode::Backspace => {
                ui_state.input.pop();
                Ok(None)
            }
            KeyCode::Esc => {
                ui_state.input_mode = InputMode::Normal;
                Ok(None)
            }
            KeyCode::Left => {
                ui_state.horizontal_scroll = ui_state.horizontal_scroll.saturating_sub(1);
                Ok(None)
            }
            KeyCode::Right => {
                let max_scroll = ui_state
                    .input
                    .len()
                    .saturating_sub(ui_state.input_width as usize);
                ui_state.horizontal_scroll = (ui_state.horizontal_scroll + 1).min(max_scroll);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
