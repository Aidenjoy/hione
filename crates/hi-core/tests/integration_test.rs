// tests/integration_test.rs
// 测试 IPC 帧读写端到端
use hi_core::message::Message;

#[tokio::test]
async fn test_ipc_roundtrip() {
    let socket_path = "/tmp/hi-test.sock";
    let _ = std::fs::remove_file(socket_path);

    #[cfg(unix)]
    {
        use tokio::net::UnixListener;
        let listener = UnixListener::bind(socket_path).unwrap();

        let msg_sent = Message::new_task("alice", "bob", "hello world");
        let msg_clone = msg_sent.clone();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let received = recv_message(&mut stream).await.unwrap();
            assert_eq!(received.sender, "alice");
            assert_eq!(received.receiver, "bob");
            assert_eq!(received.content, "hello world");
        });

        let client = tokio::spawn(async move {
            use tokio::net::UnixStream;
            let mut stream = UnixStream::connect(socket_path).await.unwrap();
            send_message(&mut stream, &msg_clone).await.unwrap();
        });

        let _ = tokio::join!(server, client);
    }
}

#[tokio::test]
async fn test_db_message_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let pool = hi_core::db::init_db(dir.path()).await.unwrap();
    let msg = Message::new_task("sender", "receiver", "test task");
    hi_core::db::insert_message(&pool, &msg).await.unwrap();
    let retrieved = hi_core::db::get_message_by_id(&pool, &msg.id.to_string()).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, "test task");
}