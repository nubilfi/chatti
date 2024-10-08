//! Renders the user interface for the chat application.

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, Wrap,
    },
    Frame,
};

use super::markdown_renderer::MarkdownRenderer;
use super::state::{InputMode, State};

/// Renders the user interface for the chat application.
#[derive(Default)]
pub struct Renderer;

impl Renderer {
    /// Creates a new `UiRenderer` instance.
    #[must_use]
    pub fn new() -> Self {
        Renderer
    }

    /// Renders the entire user interface.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to the Frame to render on.
    /// * `ui_state` - A mutable reference to the current UI state.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn render(&self, f: &mut Frame, ui_state: &mut State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.area());

        let messages_area = chunks[0];
        let messages_inner_area = messages_area.inner(Margin::new(1, 1));
        let processed_messages: Vec<ListItem> = ui_state
            .messages
            .iter()
            .map(|(role, content)| {
                let (style, prefix) = match role.as_str() {
                    "user" => (Style::default().fg(Color::Blue), "You: "),
                    "assistant" => (Style::default().fg(Color::Green), "AI: "),
                    "system" => (Style::default().fg(Color::Yellow), ""),
                    _ => (Style::default(), ""),
                };

                let content = if role == "system" && ui_state.input_mode == InputMode::Waiting {
                    format!("{} {}", ui_state.spinner.next_frame(), content)
                } else {
                    content.clone()
                };

                let available_width = messages_inner_area.width as usize - prefix.len();

                let lines: Vec<_> = if role == "system" || role == "user" {
                    Self::wrap_text(&content, available_width)
                        .into_iter()
                        .enumerate()
                        .map(|(i, line)| {
                            if i == 0 {
                                Line::from(vec![
                                    Span::styled(prefix.to_string(), style),
                                    Span::raw(line),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::raw(" ".repeat(prefix.len())),
                                    Span::raw(line),
                                ])
                            }
                        })
                        .collect()
                } else {
                    let markdown_lines =
                        MarkdownRenderer::render_markdown(&content, available_width);
                    markdown_lines
                        .into_iter()
                        .enumerate()
                        .map(|(i, line)| {
                            if i == 0 {
                                Line::from(vec![
                                    Span::styled(prefix.to_string(), style),
                                    Span::raw(line.to_string()),
                                ])
                            } else {
                                Line::from(vec![
                                    Span::raw(" ".repeat(prefix.len())),
                                    Span::raw(line.to_string()),
                                ])
                            }
                        })
                        .collect()
                };

                ListItem::new(lines)
            })
            .collect();

        let messages = List::new(processed_messages)
            .block(Block::default().title("Chatti").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));

        ui_state.vertical_scroll_state = ui_state
            .vertical_scroll_state
            .content_length(ui_state.messages.len())
            .viewport_content_length(messages_area.height as usize);

        f.render_stateful_widget(messages, messages_area, &mut ui_state.list_state);

        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            messages_area.inner(Margin::new(0, 1)),
            &mut ui_state.vertical_scroll_state,
        );

        ui_state.input_width = chunks[1].width.saturating_sub(2);

        let input = Paragraph::new(ui_state.input.as_str())
            .style(match ui_state.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
                InputMode::Waiting => Style::default().fg(Color::DarkGray),
            })
            .block(Block::default().borders(Borders::ALL))
            .scroll((0, ui_state.horizontal_scroll as u16));
        f.render_widget(input, chunks[1]);

        if ui_state.input.len() as u16 > ui_state.input_width {
            let content_length = ui_state.input.len();
            let viewport_content_length = ui_state.input_width as usize;

            ui_state.horizontal_scroll_state = ui_state
                .horizontal_scroll_state
                .content_length(content_length)
                .viewport_content_length(viewport_content_length)
                .position(ui_state.horizontal_scroll);

            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                    .thumb_symbol("🬋")
                    .begin_symbol(None)
                    .end_symbol(None),
                chunks[1].inner(Margin {
                    vertical: 0,
                    horizontal: 1,
                }),
                &mut ui_state.horizontal_scroll_state,
            );
        }

        if ui_state.input_mode == InputMode::Editing {
            f.set_cursor_position(Position::new(
                chunks[1].x + ui_state.input.len() as u16 + 1,
                chunks[1].y + 1,
            ));
        }

        let (msg, style) = match ui_state.input_mode {
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

        if ui_state.show_toggle {
            Self::render_help(f);
        }
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

    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                lines.push(String::new());
                continue;
            }

            let mut wrapped_line = String::new();
            let mut current_width = 0;

            for word in line.split_whitespace() {
                let word_width = word.chars().count();

                if current_width + word_width + 1 > max_width && !wrapped_line.is_empty() {
                    lines.push(wrapped_line);
                    wrapped_line = String::new();
                    current_width = 0;
                }

                if !wrapped_line.is_empty() {
                    wrapped_line.push(' ');
                    current_width += 1;
                }

                wrapped_line.push_str(word);
                current_width += word_width;
            }

            if !wrapped_line.is_empty() {
                lines.push(wrapped_line);
            }
        }
        lines
    }
}
