use anyhow::Result;
use hi_core::{message::Message, session::SessionInfo};
use std::env;

use super::common::{load_session, send_to_monitor};

pub async fn run(target: String, content: String) -> Result<()> {
    let session = load_session()?;
    let current_pane_id = env::var("TMUX_PANE").ok();
    let sender = detect_sender(&session, current_pane_id.as_deref());
    let target = resolve_target(&session, &target)?;

    let msg = Message::new_task(&sender, &target, &content);
    println!("Task ID: {}", msg.id);
    send_to_monitor(&session.socket_path, &msg).await?;
    println!(
        "Task dispatched from '{}' to '{}': {}",
        sender, target, msg.id
    );
    Ok(())
}

pub fn detect_sender(session: &SessionInfo, current_pane_id: Option<&str>) -> String {
    if let Some(pane_id) = current_pane_id {
        for window in &session.windows {
            if let Some(window_pane_id) = &window.tmux_pane_id {
                if window_pane_id == pane_id {
                    return window.name.clone();
                }
            }
        }
    }
    "user".to_string()
}

pub fn resolve_target(session: &SessionInfo, target: &str) -> Result<String> {
    if session.windows.iter().any(|w| w.name == target) {
        return Ok(target.to_string());
    }

    let matches = session
        .windows
        .iter()
        .filter(|w| w.name.starts_with(target))
        .map(|w| w.name.as_str())
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [single] => Ok((*single).to_string()),
        [] => {
            let available = session
                .windows
                .iter()
                .map(|w| w.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("Unknown target '{target}'. Available targets: {available}");
        }
        _ => {
            anyhow::bail!(
                "Ambiguous target '{target}'. Matches: {}",
                matches.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hi_core::session::WindowInfo;
    use std::path::PathBuf;

    #[test]
    fn test_detect_sender() {
        let session = SessionInfo {
            id: "test".to_string(),
            windows: vec![
                WindowInfo {
                    index: 1,
                    name: "opencode".to_string(),
                    command: "opencode".to_string(),
                    launch_command: "opencode".to_string(),
                    auto_mode: false,
                    resume_mode: false,
                    is_main: true,
                    pid: None,
                    tmux_pane_id: Some("%1".to_string()),
                },
                WindowInfo {
                    index: 2,
                    name: "qwen".to_string(),
                    command: "qwen".to_string(),
                    launch_command: "qwen".to_string(),
                    auto_mode: false,
                    resume_mode: false,
                    is_main: false,
                    pid: None,
                    tmux_pane_id: Some("%2".to_string()),
                },
            ],
            work_dir: PathBuf::from("/"),
            hione_dir: PathBuf::from("/"),
            socket_path: "/tmp/test.sock".to_string(),
            monitor_pid: None,
            tmux_session_name: None,
        };

        // In pane %1
        assert_eq!(detect_sender(&session, Some("%1")), "opencode");
        // In pane %2
        assert_eq!(detect_sender(&session, Some("%2")), "qwen");
        // In unknown pane
        assert_eq!(detect_sender(&session, Some("%3")), "user");
        // Not in tmux
        assert_eq!(detect_sender(&session, None), "user");
    }

    #[test]
    fn test_detect_sender_empty_session() {
        let session = SessionInfo {
            id: "test".to_string(),
            windows: vec![],
            work_dir: PathBuf::from("/"),
            hione_dir: PathBuf::from("/"),
            socket_path: "/tmp/test.sock".to_string(),
            monitor_pid: None,
            tmux_session_name: None,
        };
        assert_eq!(detect_sender(&session, Some("%1")), "user");
    }

    fn session_with_windows(names: &[&str]) -> SessionInfo {
        SessionInfo {
            id: "test".to_string(),
            windows: names
                .iter()
                .enumerate()
                .map(|(i, name)| WindowInfo {
                    index: i + 1,
                    name: (*name).to_string(),
                    command: (*name).to_string(),
                    launch_command: (*name).to_string(),
                    auto_mode: false,
                    resume_mode: false,
                    is_main: i == 0,
                    pid: None,
                    tmux_pane_id: Some(format!("%{}", i + 1)),
                })
                .collect(),
            work_dir: PathBuf::from("/"),
            hione_dir: PathBuf::from("/"),
            socket_path: "/tmp/test.sock".to_string(),
            monitor_pid: None,
            tmux_session_name: None,
        }
    }

    #[test]
    fn resolve_target_prefers_exact_match() {
        let session = session_with_windows(&["qwen", "qwen1"]);
        assert_eq!(resolve_target(&session, "qwen").unwrap(), "qwen");
    }

    #[test]
    fn resolve_target_accepts_unique_prefix() {
        let session = session_with_windows(&["opencode1", "qwen1"]);
        assert_eq!(resolve_target(&session, "qwen").unwrap(), "qwen1");
    }

    #[test]
    fn resolve_target_rejects_unknown_or_ambiguous_targets() {
        let session = session_with_windows(&["qwen1", "qwen2"]);
        assert!(resolve_target(&session, "gemini")
            .unwrap_err()
            .to_string()
            .contains("Unknown target"));
        assert!(resolve_target(&session, "qwen")
            .unwrap_err()
            .to_string()
            .contains("Ambiguous target"));
    }
}
