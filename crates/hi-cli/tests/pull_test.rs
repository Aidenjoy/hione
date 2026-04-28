use hi_cli::commands::fetch;
use hi_core::ipc::{recv_message, send_message};
use hi_core::message::{Message, MessageType, TaskStatus};
use tokio::net::UnixListener;
use uuid::Uuid;

#[cfg(unix)]
async fn start_mock(sock: std::path::PathBuf, content: Option<String>) {
    let listener = UnixListener::bind(&sock).unwrap();
    tokio::spawn(async move {
        while let Ok((mut s, _)) = listener.accept().await {
            let req = recv_message(&mut s).await.unwrap();
            if let Some(c) = content.clone() {
                let resp = Message {
                    id: Uuid::new_v4(),
                    sender: "monitor".into(),
                    receiver: req.sender,
                    timestamp: chrono::Utc::now(),
                    content: c,
                    msg_type: MessageType::SnapshotData,
                    status: TaskStatus::Completed,
                    parent_id: Some(req.id),
                };
                let _ = send_message(&mut s, &resp).await;
            }
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
}

#[cfg(unix)]
#[tokio::test]
async fn fetch_returns_snapshot_content() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("s.sock");
    start_mock(sock.clone(), Some("terminal output".into())).await;
    let got = fetch(&sock.to_string_lossy(), "opencode", 2).await.unwrap();
    assert_eq!(got, Some("terminal output".into()));
}

#[cfg(unix)]
#[tokio::test]
async fn fetch_returns_none_on_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("s.sock");
    start_mock(sock.clone(), None).await;
    let got = fetch(&sock.to_string_lossy(), "opencode", 1).await.unwrap();
    assert!(got.is_none());
}