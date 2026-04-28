use crate::error::AppError;
use hi_core::message::{Message, MessageType, TaskStatus};
use uuid::Uuid;
use chrono::Utc;

pub struct IpcClient {
    socket_path: String,
}

impl IpcClient {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn push_task(&self, sender: &str, target: &str, content: &str) -> Result<String, AppError> {
        let task_id = Uuid::new_v4();
        let msg = Message {
            id: task_id,
            sender: sender.to_string(),
            receiver: target.to_string(),
            timestamp: Utc::now(),
            content: content.to_string(),
            msg_type: MessageType::Task,
            status: TaskStatus::Pending,
            parent_id: None,
        };
        self.send_only(&msg).await?;
        Ok(task_id.to_string())
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<(), AppError> {
        let uuid = Uuid::parse_str(task_id)
            .map_err(|_| AppError::CommandFailed(format!("Invalid task_id: {}", task_id)))?;
        let msg = Message {
            id: Uuid::new_v4(),
            sender: "desktop".to_string(),
            receiver: "".to_string(),
            timestamp: Utc::now(),
            content: "".to_string(),
            msg_type: MessageType::Cancel,
            status: TaskStatus::Pending,
            parent_id: Some(uuid),
        };
        self.send_only(&msg).await
    }

    pub async fn check_agent(&self, name: &str) -> Result<bool, AppError> {
        self.check_agent_impl(name).await
    }
}

// ── platform implementations ────────────────────────────────────────────────

#[cfg(unix)]
impl IpcClient {
    async fn connect(&self) -> Result<tokio::net::UnixStream, AppError> {
        tokio::net::UnixStream::connect(&self.socket_path).await
            .map_err(|e| AppError::IpcError(e.to_string()))
    }

    async fn send_only(&self, msg: &Message) -> Result<(), AppError> {
        let mut stream = self.connect().await?;
        hi_core::ipc::send_message(&mut stream, msg).await
            .map_err(|e| AppError::IpcError(e.to_string()))
    }

    async fn check_agent_impl(&self, name: &str) -> Result<bool, AppError> {
        let mut stream = self.connect().await?;
        let msg = Message {
            id: Uuid::new_v4(),
            sender: "desktop".to_string(),
            receiver: name.to_string(),
            timestamp: Utc::now(),
            content: "".to_string(),
            msg_type: MessageType::Check,
            status: TaskStatus::Pending,
            parent_id: None,
        };
        hi_core::ipc::send_message(&mut stream, &msg).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            hi_core::ipc::recv_message(&mut stream),
        ).await;

        match result {
            Ok(Ok(resp)) => Ok(resp.msg_type == MessageType::CheckAck),
            _ => Ok(false),
        }
    }
}

#[cfg(windows)]
impl IpcClient {
    async fn send_only(&self, _msg: &Message) -> Result<(), AppError> {
        Err(AppError::IpcError(
            "IPC over Unix sockets is not supported on Windows".to_string(),
        ))
    }

    async fn check_agent_impl(&self, _name: &str) -> Result<bool, AppError> {
        Ok(false)
    }
}

#[cfg(not(any(unix, windows)))]
impl IpcClient {
    async fn send_only(&self, _msg: &Message) -> Result<(), AppError> {
        Err(AppError::IpcError("Unsupported platform".to_string()))
    }

    async fn check_agent_impl(&self, _name: &str) -> Result<bool, AppError> {
        Ok(false)
    }
}
