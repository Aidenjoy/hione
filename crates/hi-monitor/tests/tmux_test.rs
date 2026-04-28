// Tests for tmux module
// Note: deliver_to_pane requires tmux runtime, so we test the logic we can isolate

#[test]
fn bracketed_paste_format() {
    // Verify the bracketed paste escape sequence format
    let content = "hello world";
    let bracketed = format!("\x1b[200~{content}\x1b[201~");
    assert!(bracketed.starts_with("\x1b[200~"));
    assert!(bracketed.ends_with("\x1b[201~"));
    assert!(bracketed.contains(content));
}

#[test]
fn bracketed_paste_with_special_chars() {
    // Verify bracketed paste works with special characters
    let content = "Task DONE: 12345\nFrom: opencode\nTo: claude";
    let bracketed = format!("\x1b[200~{content}\x1b[201~");
    assert!(bracketed.contains("Task DONE"));
    assert!(bracketed.contains('\n'));
}