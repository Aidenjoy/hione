use crate::{server::{MonitorState, PULL_COOLDOWN_SECS}, snapshot::poll_snapshots, tmux::deliver_to_pane};
use anyhow::Result;
use hi_core::{
    db::insert_message,
    history::read_latest_response,
    message::{Message, MessageType, TaskStatus},
    protocol::{extract_result, format_result_envelope},
};
use std::{collections::HashMap, time::Duration};
use tokio::time;
use uuid::Uuid;

const STUCK_THRESHOLD_SECS: u64 = 30;
const CHECK_INTERVAL_SECS: u64 = 2;

fn is_known_tool(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "claude" | "gemini" | "opencode" | "codex" | "qwen"
    )
}

pub async fn run(state: MonitorState) -> Result<()> {
    tokio::join!(
        poll_snapshots(state.clone()),
        detect_stuck(state.clone()),
        detect_task_done_structured(state.clone()),
        detect_task_done(state.clone()),
    );
    Ok(())
}

async fn detect_stuck(state: MonitorState) {
    let mut last_seen: HashMap<String, (String, std::time::Instant)> = HashMap::new();
    let mut interval = time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));

    loop {
        interval.tick().await;
        let snapshots = state.snapshots.read().await.clone();
        let dispatch_times = state.task_dispatch_times.read().await.clone();

        for (window_name, content) in &snapshots {
            let now = std::time::Instant::now();
            let entry = last_seen.entry(window_name.clone());
            let (last_content, last_changed) = entry.or_insert((content.clone(), now));

            if content != last_content {
                *last_content = content.clone();
                *last_changed = now;
            } else if last_changed.elapsed().as_secs() >= STUCK_THRESHOLD_SECS {
                // 已知工具由结构化存储检测，不走快照 auto-pull
                if is_known_tool(window_name) {
                    *last_changed = now;
                    continue;
                }

                // 若当前内容的最后变化时刻早于任务派发时刻，说明内容是旧任务的残留
                if let Some(&dispatch_time) = dispatch_times.get(window_name) {
                    if *last_changed < dispatch_time {
                        tracing::debug!(
                            "Window '{}' has stale pre-task content, resetting stuck timer",
                            window_name
                        );
                        *last_changed = now;
                        continue;
                    }
                }

                tracing::warn!(
                    "Window '{}' appears stuck (no change for {}s), auto-pulling content",
                    window_name,
                    STUCK_THRESHOLD_SECS
                );
                *last_changed = now;
                auto_return_stuck_content(&state, window_name, content).await;
            }
        }
    }
}

pub async fn auto_return_stuck_content(state: &MonitorState, window_name: &str, content: &str) {
    use hi_core::{
        db::insert_message,
        message::{Message, MessageType, TaskStatus},
    };
    use uuid::Uuid;

    let task_info = {
        let queues = state.queues.read().await;
        queues.peek_next(window_name).map(|t| (t.id, t.sender.clone()))
    };

    if let Some((task_id, task_sender)) = task_info {
        let result_content = format!("[AUTO-PULLED due to stuck detection]\n{content}");

        // Deliver result to sender's pane (same as detect_task_done / Pull handler)
        let session = state.session.read().await;
        let sender_pane_id = session
            .windows
            .iter()
            .find(|w| w.name == task_sender)
            .and_then(|w| w.tmux_pane_id.clone());
        if let Some(pane_id) = sender_pane_id {
            let envelope = format_result_envelope(&task_id, window_name, &task_sender, &result_content);
            if let Err(e) = deliver_to_pane(&pane_id, &envelope) {
                tracing::warn!("auto-pull: failed to deliver result to pane {}: {}", pane_id, e);
            }
        }

        let result_msg = Message {
            id: Uuid::new_v4(),
            sender: window_name.to_string(),
            receiver: task_sender.clone(),
            timestamp: chrono::Utc::now(),
            content: result_content,
            msg_type: MessageType::Result,
            status: TaskStatus::Timeout,
            parent_id: Some(task_id),
        };
        if let Err(e) = insert_message(&state.pool, &result_msg).await {
            tracing::error!("Failed to save auto-pulled result: {e}");
        } else {
            let mut queues = state.queues.write().await;
            queues.pop_next(window_name);
            drop(queues);
            let mut pending = state.pending_tasks.write().await;
            pending.remove(&task_id);
            tracing::info!("Auto-pulled stuck content for task {}", task_id);
        }
    }
}

