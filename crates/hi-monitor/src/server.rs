use anyhow::Result;
use hi_core::{
    db::insert_message,
    history::supported_tool_name,
    ipc::{recv_message, send_message},
    message::{Message, MessageType, TaskStatus},
    protocol::{extract_result, format_result_envelope, format_task_envelope},
    session::SessionInfo,
};
use sqlx::SqlitePool;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    task_queue::TaskQueueMap,
    tmux::{capture_pane, deliver_to_pane},
};

pub const PULL_COOLDOWN_SECS: u64 = 60;

#[derive(Clone)]
pub struct MonitorState {
    pub session: Arc<RwLock<SessionInfo>>,
    pub pool: SqlitePool,
    pub hione_dir: PathBuf,
    pub queues: Arc<RwLock<TaskQueueMap>>,
    pub snapshots: Arc<RwLock<HashMap<String, String>>>,
    pub pending_tasks: Arc<RwLock<HashMap<Uuid, (String, String)>>>,
    pub pull_cooldown: Arc<RwLock<HashMap<Uuid, Instant>>>,
    /// 每个窗口最后一次收到任务的时刻，用于 stuck 检测过滤旧内容
    pub task_dispatch_times: Arc<RwLock<HashMap<String, Instant>>>,
    /// 每个窗口任务派发时刻的结构化存储快照，用于检测新回复
    pub response_baselines: Arc<RwLock<HashMap<String, String>>>,
}

impl MonitorState {
    pub fn new(session: SessionInfo, pool: SqlitePool, hione_dir: PathBuf) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            pool,
            hione_dir,
            queues: Arc::new(RwLock::new(TaskQueueMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            pending_tasks: Arc::new(RwLock::new(HashMap::new())),
            pull_cooldown: Arc::new(RwLock::new(HashMap::new())),
            task_dispatch_times: Arc::new(RwLock::new(HashMap::new())),
            response_baselines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn reload_session(&self) -> Result<()> {
        let session_path = self.hione_dir.join("session.json");
        let session_json = std::fs::read_to_string(&session_path)?;
        let session: SessionInfo = serde_json::from_str(&session_json)?;

        let mut current = self.session.write().await;
        *current = session;

        tracing::info!("Reloaded session from {}", session_path.display());
        Ok(())
    }
}

pub async fn run(state: MonitorState) -> Result<()> {
    #[cfg(unix)]
    {
        let socket_path = state.session.read().await.socket_path.clone();
        // 清理旧 socket 文件
        let _ = std::fs::remove_file(&socket_path);
        let listener = tokio::net::UnixListener::bind(&socket_path)?;
        tracing::info!("IPC server listening on {}", socket_path);

        loop {
            let (stream, _) = listener.accept().await?;
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection_unix(stream, state).await {
                    tracing::error!("Connection error: {e}");
                }
            });
        }
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::traits::tokio::Listener;
        use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
        // Windows uses namespaced pipes, get name from session's socket_path
        let socket_path = state.session.read().await.socket_path.clone();
        // socket_path is "hione_<hash>" format, extract the name part for to_ns_name
        let pipe_name = socket_path
            .split(['\\', '/'])
            .last()
            .unwrap_or(&socket_path);
        let name = pipe_name.to_ns_name::<GenericNamespaced>()?;
        let listener = ListenerOptions::new().name(name).create_tokio()?;
        tracing::info!("IPC server listening on named pipe '{}'", pipe_name);

        loop {
            let stream = listener.accept().await?;
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection_windows(stream, state).await {
                    tracing::error!("Connection error: {e}");
                }
            });
        }
    }
    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(unix)]
async fn handle_connection_unix(
    mut stream: tokio::net::UnixStream,
    state: MonitorState,
) -> Result<()> {
    handle_connection_inner(&mut stream, state).await
}

#[cfg(windows)]
async fn handle_connection_windows(
    mut stream: interprocess::local_socket::tokio::Stream,
    state: MonitorState,
) -> Result<()> {
    handle_connection_inner(&mut stream, state).await
}

