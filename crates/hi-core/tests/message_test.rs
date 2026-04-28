use hi_core::message::{Message, MessageType, TaskStatus};

#[test]
fn test_new_task_fields() {
    let msg = Message::new_task("claude", "opencode", "implement auth");
    assert_eq!(msg.sender, "claude");
    assert_eq!(msg.receiver, "opencode");
    assert_eq!(msg.content, "implement auth");
    assert_eq!(msg.msg_type, MessageType::Task);
    assert_eq!(msg.status, TaskStatus::Pending);
    assert!(msg.parent_id.is_none());
}

#[test]
fn test_result_links_to_parent() {
    let task = Message::new_task("claude", "opencode", "implement auth");
    let result = Message::new_result("opencode", "claude", "done", task.id);
    assert_eq!(result.parent_id, Some(task.id));
    assert_eq!(result.msg_type, MessageType::Result);
}

#[test]
fn test_message_json_roundtrip() {
    let msg = Message::new_task("a", "b", "hello");
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, msg.id);
    assert_eq!(decoded.content, "hello");
}

#[test]
fn test_new_check_defaults() {
    let msg = Message::new_check("hi", "opencode");
    assert_eq!(msg.msg_type, MessageType::Check);
    assert_eq!(msg.status, TaskStatus::Pending);
    assert_eq!(msg.sender, "hi");
    assert_eq!(msg.receiver, "opencode");
    assert!(msg.content.is_empty());
    assert!(msg.parent_id.is_none());
}

#[test]
fn test_message_type_serializes_snake_case() {
    let s = serde_json::to_string(&MessageType::CheckAck).unwrap();
    assert_eq!(s, "\"check_ack\"");
    let s = serde_json::to_string(&MessageType::SnapshotData).unwrap();
    assert_eq!(s, "\"snapshot_data\"");
}

#[test]
fn test_task_status_serializes_snake_case() {
    let s = serde_json::to_string(&TaskStatus::Completed).unwrap();
    assert_eq!(s, "\"completed\"");
    let s = serde_json::to_string(&TaskStatus::Cancelled).unwrap();
    assert_eq!(s, "\"cancelled\"");
}

#[test]
fn test_all_message_types_roundtrip() {
    for t in [
        MessageType::Task,
        MessageType::Result,
        MessageType::Cancel,
        MessageType::Check,
        MessageType::CheckAck,
        MessageType::Pull,
        MessageType::Snapshot,
        MessageType::SnapshotData,
    ] {
        let s = serde_json::to_string(&t).unwrap();
        let back: MessageType = serde_json::from_str(&s).unwrap();
        assert_eq!(back, t);
    }
}

#[test]
fn test_all_task_status_roundtrip() {
    for s in [
        TaskStatus::Pending,
        TaskStatus::Running,
        TaskStatus::Completed,
        TaskStatus::Cancelled,
        TaskStatus::Timeout,
    ] {
        let json = serde_json::to_string(&s).unwrap();
        let back: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
}
