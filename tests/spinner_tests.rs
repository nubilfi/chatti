// tests/spinner_tests.rs
use chatti::ui::spinner::Spinner;

#[test]
fn test_spinner() {
    let mut spinner = Spinner::new();
    let first_frame = spinner.next_frame();
    let second_frame = spinner.next_frame();
    assert_ne!(first_frame, second_frame);
}
