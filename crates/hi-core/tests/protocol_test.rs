use hi_core::protocol::{format_task_envelope, format_result_envelope, extract_result};
use uuid::Uuid;

#[test]
fn test_format_task_envelope() {
    let id = Uuid::new_v4();
    let envelope = format_task_envelope(&id, "claude", "opencode", "write tests", &[]);
    assert!(envelope.contains(&format!("Task ID: {}", id)));
    assert!(envelope.contains("From: claude"));
    assert!(envelope.contains("To: opencode"));
    assert!(envelope.contains("write tests"));
    assert!(envelope.contains("Task DONE:"));
    assert!(envelope.contains("IMPORTANT"));
}

#[test]
fn test_format_task_envelope_with_peers() {
    let id = Uuid::new_v4();
    let peers = vec!["gemini".to_string(), "qwen".to_string()];
    let envelope = format_task_envelope(&id, "claude", "opencode", "test", &peers);
    assert!(envelope.contains("COLLABORATION"));
    assert!(envelope.contains("Peers: gemini, qwen"));
    assert!(envelope.contains("hi push <peer>"));
}

#[test]
fn test_format_result_envelope() {
    let id = Uuid::new_v4();
    let envelope = format_result_envelope(&id, "opencode", "claude", "tests done");
    assert!(envelope.contains(&format!("Task DONE: {}", id)));
    assert!(envelope.contains("From: opencode"));
    assert!(envelope.contains("To: claude"));
    assert!(envelope.contains("Result:"));
}

#[test]
fn test_extract_result_found() {
    let id = Uuid::new_v4();
    let snapshot = format!(
        "Task ID: {}\nFrom: claude\nTo: opencode\n\nwrite tests\n\nIMPORTANT:\n- Reply in English.\n- End your reply with this exact line (verbatim, on its own line):\nTask DONE: {}\n\nAI's response here\nTask DONE: {}\n",
        id, id, id
    );
    let result = extract_result(&snapshot, &id);
    assert_eq!(result, Some("AI's response here".to_string()));
}

#[test]
fn test_extract_result_not_found() {
    let id = Uuid::new_v4();
    let snapshot = "No marker here";
    let result = extract_result(snapshot, &id);
    assert_eq!(result, None);
}

#[test]
fn test_extract_result_missing_task_id() {
    // header 不在快照里（已滚出），只有 done marker，视为 AI 完成
    let id = Uuid::new_v4();
    let snapshot = format!("Some AI output here\nTask DONE: {}\n", id);
    let result = extract_result(&snapshot, &id);
    assert_eq!(result, Some("Some AI output here".to_string()));
}

#[test]
fn test_extract_result_header_scrolled_off() {
    // 终端已滚动，task header 不在快照中，只剩 AI 回复 + done marker
    let id = Uuid::new_v4();
    let snapshot = format!("Long response line 1\nLong response line 2\nTask DONE: {}\n", id);
    let result = extract_result(&snapshot, &id);
    assert_eq!(result, Some("Long response line 1\nLong response line 2".to_string()));
}

#[test]
fn test_extract_result_only_one_done_with_header() {
    // header 在快照中但 AI 没有写第二个 done marker
    let id = Uuid::new_v4();
    let snapshot = format!(
        "Task ID: {}\nFrom: a\nTo: b\n\ncontent\n\nIMPORTANT:\n...\nTask DONE: {}\n",
        id, id
    );
    let result = extract_result(&snapshot, &id);
    // 只有一个 done marker 且 header 可见时：done marker 就是指令模板的，AI 没有回复
    assert_eq!(result, None);
}

#[test]
fn test_extract_result_with_whitespace() {
    let id = Uuid::new_v4();
    let snapshot = format!(
        "Task ID: {}\nFrom: a\nTo: b\n\nc\n\nIMPORTANT:\n...\nTask DONE: {}\n\n  AI response with spaces  \n  \nTask DONE: {}\n",
        id, id, id
    );
    let result = extract_result(&snapshot, &id);
    assert_eq!(result, Some("AI response with spaces".to_string()));
}

#[test]
fn test_extract_result_multiline_response() {
    let id = Uuid::new_v4();
    let snapshot = format!(
        "Task ID: {}\nFrom: a\nTo: b\n\nc\n\nIMPORTANT:\n...\nTask DONE: {}\n\nLine 1\nLine 2\nLine 3\nTask DONE: {}\n",
        id, id, id
    );
    let result = extract_result(&snapshot, &id);
    assert_eq!(result, Some("Line 1\nLine 2\nLine 3".to_string()));
}