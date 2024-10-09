// tests/ui_state_tests.rs
use chatti::ui::state::{InputMode, State};

#[test]
fn test_ui_state() {
    let mut ui_state = State::new();

    ui_state.scroll_down();
    assert_eq!(ui_state.list_state.selected(), Some(0));

    ui_state
        .messages
        .push(("user".to_string(), "Hello".to_string()));
    ui_state
        .messages
        .push(("assistant".to_string(), "Hi".to_string()));

    ui_state.scroll_down();
    assert_eq!(ui_state.list_state.selected(), Some(1));

    ui_state.scroll_up();
    assert_eq!(ui_state.list_state.selected(), Some(0));

    // Set the input mode to Waiting before updating the response
    ui_state.input_mode = InputMode::Waiting;
    ui_state.update_response("New response");
    assert_eq!(
        ui_state.current_response, "New response",
        "current_response should be updated when input_mode is Waiting"
    );

    ui_state.add_response("Final response".to_string());
    assert_eq!(ui_state.messages.last().unwrap().1, "Final response");
    assert_eq!(ui_state.input_mode, InputMode::Normal);
}
