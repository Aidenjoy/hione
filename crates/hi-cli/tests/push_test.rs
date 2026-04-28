use hi_cli::commands::send_to_monitor;
use hi_core::ipc::recv_message;
use hi_core::message::Message;
use tokio::net::UnixListener;

#[cfg(unix)]
#[tokio::test]
async fn send_to_monitor_delivers_frame() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("t.sock");
    let sock_str = sock.to_string_lossy().to_string();
    let listener = UnixListener::bind(&sock).unwrap();

    let msg = Message::new_task("cli", "opencode", "do it");
    let id = msg.id;

    let recv = tokio::spawn(async move {
        let (mut s, _) = listener.accept().await.unwrap();
        recv_message(&mut s).await.unwrap()
    });

    send_to_monitor(&sock_str, &msg).await.unwrap();
    let got = recv.await.unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.content, "do it");
}

#[cfg(unix)]
#[tokio::test]
async fn send_to_monitor_errors_on_missing_socket() {
    let err = send_to_monitor("/tmp/definitely-not-a-real-socket-xyz", &Message::new_task("a", "b", "c"))
        .await
        .unwrap_err();
    assert!(format!("{err}").contains("hi-monitor"));
}