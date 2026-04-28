use hi_core::db::init_db;
use hi_core::ipc::{recv_message, send_message};
use hi_core::message::{Message, MessageType, TaskStatus};
use hi_core::session::{SessionInfo, WindowInfo};
use hi_monitor::snapshot::request_snapshot;
use hi_monitor::server::MonitorState;
use uuid::Uuid;

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

fn mk_req(target: &str) -> Message {
    Message {
        id: Uuid::new_v4(),
        sender: "monitor".into(),
        receiver: target.into(),
        timestamp: chrono::Utc::now(),
        content: String::new(),
        msg_type: MessageType::Snapshot,
        status: TaskStatus::Pending,
        parent_id: None,
    }
}

#[cfg(unix)]
#[tokio::test]
async fn request_snapshot_missing_sock_is_silent_ok() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;
    let req = mk_req("opencode");
    assert!(request_snapshot(&state, &req, &None, "opencode", dir.path()).await.is_ok());
    assert!(state.snapshots.read().await.is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn request_snapshot_updates_state_on_successful_response() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;
    let snap_sock = dir.path().join("snap.sock");

    let listener = tokio::net::UnixListener::bind(&snap_sock).unwrap();
    tokio::spawn(async move {
        if let Ok((mut s, _)) = listener.accept().await {
            let req = recv_message(&mut s).await.unwrap();
            let resp = Message {
                id: Uuid::new_v4(),
                sender: "tauri".into(),
                receiver: req.sender,
                timestamp: chrono::Utc::now(),
                content: "hello world".into(),
                msg_type: MessageType::SnapshotData,
                status: TaskStatus::Completed,
                parent_id: Some(req.id),
            };
            send_message(&mut s, &resp).await.unwrap();
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let req = mk_req("opencode");
    request_snapshot(&state, &req, &None, "opencode", dir.path()).await.unwrap();
    let snaps = state.snapshots.read().await;
    assert_eq!(snaps.get("opencode").map(String::as_str), Some("hello world"));
}

#[cfg(unix)]
#[tokio::test]
async fn request_snapshot_timeout_returns_err_without_updating_state() {
    let dir = tempfile::tempdir().unwrap();
    let state = mk_state(dir.path()).await;
    let snap_sock = dir.path().join("snap.sock");

    let listener = tokio::net::UnixListener::bind(&snap_sock).unwrap();
    tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let req = mk_req("opencode");
    let err = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        request_snapshot(&state, &req, &None, "opencode", dir.path()),
    )
    .await
    .unwrap();
    assert!(err.is_err());
    assert!(state.snapshots.read().await.is_empty());
}