async fn handle_connection_inner<S>(stream: &mut S, state: MonitorState) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let msg = recv_message(&mut *stream).await?;
    tracing::info!(
        "Received {:?} from {} -> {}",
        msg.msg_type,
        msg.sender,
        msg.receiver
    );

    // 持久化所有消息
    insert_message(&state.pool, &msg).await?;

    match msg.msg_type {
        MessageType::Task => {
            let mut queues = state.queues.write().await;
            queues.enqueue(&msg.receiver, msg.clone());
            tracing::info!("Enqueued task {} for '{}'", msg.id, msg.receiver);

            // 先 reload session 文件，确保获取最新的 tmux_pane_id
            //（因为 hi s 启动流程中 tmux_pane_id 在 monitor 启动后才写入）
            if let Err(e) = state.reload_session().await {
                tracing::warn!("Failed to reload session before task dispatch: {e}");
            }

            let session = state.session.read().await;
            let pane_id = session
                .windows
                .iter()
                .find(|w| w.name == msg.receiver)
                .and_then(|w| w.tmux_pane_id.clone());

            if let Some(pane_id) = pane_id {
                let peers: Vec<String> = session
                    .windows
                    .iter()
                    .filter(|w| !w.is_main && w.name != msg.receiver)
                    .map(|w| w.name.clone())
                    .collect();

                let work_dir = session.work_dir.clone();

                let envelope =
                    format_task_envelope(&msg.id, &msg.sender, &msg.receiver, &msg.content, &peers);
                if let Err(e) = deliver_to_pane(&pane_id, &envelope) {
                    tracing::warn!("Failed to deliver task to tmux pane {}: {}", pane_id, e);
                } else {
                    let mut pending = state.pending_tasks.write().await;
                    pending.insert(msg.id, (msg.sender.clone(), msg.receiver.clone()));
                    let mut dispatch_times = state.task_dispatch_times.write().await;
                    dispatch_times.insert(msg.receiver.clone(), Instant::now());
                    tracing::info!("Task {} pending, waiting for DONE marker", msg.id);

                    // 记录结构化存储基线（异步，不阻塞主流程）
                    let baselines = state.response_baselines.clone();
                    let window_name = msg.receiver.clone();
                    tokio::spawn(async move {
                        let baseline =
                            hi_core::history::read_latest_response(&window_name, &work_dir)
                                .await
                                .unwrap_or_default();
                        baselines.write().await.insert(window_name, baseline);
                    });
                }
            } else {
                tracing::warn!(
                    "No tmux_pane_id found for '{}', task remains in queue",
                    msg.receiver
                );
            }
        }
        MessageType::Result => {
            let task_id = msg.parent_id.unwrap_or(Uuid::nil());
            tracing::info!("Task {} completed by '{}'", task_id, msg.sender);
            // 将已完成的任务从队列移除（sender 就是完成任务的 window）
            let mut queues = state.queues.write().await;
            queues.pop_next(&msg.sender);
        }
        MessageType::Cancel => {
            let task_id = msg.parent_id.unwrap_or(Uuid::nil());
            let mut queues = state.queues.write().await;
            queues.cancel(&msg.receiver, task_id);
            tracing::info!("Cancelled task {task_id}");
        }
        MessageType::Check => {
            let session = state.session.read().await;
            let window_exists = session.windows.iter().any(|w| w.name == msg.receiver);
            if !window_exists {
                return Ok(());
            }
            let ack = Message {
                id: Uuid::new_v4(),
                sender: "monitor".to_string(),
                receiver: msg.sender.clone(),
                timestamp: chrono::Utc::now(),
                content: "ok".to_string(),
                msg_type: MessageType::CheckAck,
                status: TaskStatus::Completed,
                parent_id: Some(msg.id),
            };
            send_message(&mut *stream, &ack).await?;
        }
        MessageType::Pull => {
            let task_info = {
                let pending = state.pending_tasks.read().await;
                pending
                    .iter()
                    .find(|(_, (_, receiver))| receiver == &msg.receiver)
                    .map(|(id, (sender, receiver))| (*id, sender.clone(), receiver.clone()))
            };

            let (task_id, task_sender, task_receiver) = match task_info {
                Some(t) => t,
                None => {
                    let resp = Message {
                        id: Uuid::new_v4(),
                        sender: "monitor".to_string(),
                        receiver: msg.sender.clone(),
                        timestamp: chrono::Utc::now(),
                        content: format!("No pending task for '{}'", msg.receiver),
                        msg_type: MessageType::SnapshotData,
                        status: TaskStatus::Completed,
                        parent_id: Some(msg.id),
                    };
                    send_message(&mut *stream, &resp).await?;
                    return Ok(());
                }
            };

            let now = Instant::now();
            let elapsed_since_last_pull = {
                let cooldowns = state.pull_cooldown.read().await;
                cooldowns.get(&task_id).map(|t| t.elapsed().as_secs())
            };
            if let Some(elapsed) = elapsed_since_last_pull {
                if elapsed < PULL_COOLDOWN_SECS {
                    let resp = Message {
                        id: Uuid::new_v4(),
                        sender: "monitor".to_string(),
                        receiver: msg.sender.clone(),
                        timestamp: chrono::Utc::now(),
                        content: format!(
                            "Task {} may still be working (last pull was {}s ago, cooldown is {}s)",
                            task_id, elapsed, PULL_COOLDOWN_SECS
                        ),
                        msg_type: MessageType::SnapshotData,
                        status: TaskStatus::Pending,
                        parent_id: Some(msg.id),
                    };
                    send_message(&mut *stream, &resp).await?;
                    return Ok(());
                }
            }

            let session = state.session.read().await;
            let task_receiver_clone = task_receiver.clone();
            let snapshot = if let Some(content) =
                hi_core::history::read_latest_response(&task_receiver_clone, &session.work_dir)
                    .await
            {
                content
            } else if supported_tool_name(&task_receiver).is_some() {
                String::new()
            } else {
                let pane_id_opt = session
                    .windows
                    .iter()
                    .find(|w| w.name == task_receiver)
                    .and_then(|w| w.tmux_pane_id.clone());

                if let Some(pane_id) = pane_id_opt {
                    tokio::task::spawn_blocking(move || capture_pane(&pane_id).unwrap_or_default())
                        .await
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            };

            if snapshot.trim().is_empty() {
                let resp = Message {
                    id: Uuid::new_v4(),
                    sender: "monitor".to_string(),
                    receiver: msg.sender.clone(),
                    timestamp: chrono::Utc::now(),
                    content: format!(
                        "No content available for '{}', task remains pending",
                        msg.receiver
                    ),
                    msg_type: MessageType::SnapshotData,
                    status: TaskStatus::Pending,
                    parent_id: Some(msg.id),
                };
                send_message(&mut *stream, &resp).await?;
                return Ok(());
            }

            let (result_content, task_status) = match extract_result(&snapshot, &task_id) {
                Some(clean) => (clean, TaskStatus::Completed),
                None => (snapshot.clone(), TaskStatus::Timeout),
            };

            let sender_pane_id = session
                .windows
                .iter()
                .find(|w| w.name == task_sender)
                .and_then(|w| w.tmux_pane_id.clone());

            let delivered = if let Some(pane_id) = sender_pane_id {
                let envelope =
                    format_result_envelope(&task_id, &task_receiver, &task_sender, &result_content);
                match deliver_to_pane(&pane_id, &envelope) {
                    Ok(()) => true,
                    Err(e) => {
                        tracing::warn!("pull: failed to deliver result to pane {}: {}", pane_id, e);
                        false
                    }
                }
            } else {
                true
            };

            if delivered {
                state.pull_cooldown.write().await.insert(task_id, now);

                let result_msg = Message {
                    id: Uuid::new_v4(),
                    sender: task_receiver.clone(),
                    receiver: task_sender.clone(),
                    timestamp: chrono::Utc::now(),
                    content: result_content.clone(),
                    msg_type: MessageType::Result,
                    status: task_status.clone(),
                    parent_id: Some(task_id),
                };
                if let Err(e) = insert_message(&state.pool, &result_msg).await {
                    tracing::error!("pull: failed to save result: {e}");
                }

                {
                    let mut pending = state.pending_tasks.write().await;
                    pending.remove(&task_id);
                }
                {
                    let mut queues = state.queues.write().await;
                    queues.pop_next(&task_receiver);
                }

                let ack_content = format!(
                    "Pulled result for task {} from '{}' (status: {:?})",
                    task_id, msg.receiver, task_status
                );
                tracing::info!("{}", ack_content);
                let resp = Message {
                    id: Uuid::new_v4(),
                    sender: "monitor".to_string(),
                    receiver: msg.sender.clone(),
                    timestamp: chrono::Utc::now(),
                    content: ack_content,
                    msg_type: MessageType::SnapshotData,
                    status: TaskStatus::Completed,
                    parent_id: Some(msg.id),
                };
                send_message(&mut *stream, &resp).await?;
            } else {
                let resp = Message {
                    id: Uuid::new_v4(),
                    sender: "monitor".to_string(),
                    receiver: msg.sender.clone(),
                    timestamp: chrono::Utc::now(),
                    content: format!(
                        "Failed to deliver result to sender '{}', task remains pending",
                        task_sender
                    ),
                    msg_type: MessageType::SnapshotData,
                    status: TaskStatus::Pending,
                    parent_id: Some(msg.id),
                };
                send_message(&mut *stream, &resp).await?;
            }
        }
        MessageType::SessionReady => {
            // tmux 启动完成，CLI 发送完整的 session.json 内容
            // 直接更新内存中的 session 状态
            let session: SessionInfo = match serde_json::from_str(&msg.content) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to parse SessionReady content: {e}");
                    return Ok(());
                }
            };
            let mut current = state.session.write().await;
            *current = session;
            tracing::info!("Session ready, updated pane_id mappings");
        }
        _ => {
            tracing::warn!("Unhandled message type: {:?}", msg.msg_type);
        }
    }
    Ok(())
}
