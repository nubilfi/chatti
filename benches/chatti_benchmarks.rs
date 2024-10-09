use chatti::config::Config;
use chatti::ui::{markdown_renderer::MarkdownRenderer, spinner::Spinner, state::State};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_markdown_renderer(c: &mut Criterion) {
    let markdown = "# Header\n\nThis is a paragraph with **bold** and *italic* text.\n\n- List item 1\n- List item 2\n\n```rust\nfn hello() {\n    println!(\"Hello, world!\");\n}\n```";

    c.bench_function("markdown_render", |b| {
        b.iter(|| MarkdownRenderer::render_markdown(black_box(markdown), black_box(80)))
    });
}

fn bench_spinner(c: &mut Criterion) {
    let mut spinner = Spinner::new();
    c.bench_function("spinner_next_frame", |b| {
        b.iter(|| {
            black_box(spinner.next_frame());
        })
    });
}

fn bench_ui_state_update_response(c: &mut Criterion) {
    let mut ui_state = State::new();
    ui_state.input_mode = chatti::ui::state::InputMode::Waiting;
    let response = "This is a test response that will be added to the UI state.";

    c.bench_function("ui_state_update_response", |b| {
        b.iter(|| {
            ui_state.update_response(black_box(response));
        })
    });
}

fn bench_config_load(c: &mut Criterion) {
    c.bench_function("config_load", |b| {
        b.iter(|| {
            Config::load().unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_markdown_renderer,
    bench_spinner,
    bench_ui_state_update_response,
    bench_config_load
);
criterion_main!(benches);
