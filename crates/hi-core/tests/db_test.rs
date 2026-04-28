use hi_core::db::{get_message_by_id, init_db, insert_message};
use hi_core::message::{Message, MessageType, TaskStatus};

#[tokio::test]
async fn init_creates_tables() {
    let dir = tempfile::tempdir().unwrap();
    let pool = init_db(dir.path()).await.unwrap();
    let _ = init_db(dir.path()).await.unwrap();
    let msg = Message::new_task("a", "b", "c");
    insert_message(&pool, &msg).await.unwrap();
}

#[tokio::test]
async fn get_missing_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let pool = init_db(dir.path()).await.unwrap();
    let r = get_message_by_id(&pool, "00000000-0000-0000-0000-000000000000")
        .await
        .unwrap();
    assert!(r.is_none());
}

#[tokio::test]
async fn insert_get_preserves_all_fields() {
    let dir = tempfile::tempdir().unwrap();
    let pool = init_db(dir.path()).await.unwrap();

    let task = Message::new_task("claude", "opencode", "implement auth");
    let task_id = task.id;
    insert_message(&pool, &task).await.unwrap();

    let result = Message::new_result("opencode", "claude", "done", task_id);
    insert_message(&pool, &result).await.unwrap();

    let got_task = get_message_by_id(&pool, &task_id.to_string())
        .await
        .unwrap()
        .expect("task missing");
    assert_eq!(got_task.sender, "claude");
    assert_eq!(got_task.receiver, "opencode");
    assert_eq!(got_task.content, "implement auth");
    assert_eq!(got_task.msg_type, MessageType::Task);
    assert_eq!(got_task.status, TaskStatus::Pending);
    assert!(got_task.parent_id.is_none());

    let got_result = get_message_by_id(&pool, &result.id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got_result.parent_id, Some(task_id));
    assert_eq!(got_result.msg_type, MessageType::Result);
    assert_eq!(got_result.status, TaskStatus::Completed);
}

#[tokio::test]
async fn insert_duplicate_id_fails() {
    let dir = tempfile::tempdir().unwrap();
    let pool = init_db(dir.path()).await.unwrap();
    let msg = Message::new_task("a", "b", "c");
    insert_message(&pool, &msg).await.unwrap();
    let err = insert_message(&pool, &msg).await.unwrap_err();
    matches!(err, hi_core::error::HiError::Db(_));
}