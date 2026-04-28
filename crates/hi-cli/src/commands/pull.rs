use anyhow::Result;
use hi_core::{
    ipc::{recv_message, send_message},
    message::{Message, MessageType, TaskStatus},
};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use super::common::load_session;

pub async fn run(target: String, timeout_secs: u64) -> Result<()> {
    let session = load_session()?;
    match fetch(&session.socket_path, &target, timeout_secs).await? {
        Some(content) => {
            println!("{content}");
            Ok(())
        }
        None => {
            println!("[pull] timeout or no response from monitor for '{target}'");
            std::process::exit(1);
        }
    }
}

pub async fn fetch(
    socket_path: &str,
    target: &str,
    timeout_secs: u64,
) -> Result<Option<String>> {
    let pull_msg = Message {
        id: Uuid::new_v4(),
        sender: "hi".to_string(),
        receiver: target.to_string(),
        timestamp: chrono::Utc::now(),
        content: String::new(),
        msg_type: MessageType::Pull,
        status: TaskStatus::Pending,
        parent_id: None,
    };

    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path).await?;
        send_message(&mut stream, &pull_msg).await?;
        let result = timeout(Duration::from_secs(timeout_secs), recv_message(&mut stream)).await;
        Ok(match result {
            Ok(Ok(resp)) if resp.msg_type == MessageType::SnapshotData => Some(resp.content),
            _ => None,
        })
    }
    #[cfg(windows)]
    {
        let _ = (socket_path, target, timeout_secs);
        anyhow::bail!("Windows named pipe not yet implemented");
    }
}
