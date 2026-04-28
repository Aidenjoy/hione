use hi_core::db::init_db;
use hi_core::message::{Message, MessageType, TaskStatus};
use hi_core::session::{SessionInfo, WindowInfo};
use hi_monitor::detector::{auto_return_stuck_content, run};
use hi_monitor::server::MonitorState;
use sqlx::Row;

async fn mk_state(dir: &std::path::Path) -> MonitorState {
    let pool = init_db(dir).await.unwrap();
    let session = SessionInfo {
        id: "t".into(),
        windows: vec![WindowInfo {
            index: 1,
            name: "opencode".into(),
            command: "opencode".into(),
            launch_command: "opencode".into(),
            auto_mode: false,
            resume_mode: false,
            is_main: true,
            pid: None,
            tmux_pane_id: None,
        }],
        work_dir: dir.to_path_buf(),
        hione_dir: dir.to_path_buf(),
        socket_path: SessionInfo::socket_path_for(dir),
        monitor_pid: None,
        tmux_session_name: None,
    };
    MonitorState::new(session, pool, dir.to_path_buf())
}

#[tokio::test]
async fn auto_return_with_empty_queue_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;
    auto_return_stuck_content(&state, "opencode", "stuck output").await;
    let count: i64 = sqlx::query("SELECT COUNT(*) as c FROM messages WHERE msg_type LIKE '%result%'")
        .fetch_one(&state.pool)
        .await
        .unwrap()
        .get("c");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn auto_return_writes_timeout_result_and_pops_queue() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;

    let task = Message::new_task("claude", "opencode", "compile");
    {
        let mut q = state.queues.write().await;
        q.enqueue("opencode", task.clone());
    }

    auto_return_stuck_content(&state, "opencode", "last frame").await;

    assert_eq!(state.queues.read().await.len("opencode"), 0);

    let row = sqlx::query("SELECT content, status, parent_id FROM messages WHERE parent_id = ?1")
        .bind(task.id.to_string())
        .fetch_one(&state.pool)
        .await
        .unwrap();
    let content: String = row.get("content");
    let status: String = row.get("status");
    assert!(content.contains("AUTO-PULLED"));
    assert!(content.contains("last frame"));
    assert!(status.contains("timeout"));

    let row = sqlx::query("SELECT sender, receiver, msg_type FROM messages WHERE parent_id = ?1")
        .bind(task.id.to_string())
        .fetch_one(&state.pool)
        .await
        .unwrap();
    let sender: String = row.get("sender");
    let receiver: String = row.get("receiver");
    let msg_type: String = row.get("msg_type");
    assert_eq!(sender, "opencode");
    assert_eq!(receiver, "claude");
    assert!(msg_type.contains("result"));

    let _ = (MessageType::Result, TaskStatus::Timeout);
}

#[tokio::test]
async fn run_wrapper_returns_ok_quickly_when_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;
    let handle = tokio::spawn(async move {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(50), run(state)).await;
    });
    handle.await.unwrap();
}