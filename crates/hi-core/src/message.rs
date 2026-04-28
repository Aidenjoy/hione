use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub receiver: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub msg_type: MessageType,
    pub status: TaskStatus,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Task,
    Result,
    Cancel,
    Check,
    CheckAck,
    Pull,
    Snapshot,
    SnapshotData,
    /// tmux 启动完成，携带 pane_id 映射
    SessionReady,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Timeout,
}

impl Message {
    pub fn new_task(sender: &str, receiver: &str, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            timestamp: Utc::now(),
            content: content.to_string(),
            msg_type: MessageType::Task,
            status: TaskStatus::Pending,
            parent_id: None,
        }
    }

    pub fn new_result(sender: &str, receiver: &str, content: &str, task_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            timestamp: Utc::now(),
            content: content.to_string(),
            msg_type: MessageType::Result,
            status: TaskStatus::Completed,
            parent_id: Some(task_id),
        }
    }

    pub fn new_check(sender: &str, receiver: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            timestamp: Utc::now(),
            content: String::new(),
            msg_type: MessageType::Check,
            status: TaskStatus::Pending,
            parent_id: None,
        }
    }
}
