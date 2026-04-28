use crate::error::AppError;
use hi_core::ipc::{send_message, recv_message};
use hi_core::message::{Message, MessageType, TaskStatus};
use tokio::net::UnixStream;
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
        let mut stream = UnixStream::connect(&self.socket_path).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
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
        
        send_message(&mut stream, &msg).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
        Ok(task_id.to_string())
    }
    
    pub async fn cancel_task(&self, task_id: &str) -> Result<(), AppError> {
        let mut stream = UnixStream::connect(&self.socket_path).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
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
        
        send_message(&mut stream, &msg).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn check_agent(&self, name: &str) -> Result<bool, AppError> {
        let mut stream = UnixStream::connect(&self.socket_path).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
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
        
        send_message(&mut stream, &msg).await
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            recv_message(&mut stream)
        ).await;
        
        match result {
            Ok(Ok(resp)) => Ok(resp.msg_type == MessageType::CheckAck),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }
}