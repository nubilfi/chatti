//! Renders Markdown content for the chat UI.

use pulldown_cmark::{Event as MarkdownEvent, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Renders Markdown content for the chat UI.
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    /// Renders Markdown content into a vector of styled lines.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the Markdown content to render.
    /// * `width` - The maximum width for wrapping text.
    ///
    /// # Returns
    ///
    /// A vector of `Line`s representing the rendered Markdown content.
    ///
    /// # Examples
    ///
    /// ```
    /// use chatti::ui::markdown_renderer::MarkdownRenderer;
    /// use ratatui::text::Line;
    ///
    /// let markdown = "# Hello\n\nThis is **bold** and *italic* text.";
    /// let rendered = MarkdownRenderer::render_markdown(markdown, 80);
    /// assert!(rendered.len() > 0);
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn render_markdown(content: &str, width: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(content, options);

        let mut current_line = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang = String::new();
        let mut code_block_content = String::new();
        let mut list_level = 0;
        let mut current_style = Style::default();

        for event in parser {
            match event {
                MarkdownEvent::Start(Tag::CodeBlock(kind)) => {
                    Self::flush_line(&mut lines, &mut current_line);
                    in_code_block = true;
                    if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                        code_block_lang = lang.into_string();
                    }
                }
                MarkdownEvent::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    let highlighted = Self::highlight_code(&code_block_content, &code_block_lang);
                    lines.extend(highlighted);
                    code_block_content.clear();
                    code_block_lang.clear();
                }
                MarkdownEvent::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else {
                        Self::add_text_to_line(
                            &mut lines,
                            &mut current_line,
                            &text,
                            width,
                            list_level,
                            current_style,
                        );
                    }
                }
                MarkdownEvent::SoftBreak => {
                    if !in_code_block {
                        Self::add_text_to_line(
                            &mut lines,
                            &mut current_line,
                            " ",
                            width,
                            list_level,
                            current_style,
                        );
                    }
                }
                MarkdownEvent::HardBreak => {
                    if !in_code_block {
                        Self::flush_line(&mut lines, &mut current_line);
                    }
                }
                MarkdownEvent::Start(Tag::List(..)) => {
                    Self::flush_line(&mut lines, &mut current_line);
                    list_level += 1;
                }
                MarkdownEvent::End(TagEnd::List(..)) => {
                    Self::flush_line(&mut lines, &mut current_line);
                    list_level = list_level.saturating_sub(1);
                }
                MarkdownEvent::Start(Tag::Item) => {
                    Self::flush_line(&mut lines, &mut current_line);
                    let bullet = if list_level % 2 == 1 { "• " } else { "◦ " };
                    current_line.push(Span::raw("  ".repeat(list_level - 1) + bullet));
                }
                MarkdownEvent::End(TagEnd::Item) => {
                    Self::flush_line(&mut lines, &mut current_line);
                }
                MarkdownEvent::Start(Tag::Emphasis) => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                MarkdownEvent::End(TagEnd::Emphasis) => {
                    current_style = current_style.remove_modifier(Modifier::ITALIC);
                }
                MarkdownEvent::Start(Tag::Strong) => {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                MarkdownEvent::End(TagEnd::Strong) => {
                    current_style = current_style.remove_modifier(Modifier::BOLD);
                }
                MarkdownEvent::Code(text) => {
                    let code_style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC);
                    let code_span = format!("`{text}`");
                    Self::add_text_to_line(
                        &mut lines,
                        &mut current_line,
                        &code_span,
                        width,
                        list_level,
                        code_style,
                    );
                }
                MarkdownEvent::Start(Tag::Paragraph) => {
                    if !lines.is_empty() {
                        lines.push(Line::default());
                    }
                }
                MarkdownEvent::End(TagEnd::Paragraph) => {
                    Self::flush_line(&mut lines, &mut current_line);
                    lines.push(Line::default());
                }
                _ => {}
            }
        }

        Self::flush_line(&mut lines, &mut current_line);

        while lines.last().map_or(false, |line| line.spans.is_empty()) {
            lines.pop();
        }

        lines
    }

    fn add_text_to_line(
        lines: &mut Vec<Line<'static>>,
        current_line: &mut Vec<Span<'static>>,
        text: &str,
        width: usize,
        list_level: usize,
        style: Style,
    ) {
        let indent = if list_level > 0 { 2 * list_level } else { 0 };
        let available_width = width.saturating_sub(indent);

        if current_line.is_empty() && list_level > 0 {
            current_line.push(Span::raw(" ".repeat(indent)));
        }

        let mut remaining_text = text;
        while !remaining_text.is_empty() {
            let current_line_width = current_line
                .iter()
                .map(|span| span.content.width())
                .sum::<usize>();
            let space_left = available_width.saturating_sub(current_line_width);

            if space_left == 0 {
                Self::flush_line(lines, current_line);
                if list_level > 0 {
                    current_line.push(Span::raw(" ".repeat(indent)));
                }
                continue;
            }

            let (chunk, rest) = Self::split_at_width(remaining_text, space_left);
            if !chunk.is_empty() {
                if !current_line.is_empty() && !current_line.last().unwrap().content.ends_with(' ')
                {
                    current_line.push(Span::raw(" "));
                }
                current_line.push(Span::styled(chunk.trim_end().to_string(), style));
            }

            remaining_text = rest;
            if !remaining_text.is_empty() {
                Self::flush_line(lines, current_line);
                if list_level > 0 {
                    current_line.push(Span::raw(" ".repeat(indent)));
                }
            }
        }
    }

    fn split_at_width(text: &str, width: usize) -> (&str, &str) {
        let mut total_width = 0;
        let mut split_index = text.len();

        for (idx, c) in text.char_indices() {
            let char_width = c.width_cjk().unwrap_or(1);
            if total_width + char_width > width {
                split_index = idx;
                break;
            }
            total_width += char_width;
        }

        text.split_at(split_index)
    }

    fn flush_line(lines: &mut Vec<Line<'static>>, current_line: &mut Vec<Span<'static>>) {
        if !current_line.is_empty() {
            lines.push(Line::from(std::mem::take(current_line)));
        }
    }

    fn highlight_code(code: &str, lang: &str) -> Vec<Line<'static>> {
        let theme_set = ThemeSet::load_defaults();
        let syntax_set = SyntaxSet::load_defaults_newlines();

        let syntax = syntax_set
            .find_syntax_by_extension(lang)
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]);

        LinesWithEndings::from(code)
            .map(|line| {
                let highlighted = h.highlight_line(line, &syntax_set).unwrap();
                let spans: Vec<Span> = highlighted
                    .into_iter()
                    .map(|(style, content)| {
                        let color =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(content.to_string(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}
