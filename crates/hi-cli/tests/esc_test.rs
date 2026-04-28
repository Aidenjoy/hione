use hi_cli::commands::send_cancel;
use hi_core::ipc::recv_message;
use hi_core::message::{MessageType, TaskStatus};
use tokio::net::UnixListener;
use uuid::Uuid;

#[cfg(unix)]
#[tokio::test]
async fn send_cancel_delivers_cancel_message() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("s.sock");
    let listener = UnixListener::bind(&sock).unwrap();

    let task_id = Uuid::new_v4();
    let recv = tokio::spawn(async move {
        let (mut s, _) = listener.accept().await.unwrap();
        recv_message(&mut s).await.unwrap()
    });

    send_cancel(&sock.to_string_lossy(), task_id).await.unwrap();
    let got = recv.await.unwrap();
    assert_eq!(got.msg_type, MessageType::Cancel);
    assert_eq!(got.parent_id, Some(task_id));
    assert_eq!(got.content, task_id.to_string());

    let _ = TaskStatus::Cancelled;
}

// run function uses load_session which requires .hione/session.json
// This test is kept inline in esc.rs as it tests parsing logic