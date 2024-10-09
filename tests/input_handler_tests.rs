// tests/input_handler_tests.rs
use chatti::ui::input_handler::InputHandler;
use chatti::ui::state::{InputMode, State};
use crossterm::event::KeyCode;

#[test]
fn test_input_handler_normal_mode() {
    let input_handler = InputHandler::new();
    let mut ui_state = State::new();

    input_handler.handle_normal_mode(&mut ui_state, KeyCode::Char('q'));
    assert!(ui_state.quit);

    ui_state.quit = false;
    input_handler.handle_normal_mode(&mut ui_state, KeyCode::Char('e'));
    assert_eq!(ui_state.input_mode, InputMode::Editing);
}

#[test]
fn test_input_handler_editing_mode() {
    let input_handler = InputHandler::new();
    let mut ui_state = State::new();
    ui_state.input_mode = InputMode::Editing;

    let result = input_handler
        .handle_editing_mode(&mut ui_state, KeyCode::Char('a'))
        .unwrap();
    assert_eq!(result, None);
    assert_eq!(ui_state.input, "a");

    let result = input_handler
        .handle_editing_mode(&mut ui_state, KeyCode::Enter)
        .unwrap();
    assert_eq!(result, Some("a".to_string()));
    assert_eq!(ui_state.input_mode, InputMode::Waiting);
}
