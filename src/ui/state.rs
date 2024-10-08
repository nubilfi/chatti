//! Manages the state of the user interface.

use super::spinner::Spinner;
use ratatui::widgets::{ListState, ScrollbarState};

/// Represents the current state of the user interface.
#[derive(Default, Debug)]
pub struct State {
    pub current_response: String,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    pub input: String,
    pub input_mode: InputMode,
    pub input_width: u16,
    pub list_state: ListState,
    pub messages: Vec<(String, String)>,
    pub show_toggle: bool,
    pub spinner: Spinner,
    pub vertical_scroll_state: ScrollbarState,
    pub quit: bool,
}

impl State {
    /// Creates a new `UiState` instance with default values.
    ///
    /// # Returns
    ///
    /// A new `UiState` instance.
    #[must_use]
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        State {
            current_response: String::new(),
            horizontal_scroll_state: ScrollbarState::default(),
            horizontal_scroll: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
            input_width: 0,
            list_state,
            messages: Vec::new(),
            show_toggle: false,
            spinner: Spinner::new(),
            vertical_scroll_state: ScrollbarState::default(),
            quit: false,
        }
    }

    /// Scrolls the message list up by one item.
    pub fn scroll_up(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(1);
        self.list_state.select(Some(next));
        self.vertical_scroll_state = self.vertical_scroll_state.position(next);
    }

    /// Scrolls the message list down by one item.
    pub fn scroll_down(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.messages.len().saturating_sub(1));
        self.list_state.select(Some(next));
        self.vertical_scroll_state = self.vertical_scroll_state.position(next);
    }

    /// Updates the current response with new content.
    ///
    /// # Arguments
    ///
    /// * `new_content` - A string slice containing the new content to be added to the response.
    pub fn update_response(&mut self, new_content: &str) {
        if self.input_mode == InputMode::Waiting {
            self.current_response.push_str(new_content);

            if let Some((role, content)) = self.messages.last_mut() {
                if role == "assistant" {
                    content.clone_from(&self.current_response);
                } else {
                    self.messages
                        .push(("assistant".to_string(), self.current_response.clone()));
                }
            } else {
                self.messages
                    .push(("assistant".to_string(), self.current_response.clone()));
            }

            let is_at_bottom = self.list_state.selected() == Some(self.messages.len() - 1);

            if is_at_bottom {
                self.list_state.select(Some(self.messages.len() - 1));
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.messages.len() - 1);
            }
        }
    }

    /// Adds a complete response to the message list.
    ///
    /// # Arguments
    ///
    /// * `response` - A string containing the complete response to be added.
    pub fn add_response(&mut self, response: String) {
        if self.input_mode == InputMode::Waiting {
            self.messages.pop();

            if let Some((role, content)) = self.messages.last() {
                if role == "system" && content == "Generating..." {
                    self.messages.pop();
                }
            }
        }
        self.messages.push(("assistant".to_string(), response));
        self.input_mode = InputMode::Normal;
        self.current_response.clear();

        self.list_state.select(Some(self.messages.len() - 1));
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.messages.len() - 1);

        self.horizontal_scroll = 0;
        self.horizontal_scroll_state = ScrollbarState::default();
    }

    /// Prepares the UI state for a new response.
    pub fn start_new_response(&mut self) {
        self.input_mode = InputMode::Waiting;
        self.current_response.clear();
        self.messages.push(("assistant".to_string(), String::new()));
    }
}

/// Represents the different input modes of the UI.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    /// No rmal mode for navigation and command input.
    #[default]
    Normal,
    /// Editing mode for text input.
    Editing,
    /// Waiting mode while processing a request.
    Waiting,
}

/// Represents possible actions that can be taken in the UI.
#[derive(PartialEq)]
pub enum Action {
    /// Action to cancel the current request.
    CancelRequest,
}
