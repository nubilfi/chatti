use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};

use std::{
    io::{stdout, Stdout},
    time::{Duration, Instant},
};

/// Represents the Chatti Interface Structure.
///
/// This struct manages the terminal interface, including input handling,
/// message display, and UI rendering.
pub struct ChatUI {
    current_response: String,
    horizontal_scroll_state: ScrollbarState,
    horizontal_scroll: usize,
    input: String,
    input_mode: InputMode,
    input_width: u16,
    list_state: ListState,
    messages: Vec<(String, String)>,
    show_toggle: bool,
    spinner: Spinner,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    vertical_scroll_state: ScrollbarState,
}

#[derive(PartialEq, Clone, Copy)]
enum InputMode {
    Normal,
    Editing,
    Waiting,
}

struct Spinner {
    frames: Vec<char>,
    current: usize,
}

impl Spinner {
    fn new() -> Self {
        Spinner {
            frames: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current: 0,
        }
    }

    fn next(&mut self) -> char {
        let char = self.frames[self.current];
        self.current = (self.current + 1) % self.frames.len();

        char
    }
}

#[derive(PartialEq)]
pub enum Action {
    CancelRequest,
}

impl ChatUI {
    /// Creates a new `ChatUI` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new `ChatUI` instance or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatti::ui::ChatUI;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let chat_ui = ChatUI::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        let mut chat_ui = ChatUI {
            current_response: String::new(),
            horizontal_scroll_state: ScrollbarState::default(),
            horizontal_scroll: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
            input_width: 0,
            list_state: ListState::default(),
            messages: Vec::new(),
            show_toggle: false,
            spinner: Spinner::new(),
            terminal,
            vertical_scroll_state: ScrollbarState::default(),
        };
        chat_ui.list_state.select(Some(0));
        Ok(chat_ui)
    }

    pub fn update(&mut self) -> Result<Option<Action>> {
        self.draw()?;

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc && self.input_mode == InputMode::Waiting {
                    self.input_mode = InputMode::Normal;
                    return Ok(Some(Action::CancelRequest));
                }
            }
        }

        Ok(None)
    }

    /// Runs the main event loop of the UI.
    ///
    /// This method handles user input and updates the UI accordingly.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option<String>` (the user's message)
    /// or an error.
    pub fn run(&mut self) -> Result<Option<String>> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            self.draw()?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if self.input_mode == InputMode::Waiting {
                if event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Esc {
                            self.input_mode = InputMode::Normal;
                            self.messages.pop();
                        }
                    }
                }

                continue;
            }

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(None),
                        KeyCode::Char('?') => self.toggle_help(),
                        _ => self.handle_normal_mode(key.code),
                    },
                    InputMode::Editing => {
                        if let Some(message) = self.handle_editing_mode(key.code)? {
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

    fn handle_normal_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('e') => self.input_mode = InputMode::Editing,
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),
            _ => {}
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn handle_editing_mode(&mut self, key: KeyCode) -> Result<Option<String>> {
        match key {
            KeyCode::Enter => {
                let message: String = self.input.drain(..).collect();
                self.messages.push(("user".to_string(), message.clone()));
                self.input_mode = InputMode::Waiting;
                self.messages
                    .push(("system".to_string(), "Generating...".to_string()));
                self.horizontal_scroll = 0;
                self.horizontal_scroll_state = ScrollbarState::default();
                Ok(Some(message))
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                Ok(None)
            }
            KeyCode::Backspace => {
                self.input.pop();
                Ok(None)
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                Ok(None)
            }
            KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                Ok(None)
            }
            KeyCode::Right => {
                let max_scroll = self.input.len().saturating_sub(self.input_width as usize);
                self.horizontal_scroll = (self.horizontal_scroll + 1).min(max_scroll);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn scroll_up(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(1);
        self.list_state.select(Some(next));
        self.vertical_scroll_state = self.vertical_scroll_state.position(next);
    }

    fn scroll_down(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.messages.len().saturating_sub(1));
        self.list_state.select(Some(next));
        self.vertical_scroll_state = self.vertical_scroll_state.position(next);
    }

    /// Updates the UI with new content from the ongoing response.
    ///
    /// # Arguments
    ///
    /// * `new_content` - A string slice containing the new content to be added.
    pub fn update_response(&mut self, new_content: &str) {
        if self.input_mode == InputMode::Waiting {
            self.current_response.push_str(new_content);

            if let Some((role, content)) = self.messages.last_mut() {
                if role == "assistant" {
                    // *content = self.current_response.clone();
                    content.clone_from(&self.current_response);
                } else {
                    self.messages
                        .push(("assistant".to_string(), self.current_response.clone()));
                }
            } else {
                self.messages
                    .push(("assistant".to_string(), self.current_response.clone()));
            }

            // Scroll to the bottom
            self.list_state.select(Some(self.messages.len() - 1));
            self.vertical_scroll_state =
                self.vertical_scroll_state.position(self.messages.len() - 1);
        }
    }

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

        // scroll to the bottom
        self.list_state.select(Some(self.messages.len() - 1));
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.messages.len() - 1);

        // reset horizontal scroll when switching back to normal mode
        self.horizontal_scroll = 0;
        self.horizontal_scroll_state = ScrollbarState::default();
    }

    pub fn start_new_response(&mut self) {
        self.input_mode = InputMode::Waiting;
        self.current_response.clear();
        self.messages.push(("assistant".to_string(), String::new()));
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::items_after_statements)]
    // TODO: Optimize later
    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(f.area());

            let messages_area = chunks[0];
            let messages_inner_area = messages_area.inner(Margin::new(1, 1));
            let mut messages = Vec::with_capacity(self.messages.len());

            for (role, content) in &self.messages {
                let (style, prefix) = match role.as_str() {
                    "user" => (Style::default().fg(Color::Blue), "You: "),
                    "assistant" => (Style::default().fg(Color::Green), "AI: "),
                    "system" => (Style::default().fg(Color::Yellow), ""),
                    _ => (Style::default(), ""),
                };

                let content = if role == "system" && self.input_mode == InputMode::Waiting {
                    format!("{} {}", self.spinner.next(), content)
                } else {
                    content.clone()
                };

                let wrapped_content = wrap_text(
                    content.as_str(),
                    messages_inner_area.width as usize - prefix.len(),
                );
                let mut lines = Vec::with_capacity(wrapped_content.len());

                for (i, line) in wrapped_content.into_iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, style),
                            Span::raw(line),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::raw(" ".repeat(prefix.len())),
                            Span::raw(line),
                        ]));
                    }
                }

                messages.push(ListItem::new(lines));
            }

            let messages = List::new(messages)
                .block(Block::default().title("Chatti").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));

            self.vertical_scroll_state = self
                .vertical_scroll_state
                .content_length(self.messages.len())
                .viewport_content_length(messages_area.height as usize);

            f.render_stateful_widget(messages, messages_area, &mut self.list_state);

            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None),
                messages_area.inner(Margin::new(0, 1)),
                &mut self.vertical_scroll_state,
            );

            // Store the available width for the input
            self.input_width = chunks[1].width.saturating_sub(2);

            let input = Paragraph::new(self.input.as_str())
                .style(match self.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                    InputMode::Waiting => Style::default().fg(Color::DarkGray),
                })
                .block(Block::default().borders(Borders::ALL))
                .scroll((0, self.horizontal_scroll as u16));
            f.render_widget(input, chunks[1]);

            // Only update scroll state and render scrollbar if necessary
            if self.input.len() as u16 > self.input_width {
                let content_length = self.input.len();
                let viewport_content_length = self.input_width as usize;

                self.horizontal_scroll_state = self
                    .horizontal_scroll_state
                    .content_length(content_length)
                    .viewport_content_length(viewport_content_length)
                    .position(self.horizontal_scroll);

                f.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                        .thumb_symbol("🬋")
                        .begin_symbol(None)
                        .end_symbol(None),
                    chunks[1].inner(Margin {
                        vertical: 0,
                        horizontal: 1,
                    }),
                    &mut self.horizontal_scroll_state,
                );
            }

            if self.input_mode == InputMode::Editing {
                f.set_cursor_position(Position::new(
                    chunks[1].x + self.input.len() as u16 + 1,
                    chunks[1].y + 1,
                ));
            }

            let (msg, style) = match self.input_mode {
                InputMode::Normal => (
                    vec![
                        "Press ".into(),
                        "q".bold(),
                        " to exit, ".into(),
                        "e".bold(),
                        " to start editing, ".into(),
                        "?".bold(),
                        " to show help menu".into(),
                    ],
                    Style::default(),
                ),
                InputMode::Editing => (
                    vec![
                        "Press ".into(),
                        "Esc".bold(),
                        " to stop editing, ".into(),
                        "Enter".bold(),
                        " to send the message".into(),
                    ],
                    Style::default(),
                ),
                InputMode::Waiting => (
                    vec!["Press ".into(), "Esc".bold(), " to cancel request".into()],
                    Style::default(),
                ),
            };

            let text = Text::from(Line::from(msg)).patch_style(style);
            let help_message = Paragraph::new(text);

            f.render_widget(help_message, chunks[1]);

            if self.show_toggle {
                ChatUI::render_help(f);
            }
        })?;

        Ok(())
    }

    fn render_help(f: &mut Frame) {
        let area = f.area();
        let help_area = Rect::new(
            area.width / 4,
            area.height / 5,
            area.width / 2,
            area.height / 2,
        );

        f.render_widget(Clear, help_area);

        let help_text = vec![
            Line::from("Shortcut Information"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "?",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to toggle/untoggle this help menu"),
            ]),
            Line::from(vec![
                Span::styled(
                    "q",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to quit the application"),
            ]),
            Line::from(vec![
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to exit from Editing Mode"),
            ]),
            Line::from(vec![
                Span::styled(
                    "Left/Right key",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to scrolling horizontally"),
            ]),
            Line::from(vec![
                Span::styled(
                    "Up/Down key",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to scrolling vertically"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, help_area);
    }

    pub fn toggle_help(&mut self) {
        self.show_toggle = !self.show_toggle;
    }
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > max_width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            if word.len() > max_width {
                let mut remaining = word;
                while !remaining.is_empty() {
                    let (chunk, rest) =
                        remaining.split_at(std::cmp::min(remaining.len(), max_width));
                    lines.push(chunk.to_string());
                    remaining = rest;
                }
            } else {
                current_line = word.to_string();
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

impl Drop for ChatUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        stdout().execute(LeaveAlternateScreen).unwrap();
    }
}
