// Tests for start command - tests that don't require session state
// Note: build_launch_command_with_hione tests are kept inline in start.rs
// as they test a private function

use hi_core::session::{SessionInfo, WindowInfo};
use std::path::PathBuf;

#[test]
fn window_info_launch_command_preserved() {
    let w = WindowInfo {
        index: 1,
        name: "claude".to_string(),
        command: "claude".to_string(),
        launch_command: "claude --dangerously-skip-permissions --continue".to_string(),
        auto_mode: true,
        resume_mode: true,
        is_main: true,
        pid: None,
        tmux_pane_id: Some("%0".to_string()),
    };

    let json = serde_json::to_string(&w).unwrap();
    let back: WindowInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.launch_command,
        "claude --dangerously-skip-permissions --continue"
    );
    assert!(back.auto_mode);
    assert!(back.resume_mode);
}

#[test]
fn session_with_multiple_windows_serialization() {
    let session = SessionInfo {
        id: "multi-test".to_string(),
        windows: vec![
            WindowInfo {
                index: 1,
                name: "claude".to_string(),
                command: "claude".to_string(),
                launch_command: "claude".to_string(),
                auto_mode: false,
                resume_mode: false,
                is_main: true,
                pid: Some(100),
                tmux_pane_id: Some("%1".to_string()),
            },
            WindowInfo {
                index: 2,
                name: "opencode".to_string(),
                command: "opencode".to_string(),
                launch_command: "opencode".to_string(),
                auto_mode: false,
                resume_mode: false,
                is_main: false,
                pid: Some(101),
                tmux_pane_id: Some("%2".to_string()),
            },
            WindowInfo {
                index: 3,
                name: "gemini".to_string(),
                command: "gemini".to_string(),
                launch_command: "gemini".to_string(),
                auto_mode: true,
                resume_mode: false,
                is_main: false,
                pid: Some(102),
                tmux_pane_id: Some("%3".to_string()),
            },
        ],
        work_dir: PathBuf::from("/project"),
        hione_dir: PathBuf::from("/project/.hione"),
        socket_path: "/project/.hione/hi.sock".to_string(),
        monitor_pid: Some(999),
        tmux_session_name: Some("hi_proj".to_string()),
    };

    let json = serde_json::to_string(&session).unwrap();
    let back: SessionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.windows.len(), 3);
    assert!(back.windows.iter().any(|w| w.auto_mode));
}