use anyhow::Result;
use hi_core::{
    ipc::{recv_message, send_message},
    message::{Message, MessageType},
};
use std::time::Duration;
use tokio::time::timeout;

use super::common::load_session;

pub async fn run(target: String, timeout_secs: u64) -> Result<()> {
    let session = load_session()?;
    match probe(&session.socket_path, &target, timeout_secs).await? {
        true => {
            println!("✓ '{target}' is online");
            Ok(())
        }
        false => {
            println!("✗ '{target}' is not responding (timeout: {timeout_secs}s)");
            std::process::exit(1);
        }
    }
}

pub async fn probe(socket_path: &str, target: &str, timeout_secs: u64) -> Result<bool> {
    let check_msg = Message::new_check("hi", target);

    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path).await?;
        send_message(&mut stream, &check_msg).await?;
        let result = timeout(Duration::from_secs(timeout_secs), recv_message(&mut stream)).await;
        Ok(matches!(
            result,
            Ok(Ok(ack)) if ack.msg_type == MessageType::CheckAck
        ))
    }
    #[cfg(windows)]
    {
        let _ = (socket_path, timeout_secs);
        anyhow::bail!("Windows named pipe not yet implemented");
    }
}
