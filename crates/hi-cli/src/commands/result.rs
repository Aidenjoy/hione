use anyhow::Result;
use hi_core::{ipc::send_message, message::Message};
use std::path::Path;
use uuid::Uuid;

use super::common::{hione_dir, load_session};

pub async fn run(task_id_str: String, content: String) -> Result<()> {
    let task_id: Uuid = task_id_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid task ID: {task_id_str}"))?;
    let session = load_session()?;
    let sender = std::env::var("HI_WINDOW_NAME").unwrap_or_else(|_| "unknown".to_string());
    let hione_dir = hione_dir()?;
    submit(&hione_dir, &session.socket_path, &sender, task_id, &content).await?;
    println!("Result returned for task {task_id_str}");
    Ok(())
}

pub async fn submit(
    hione_dir: &Path,
    socket_path: &str,
    sender: &str,
    task_id: Uuid,
    content: &str,
) -> Result<()> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite:{}",
        hione_dir.join("hi.db").display()
    ))
    .await?;
    let original = hi_core::db::get_message_by_id(&pool, &task_id.to_string())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Task {task_id} not found in DB"))?;

    let result_msg = Message::new_result(sender, &original.sender, content, task_id);

    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path).await?;
        send_message(&mut stream, &result_msg).await?;
        Ok(())
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
        send_message(&mut stream, &result_msg).await?;
        Ok(())
    }
}
