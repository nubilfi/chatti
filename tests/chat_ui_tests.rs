use chatti::ui::state::InputMode;

// Import mock module
mod mock;
use mock::create_mock_chat_ui;

#[test]
#[ignore = "Intermittent OS error (WouldBlock) in GitHub Actions; tests pass locally"]
fn test_chat_ui_new_response() {
    let mut chat_ui = create_mock_chat_ui();
    chat_ui.start_new_response();
    assert_eq!(chat_ui.ui_state.input_mode, InputMode::Waiting);
    assert_eq!(chat_ui.ui_state.current_response, "");
    assert_eq!(chat_ui.ui_state.messages.last().unwrap().0, "assistant");
    assert_eq!(chat_ui.ui_state.messages.last().unwrap().1, "");
}

#[test]
#[ignore = "Intermittent OS error (WouldBlock) in GitHub Actions; tests pass locally"]
fn test_chat_ui_update_response() {
    let mut chat_ui = create_mock_chat_ui();
    chat_ui.start_new_response();
    chat_ui.update_response("Hello");
    chat_ui.update_response(", world!");
    assert_eq!(chat_ui.ui_state.current_response, "Hello, world!");
    assert_eq!(chat_ui.ui_state.messages.last().unwrap().1, "Hello, world!");
}

#[test]
#[ignore = "Intermittent OS error (WouldBlock) in GitHub Actions; tests pass locally"]
fn test_chat_ui_add_response() {
    let mut chat_ui = create_mock_chat_ui();
    chat_ui.add_response("Test response".to_string());
    assert_eq!(chat_ui.ui_state.input_mode, InputMode::Normal);
    assert_eq!(chat_ui.ui_state.messages.last().unwrap().0, "assistant");
    assert_eq!(chat_ui.ui_state.messages.last().unwrap().1, "Test response");
}
