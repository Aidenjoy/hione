use anyhow::Result;
use hi_core::{
    ipc::send_message,
    message::{Message, MessageType, TaskStatus},
};
use uuid::Uuid;

use super::common::load_session;

pub async fn run(task_id_str: String) -> Result<()> {
    let task_id: Uuid = task_id_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid task ID: {task_id_str}"))?;
    let session = load_session()?;
    send_cancel(&session.socket_path, task_id).await?;
    println!("Cancelled task {task_id_str}");
    Ok(())
}

pub async fn send_cancel(socket_path: &str, task_id: Uuid) -> Result<()> {
    let cancel_msg = Message {
        id: Uuid::new_v4(),
        sender: "hi".to_string(),
        receiver: "monitor".to_string(),
        timestamp: chrono::Utc::now(),
        content: task_id.to_string(),
        msg_type: MessageType::Cancel,
        status: TaskStatus::Cancelled,
        parent_id: Some(task_id),
    };

    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path).await?;
        send_message(&mut stream, &cancel_msg).await?;
        Ok(())
    }
    #[cfg(windows)]
    {
        let _ = (socket_path, cancel_msg);
        anyhow::bail!("Windows named pipe not yet implemented");
    }
}
