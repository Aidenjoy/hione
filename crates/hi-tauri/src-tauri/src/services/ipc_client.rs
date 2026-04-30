use crate::error::AppError;
use chrono::Utc;
use hi_core::message::{Message, MessageType, TaskStatus};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct IpcClient {
    socket_path: String,
    hione_dir: Option<PathBuf>,
}

impl IpcClient {
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            hione_dir: None,
        }
    }

    pub fn with_hione_dir(socket_path: String, hione_dir: PathBuf) -> Self {
        Self {
            socket_path,
            hione_dir: Some(hione_dir),
        }
    }

    pub async fn push_task(
        &self,
        sender: &str,
        target: &str,
        content: &str,
    ) -> Result<String, AppError> {
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

fn is_stale_socket_error(err: &AppError) -> bool {
    match err {
        AppError::IpcError(s) | AppError::Io(s) => {
            s.contains("Connection refused")
                || s.contains("No such file or directory")
                || s.contains("os error 2")
                || s.contains("os error 61")
                || s.contains("os error 111")
                || s.contains("named pipe")
                || s.contains("系统找不到指定的文件")
        }
        _ => false,
    }
}

async fn restart_monitor(hione_dir: &Path) -> Result<(), AppError> {
    let monitor_bin = locate_monitor_bin()?;
    let mut cmd = std::process::Command::new(&monitor_bin);
    cmd.arg("--hione-dir").arg(hione_dir);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn().map_err(|e| {
        AppError::CommandFailed(format!(
            "Failed to spawn hi-monitor at {}: {}",
            monitor_bin.display(),
            e
        ))
    })?;

    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    Ok(())
}

fn locate_monitor_bin() -> Result<PathBuf, AppError> {
    if let Ok(path) = which::which("hi-monitor") {
        return Ok(path);
    }

    let exe = std::env::current_exe()?;
    let parent = exe.parent().ok_or_else(|| {
        AppError::CommandFailed("Cannot get current executable directory".to_string())
    })?;

    #[cfg(windows)]
    {
        let sibling = parent.join("hi-monitor.exe");
        if sibling.exists() {
            return Ok(sibling);
        }
    }

    let sibling = parent.join("hi-monitor");
    if sibling.exists() {
        return Ok(sibling);
    }

    Err(AppError::CommandFailed(
        "hi-monitor not found in PATH or alongside hi-tauri".to_string(),
    ))
}

// ── platform implementations ────────────────────────────────────────────────

#[cfg(unix)]
impl IpcClient {
    async fn connect_once(&self) -> Result<tokio::net::UnixStream, AppError> {
        tokio::net::UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| AppError::IpcError(e.to_string()))
    }

    async fn connect(&self) -> Result<tokio::net::UnixStream, AppError> {
        match self.connect_once().await {
            Ok(stream) => Ok(stream),
            Err(err) if is_stale_socket_error(&err) => {
                if let Some(hione_dir) = &self.hione_dir {
                    let _ = std::fs::remove_file(&self.socket_path);
                    restart_monitor(hione_dir).await?;
                    self.connect_once().await
                } else {
                    Err(err)
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn send_only(&self, msg: &Message) -> Result<(), AppError> {
        let mut stream = self.connect().await?;
        hi_core::ipc::send_message(&mut stream, msg)
            .await
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
        hi_core::ipc::send_message(&mut stream, &msg)
            .await
            .map_err(|e| AppError::IpcError(e.to_string()))?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            hi_core::ipc::recv_message(&mut stream),
        )
        .await;

        match result {
            Ok(Ok(resp)) => Ok(resp.msg_type == MessageType::CheckAck),
            _ => Ok(false),
        }
    }
}

#[cfg(windows)]
impl IpcClient {
    async fn connect_once(&self) -> Result<interprocess::local_socket::tokio::Stream, AppError> {
        use interprocess::local_socket::tokio::prelude::LocalSocketStream;
        use interprocess::local_socket::traits::tokio::Stream;
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        // socket_path is "hione_<hash>" format, extract the name part
        let pipe_name = self
            .socket_path
            .split(['\\', '/'])
            .last()
            .unwrap_or(&self.socket_path);
        let name = pipe_name
            .to_ns_name::<GenericNamespaced>()
            .map_err(|e| AppError::IpcError(e.to_string()))?;
        LocalSocketStream::connect(name)
            .await
            .map_err(|e| AppError::IpcError(e.to_string()))
    }

    async fn connect(&self) -> Result<interprocess::local_socket::tokio::Stream, AppError> {
        match self.connect_once().await {
            Ok(stream) => Ok(stream),
            Err(err) if is_stale_socket_error(&err) => {
                if let Some(hione_dir) = &self.hione_dir {
                    restart_monitor(hione_dir).await?;
                    self.connect_once().await
                } else {
                    Err(err)
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn send_only(&self, msg: &Message) -> Result<(), AppError> {
        let mut stream = self.connect().await?;
        hi_core::ipc::send_message(&mut stream, msg)
            .await
            .map_err(|e| AppError::IpcError(e.to_string()))
    }

    async fn check_agent_impl(&self, name: &str) -> Result<bool, AppError> {
        let mut stream = self.connect().await?;
        let msg = Message {
            id: Uuid::new_v4(),
            sender: "desktop".to_string(),
            receiver: name.to_string(),
            timestamp: chrono::Utc::now(),
            content: "".to_string(),
            msg_type: MessageType::Check,
            status: TaskStatus::Pending,
            parent_id: None,
        };
        hi_core::ipc::send_message(&mut stream, &msg)
            .await
            .map_err(|e| AppError::IpcError(e.to_string()))?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            hi_core::ipc::recv_message(&mut stream),
        )
        .await;

        match result {
            Ok(Ok(resp)) => Ok(resp.msg_type == MessageType::CheckAck),
            _ => Ok(false),
        }
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
