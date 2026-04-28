use hi_cli::commands::probe;
use hi_core::ipc::{recv_message, send_message};
use hi_core::message::{Message, MessageType, TaskStatus};
use tokio::net::UnixListener;
use uuid::Uuid;

#[cfg(unix)]
async fn start_mock(sock: std::path::PathBuf, respond_ack: bool) {
    let listener = UnixListener::bind(&sock).unwrap();
    tokio::spawn(async move {
        while let Ok((mut s, _)) = listener.accept().await {
            let req = recv_message(&mut s).await.unwrap();
            if respond_ack {
                let ack = Message {
                    id: Uuid::new_v4(),
                    sender: "monitor".into(),
                    receiver: req.sender,
                    timestamp: chrono::Utc::now(),
                    content: "ok".into(),
                    msg_type: MessageType::CheckAck,
                    status: TaskStatus::Completed,
                    parent_id: Some(req.id),
                };
                let _ = send_message(&mut s, &ack).await;
            }
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
}

#[cfg(unix)]
#[tokio::test]
async fn probe_returns_true_on_ack() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("s.sock");
    start_mock(sock.clone(), true).await;
    let ok = probe(&sock.to_string_lossy(), "opencode", 2).await.unwrap();
    assert!(ok);
}

#[cfg(unix)]
#[tokio::test]
async fn probe_returns_false_on_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("s.sock");
    start_mock(sock.clone(), false).await;
    let ok = probe(&sock.to_string_lossy(), "opencode", 1).await.unwrap();
    assert!(!ok);
}