async fn detect_task_done_structured(state: MonitorState) {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;

        let pending = state.pending_tasks.read().await.clone();
        if pending.is_empty() {
            continue;
        }

        let session = state.session.read().await;
        let work_dir = session.work_dir.clone();
        let baselines = state.response_baselines.read().await.clone();

        for (task_id, (sender, receiver)) in &pending {
            // 只处理已知工具，未知工具交给快照检测
            if !is_known_tool(receiver) {
                continue;
            }

            let baseline = baselines.get(receiver).cloned().unwrap_or_default();

            let current = match read_latest_response(receiver, &work_dir).await {
                None => continue,               // 结构化存储暂无数据，等待
                Some(c) if c == baseline => continue, // 无新内容
                Some(c) => c,
            };

            // 提取基线之后新增的内容
            let result = if current.len() > baseline.len() && current.starts_with(&baseline) {
                current[baseline.len()..].trim().to_string()
            } else {
                current.trim().to_string()
            };

            if result.is_empty() {
                continue;
            }

            tracing::info!(
                "Structured storage: new response detected for task {} from '{}'",
                task_id,
                receiver
            );

            // 投递结果给发送方 pane
            let sender_pane_id = session
                .windows
                .iter()
                .find(|w| w.name == *sender)
                .and_then(|w| w.tmux_pane_id.clone());

            if let Some(pane_id) = sender_pane_id {
                let envelope = format_result_envelope(task_id, receiver, sender, &result);
                if let Err(e) = deliver_to_pane(&pane_id, &envelope) {
                    tracing::warn!("structured: failed to deliver to pane {}: {}", pane_id, e);
                    continue;
                }
            }

            let result_msg = Message {
                id: Uuid::new_v4(),
                sender: receiver.clone(),
                receiver: sender.clone(),
                timestamp: chrono::Utc::now(),
                content: result.clone(),
                msg_type: MessageType::Result,
                status: TaskStatus::Completed,
                parent_id: Some(*task_id),
            };
            if let Err(e) = insert_message(&state.pool, &result_msg).await {
                tracing::error!("Failed to save structured result: {e}");
            } else {
                let mut pending_w = state.pending_tasks.write().await;
                pending_w.remove(task_id);
                drop(pending_w);
                let mut queues = state.queues.write().await;
                queues.pop_next(receiver);
                drop(queues);
                // 更新基线，为下一个任务做准备
                state.response_baselines.write().await.insert(receiver.clone(), current);
            }
        }
    }
}

async fn detect_task_done(state: MonitorState) {
    let mut interval = time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));
    let mut cleanup_tick: u32 = 0;

    loop {
        interval.tick().await;

        cleanup_tick += 1;
        if cleanup_tick >= 150 {
            cleanup_tick = 0;
            let mut cooldowns = state.pull_cooldown.write().await;
            cooldowns.retain(|_, t| t.elapsed().as_secs() < PULL_COOLDOWN_SECS);
        }

        let pending = state.pending_tasks.read().await.clone();
        if pending.is_empty() {
            continue;
        }

        let snapshots = state.snapshots.read().await.clone();
        let session = state.session.read().await;

        for (task_id, (sender, receiver)) in &pending {
            // 已知工具由结构化存储检测，快照检测不介入
            if is_known_tool(receiver) {
                continue;
            }

            let snapshot = snapshots.get(receiver).cloned().unwrap_or_default();

            if let Some(result_content) = extract_result(&snapshot, task_id) {
                tracing::info!(
                    "Detected DONE marker for task {} from '{}'",
                    task_id,
                    receiver
                );

                let sender_pane_id = session
                    .windows
                    .iter()
                    .find(|w| w.name == *sender)
                    .and_then(|w| w.tmux_pane_id.clone());

                if let Some(pane_id) = sender_pane_id {
                    let envelope = format_result_envelope(task_id, receiver, sender, &result_content);
                    if let Err(e) = deliver_to_pane(&pane_id, &envelope) {
                        tracing::warn!(
                            "Failed to deliver result to tmux pane {}: {}",
                            pane_id,
                            e
                        );
                    }
                }

                let result_msg = Message {
                    id: Uuid::new_v4(),
                    sender: receiver.clone(),
                    receiver: sender.clone(),
                    timestamp: chrono::Utc::now(),
                    content: result_content,
                    msg_type: MessageType::Result,
                    status: TaskStatus::Completed,
                    parent_id: Some(*task_id),
                };
                if let Err(e) = insert_message(&state.pool, &result_msg).await {
                    tracing::error!("Failed to save result message: {e}");
                }

                let mut pending = state.pending_tasks.write().await;
                pending.remove(task_id);

                let mut queues = state.queues.write().await;
                queues.pop_next(receiver);
            }
        }
    }
}
