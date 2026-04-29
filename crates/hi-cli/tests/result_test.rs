#[cfg(unix)]
use hi_cli::commands::submit;
#[cfg(unix)]
use hi_core::db::{init_db, insert_message};
#[cfg(unix)]
use hi_core::ipc::recv_message;
#[cfg(unix)]
use hi_core::message::Message;
#[cfg(unix)]
use uuid::Uuid;

#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(unix)]
#[tokio::test]
async fn submit_routes_result_to_original_sender() {
    let dir = tempfile::tempdir().unwrap();
    let pool = init_db(dir.path()).await.unwrap();

    let task = Message::new_task("claude", "opencode", "do");
    insert_message(&pool, &task).await.unwrap();

    let sock = dir.path().join("s.sock");
    let listener = UnixListener::bind(&sock).unwrap();
    let recv = tokio::spawn(async move {
        let (mut s, _) = listener.accept().await.unwrap();
        recv_message(&mut s).await.unwrap()
    });

    submit(
        dir.path(),
        &sock.to_string_lossy(),
        "opencode",
        task.id,
        "done!",
    )
    .await
    .unwrap();

    let got = recv.await.unwrap();
    assert_eq!(got.sender, "opencode");
    assert_eq!(got.receiver, "claude");
    assert_eq!(got.content, "done!");
    assert_eq!(got.parent_id, Some(task.id));
}

#[cfg(unix)]
#[tokio::test]
async fn submit_errors_when_task_not_in_db() {
    let dir = tempfile::tempdir().unwrap();
    let _ = init_db(dir.path()).await.unwrap();
    let bogus = Uuid::new_v4();
    let err = submit(dir.path(), "/tmp/nope.sock", "me", bogus, "x")
        .await
        .unwrap_err();
    assert!(format!("{err}").contains("not found in DB"));
}

// run function tests are kept inline in result.rs as they test UUID parsing
// which is independent of session loading