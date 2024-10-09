// tests/mock/mod.rs
use chatti::ui::chat::Interface;
use chatti::ui::input_handler::InputHandler;
use chatti::ui::renderer::Renderer;
use chatti::ui::state::State;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;

pub fn create_mock_chat_ui() -> Interface {
    let backend = CrosstermBackend::new(stdout());
    let terminal = ratatui::Terminal::new(backend).unwrap();
    let ui_renderer = Renderer::new();

    Interface {
        ui_state: State::new(),
        input_handler: InputHandler::new(),
        ui_renderer,
        terminal,
    }
}
