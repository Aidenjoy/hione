//! 集成测试：启动真实 server::run，通过 UnixStream 发消息验证行为。
#![cfg(unix)]

use hi_core::{
    db::init_db,
    ipc::{recv_message, send_message},
    message::{Message, MessageType, TaskStatus},
    session::{SessionInfo, WindowInfo},
};
use hi_monitor::server::{self, MonitorState};
use std::time::Duration;
use tokio::net::UnixStream;
use uuid::Uuid;

async fn build_state(dir: &std::path::Path) -> MonitorState {
    let pool = init_db(dir).await.unwrap();
    let session = SessionInfo {
        id: "test".into(),
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

async fn spawn_server(state: MonitorState) {
    tokio::spawn(async move {
        let _ = server::run(state).await;
    });
    // 等 bind
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn check_receives_ack() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    spawn_server(state).await;

    let mut stream = UnixStream::connect(&sock).await.unwrap();
    let check = Message::new_check("cli", "opencode");
    send_message(&mut stream, &check).await.unwrap();

    let ack = tokio::time::timeout(Duration::from_secs(2), recv_message(&mut stream))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ack.msg_type, MessageType::CheckAck);
    assert_eq!(ack.parent_id, Some(check.id));
    assert_eq!(ack.sender, "monitor");
    assert_eq!(ack.status, TaskStatus::Completed);
}

#[tokio::test]
async fn task_is_enqueued_and_persisted() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    let queues = state.queues.clone();
    let pool = state.pool.clone();
    spawn_server(state).await;

    let mut stream = UnixStream::connect(&sock).await.unwrap();
    let task = Message::new_task("cli", "opencode", "write tests");
    send_message(&mut stream, &task).await.unwrap();
    drop(stream);

    // 轮询等待入队（最多 1s）
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if queues.read().await.len("opencode") == 1 {
            break;
        }
    }
    assert_eq!(queues.read().await.len("opencode"), 1);

    // 验证持久化
    let got = hi_core::db::get_message_by_id(&pool, &task.id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.content, "write tests");
}

#[tokio::test]
async fn pull_with_no_pending_task() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    spawn_server(state).await;

    let mut stream = UnixStream::connect(&sock).await.unwrap();
    let pull = Message {
        id: Uuid::new_v4(),
        sender: "cli".into(),
        receiver: "opencode".into(),
        timestamp: chrono::Utc::now(),
        content: String::new(),
        msg_type: MessageType::Pull,
        status: TaskStatus::Pending,
        parent_id: None,
    };
    send_message(&mut stream, &pull).await.unwrap();

    let resp = tokio::time::timeout(Duration::from_secs(2), recv_message(&mut stream))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resp.msg_type, MessageType::SnapshotData);
    assert_eq!(resp.content, "No pending task for 'opencode'");
    assert_eq!(resp.parent_id, Some(pull.id));
}

#[tokio::test]
async fn pull_for_unknown_window() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    spawn_server(state).await;

    let mut stream = UnixStream::connect(&sock).await.unwrap();
    let pull = Message {
        id: Uuid::new_v4(),
        sender: "cli".into(),
        receiver: "ghost".into(),
        timestamp: chrono::Utc::now(),
        content: String::new(),
        msg_type: MessageType::Pull,
        status: TaskStatus::Pending,
        parent_id: None,
    };
    send_message(&mut stream, &pull).await.unwrap();

    let resp = tokio::time::timeout(Duration::from_secs(2), recv_message(&mut stream))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resp.msg_type, MessageType::SnapshotData);
    assert_eq!(resp.content, "No pending task for 'ghost'");
}

#[tokio::test]
async fn result_pops_head_of_queue() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    let queues = state.queues.clone();
    spawn_server(state).await;

    // 先 push 两个 task
    let t1 = Message::new_task("cli", "opencode", "a");
    let t2 = Message::new_task("cli", "opencode", "b");
    let mut s = UnixStream::connect(&sock).await.unwrap();
    send_message(&mut s, &t1).await.unwrap();
    drop(s);
    let mut s = UnixStream::connect(&sock).await.unwrap();
    send_message(&mut s, &t2).await.unwrap();
    drop(s);

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if queues.read().await.len("opencode") == 2 {
            break;
        }
    }
    assert_eq!(queues.read().await.len("opencode"), 2);

    // opencode 回填 result：sender = opencode，应弹出队首
    let result = Message::new_result("opencode", "cli", "done a", t1.id);
    let mut s = UnixStream::connect(&sock).await.unwrap();
    send_message(&mut s, &result).await.unwrap();
    drop(s);

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if queues.read().await.len("opencode") == 1 {
            break;
        }
    }
    assert_eq!(queues.read().await.len("opencode"), 1);
    // 剩下的应该是 t2
    let remaining = queues.read().await;
    let peek = remaining.peek_next("opencode").unwrap();
    assert_eq!(peek.id, t2.id);
}

#[tokio::test]
async fn cancel_removes_task_from_queue() {
    let dir = tempfile::tempdir().unwrap();
    let state = build_state(dir.path()).await;
    let sock = state.session.read().await.socket_path.clone();
    let queues = state.queues.clone();
    spawn_server(state).await;

    let t = Message::new_task("cli", "opencode", "slow");
    let mut s = UnixStream::connect(&sock).await.unwrap();
    send_message(&mut s, &t).await.unwrap();
    drop(s);

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if queues.read().await.len("opencode") == 1 {
            break;
        }
    }
    assert_eq!(queues.read().await.len("opencode"), 1);

    let cancel = Message {
        id: Uuid::new_v4(),
        sender: "cli".into(),
        receiver: "opencode".into(),
        timestamp: chrono::Utc::now(),
        content: t.id.to_string(),
        msg_type: MessageType::Cancel,
        status: TaskStatus::Cancelled,
        parent_id: Some(t.id),
    };
    let mut s = UnixStream::connect(&sock).await.unwrap();
    send_message(&mut s, &cancel).await.unwrap();
    drop(s);

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        if queues.read().await.len("opencode") == 0 {
            break;
        }
    }
    assert_eq!(queues.read().await.len("opencode"), 0);
}
