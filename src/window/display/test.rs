use crate::window::WindowData;

#[test]
pub fn test_angular_brackets_sanitize() {
    assert_eq!(WindowData::sanitize_text("Mozilla Firefox: <>"), "Mozilla Firefox: ");
}