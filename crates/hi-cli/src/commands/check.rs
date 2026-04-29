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
        use interprocess::local_socket::tokio::prelude::LocalSocketStream;
        use interprocess::local_socket::traits::tokio::Stream;
        use interprocess::local_socket::{ToNsName, GenericNamespaced};

        // socket_path is "hione_<hash>" format, extract the name part
        let pipe_name = socket_path.split('\\').last().unwrap_or(socket_path);
        let name = pipe_name.to_ns_name::<GenericNamespaced>()
            .map_err(|e| anyhow::anyhow!("Failed to create pipe name: {}", e))?;
        let mut stream = LocalSocketStream::connect(name)
            .await
            .map_err(|e| anyhow::anyhow!("Cannot connect to hi-monitor named pipe: {}", e))?;
        send_message(&mut stream, &check_msg).await?;
        let result = timeout(Duration::from_secs(timeout_secs), recv_message(&mut stream)).await;
        Ok(matches!(
            result,
            Ok(Ok(ack)) if ack.msg_type == MessageType::CheckAck
        ))
    }
}
