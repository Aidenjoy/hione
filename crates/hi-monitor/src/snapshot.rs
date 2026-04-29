use crate::server::MonitorState;
use hi_core::{
    history::read_latest_response,
    ipc::{recv_message, send_message},
    message::{Message, MessageType, TaskStatus},
};
use std::{path::Path, process::Command, time::Duration};
use tokio::time;
use uuid::Uuid;

/// Get the multiplexer binary name for the current platform
fn mux_bin() -> &'static str {
    if cfg!(windows) { "psmux" } else { "tmux" }
}

pub async fn poll_snapshots(state: MonitorState) {
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        let session = state.session.read().await;
        let windows = session.windows.clone();
        let cwd = session.work_dir.clone();
        for window in &windows {
            let req = Message {
                id: Uuid::new_v4(),
                sender: "monitor".to_string(),
                receiver: window.name.clone(),
                timestamp: chrono::Utc::now(),
                content: String::new(),
                msg_type: MessageType::Snapshot,
                status: TaskStatus::Pending,
                parent_id: None,
            };
            if let Err(e) = request_snapshot(&state, &req, &window.tmux_pane_id, &window.name, &cwd).await {
                tracing::warn!(
                    "Snapshot request failed for '{}': {e}",
                    window.name
                );
            }
        }
    }
}

pub async fn request_snapshot(
    state: &MonitorState,
    req: &Message,
    tmux_pane_id: &Option<String>,
    window_name: &str,
    cwd: &Path,
) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let snap_socket = state.hione_dir.join("snap.sock");
        let stream_result = UnixStream::connect(&snap_socket).await;
        let mut stream = match stream_result {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound
                    || e.kind() == std::io::ErrorKind::ConnectionRefused => {
                if let Some(content) = read_latest_response(window_name, cwd).await {
                    let mut snapshots = state.snapshots.write().await;
                    snapshots.insert(req.receiver.clone(), content);
                    return Ok(())
                }

                if let Some(pane_id) = tmux_pane_id {
                    let pane_id = pane_id.clone();
                    let mux = mux_bin();
                    let content = tokio::task::spawn_blocking(move || {
                        Command::new(mux)
                            .args(["capture-pane", "-p", "-t", &pane_id, "-S", "-500"])
                            .output()
                            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                    })
                    .await??;
                    let mut snapshots = state.snapshots.write().await;
                    snapshots.insert(req.receiver.clone(), content);
                }
                return Ok(())
            }
            Err(e) => return Err(e.into()),
        };
        send_message(&mut stream, req).await?;
        let resp = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            recv_message(&mut stream),
        )
        .await??;

        if resp.msg_type == MessageType::SnapshotData {
            let mut snapshots = state.snapshots.write().await;
            snapshots.insert(req.receiver.clone(), resp.content);
        }
    }

    #[cfg(windows)]
    {
        // Windows: use psmux capture-pane or read history
        if let Some(content) = read_latest_response(window_name, cwd).await {
            let mut snapshots = state.snapshots.write().await;
            snapshots.insert(req.receiver.clone(), content);
            return Ok(())
        }

        if let Some(pane_id) = tmux_pane_id {
            let pane_id = pane_id.clone();
            let content = tokio::task::spawn_blocking(move || {
                Command::new(mux_bin())
                    .args(["capture-pane", "-p", "-t", &pane_id, "-S", "-500"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            })
            .await??;
            let mut snapshots = state.snapshots.write().await;
            snapshots.insert(req.receiver.clone(), content);
        }
    }
    Ok(())
}
