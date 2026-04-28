use hi_core::ipc::{recv_message, send_message};
use hi_core::message::Message;
use tokio::io::duplex;

#[tokio::test]
async fn roundtrip_via_duplex() {
    let (mut a, mut b) = duplex(4096);
    let msg = Message::new_task("alice", "bob", "hello");
    let id = msg.id;

    tokio::spawn(async move {
        send_message(&mut a, &msg).await.unwrap();
    });

    let got = recv_message(&mut b).await.unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.sender, "alice");
    assert_eq!(got.content, "hello");
}

#[tokio::test]
async fn rejects_oversize_frame() {
    let (mut a, mut b) = duplex(8);
    let bogus_len: u32 = 17 * 1024 * 1024;
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let _ = a.write_all(&bogus_len.to_be_bytes()).await;
    });

    let err = recv_message(&mut b).await.unwrap_err();
    match err {
        hi_core::error::HiError::IpcConnect(m) => assert!(m.contains("too large")),
        other => panic!("expected IpcConnect, got {other:?}"),
    }
}

#[tokio::test]
async fn truncated_stream_errors() {
    let (mut a, mut b) = duplex(4);
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let _ = a.write_all(&[0u8, 0u8]).await;
        drop(a);
    });

    let err = recv_message(&mut b).await.unwrap_err();
    match err {
        hi_core::error::HiError::Io(_) => {}
        other => panic!("expected Io, got {other:?}"),
    }
}

#[tokio::test]
async fn invalid_json_payload_errors() {
    let (mut a, mut b) = duplex(64);
    let junk = b"{not valid json";
    let len = (junk.len() as u32).to_be_bytes();
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        a.write_all(&len).await.unwrap();
        a.write_all(junk).await.unwrap();
    });

    let err = recv_message(&mut b).await.unwrap_err();
    match err {
        hi_core::error::HiError::Serialize(_) => {}
        other => panic!("expected Serialize, got {other:?}"),
    }
}