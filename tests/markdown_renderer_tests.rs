// tests/markdown_renderer_tests.rs
use chatti::ui::markdown_renderer::MarkdownRenderer;

#[test]
fn test_markdown_renderer() {
    let markdown = "# Hello\n\nThis is **bold** and *italic* text.";
    let rendered = MarkdownRenderer::render_markdown(markdown, 80);

    assert!(!rendered.is_empty());

    // Check if the first span of the first line contains "Hello"
    assert_eq!(rendered[0].spans[0].content, "Hello");

    // Check if the rendered content includes bold and italic text
    let full_content: String = rendered[0]
        .spans
        .iter()
        .map(|span| span.content.clone())
        .collect();
    assert!(full_content.contains("bold"));
    assert!(full_content.contains("italic"));

    // Check for styling (this might need adjustment based on your exact implementation)
    assert!(rendered[0].spans.iter().any(|span| span
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD)));
    assert!(rendered[0].spans.iter().any(|span| span
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::ITALIC)));
}
