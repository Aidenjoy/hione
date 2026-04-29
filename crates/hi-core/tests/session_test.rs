use hi_core::session::{SessionInfo, WindowInfo};
use std::path::PathBuf;

#[cfg(unix)]
#[test]
fn socket_path_joins_hione_dir() {
    let p = SessionInfo::socket_path_for(Path::new("/tmp/xx/.hione"));
    assert!(p.ends_with("/.hione/hi.sock"), "got: {p}");
}

#[test]
fn session_serde_roundtrip() {
    let s = SessionInfo {
        id: "abc".to_string(),
        windows: vec![WindowInfo {
            index: 1,
            name: "claude".to_string(),
            command: "claude".to_string(),
            launch_command: "claude".to_string(),
            auto_mode: false,
            resume_mode: false,
            is_main: true,
            pid: Some(42),
            tmux_pane_id: None,
        }],
        work_dir: PathBuf::from("/tmp/work"),
        hione_dir: PathBuf::from("/tmp/work/.hione"),
        socket_path: "/tmp/work/.hione/hi.sock".to_string(),
        monitor_pid: Some(100),
        tmux_session_name: None,
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: SessionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "abc");
    assert_eq!(back.windows.len(), 1);
    assert_eq!(back.windows[0].name, "claude");
    assert!(back.windows[0].is_main);
    assert_eq!(back.monitor_pid, Some(100));
}

#[test]
fn session_load_from_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let session = SessionInfo {
        id: "test-id".to_string(),
        windows: vec![WindowInfo {
            index: 1,
            name: "gemini".to_string(),
            command: "gemini".to_string(),
            launch_command: "gemini".to_string(),
            auto_mode: true,
            resume_mode: false,
            is_main: true,
            pid: None,
            tmux_pane_id: Some("%5".to_string()),
        }],
        work_dir: dir.path().to_path_buf(),
        hione_dir: dir.path().join(".hione"),
        socket_path: SessionInfo::socket_path_for(&dir.path().join(".hione")),
        monitor_pid: Some(1234),
        tmux_session_name: Some("hi_session".to_string()),
    };

    std::fs::create_dir_all(dir.path().join(".hione")).unwrap();
    std::fs::write(
        dir.path().join(".hione/session.json"),
        serde_json::to_string(&session).unwrap(),
    )
    .unwrap();

    let loaded = SessionInfo::load_from(&dir.path().join(".hione")).unwrap();
    assert_eq!(loaded.id, "test-id");
    assert_eq!(loaded.windows[0].name, "gemini");
    assert_eq!(loaded.windows[0].tmux_pane_id, Some("%5".to_string()));
    assert_eq!(loaded.tmux_session_name, Some("hi_session".to_string()));
}

#[test]
fn session_load_from_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let result = SessionInfo::load_from(&dir.path().join(".hione"));
    assert!(result.is_none());
}

#[test]
fn session_load_from_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".hione")).unwrap();
    std::fs::write(dir.path().join(".hione/session.json"), "{invalid}").unwrap();
    let result = SessionInfo::load_from(&dir.path().join(".hione"));
    assert!(result.is_none());
}

#[test]
fn window_info_with_optional_fields() {
    let w = WindowInfo {
        index: 2,
        name: "opencode".to_string(),
        command: "opencode".to_string(),
        launch_command: "opencode --continue".to_string(),
        auto_mode: false,
        resume_mode: true,
        is_main: false,
        pid: Some(999),
        tmux_pane_id: Some("%10".to_string()),
    };

    let json = serde_json::to_string(&w).unwrap();
    let back: WindowInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tmux_pane_id, Some("%10".to_string()));
    assert_eq!(back.resume_mode, true);
}

#[test]
fn window_info_skip_none_tmux_pane_id() {
    let w = WindowInfo {
        index: 1,
        name: "claude".to_string(),
        command: "claude".to_string(),
        launch_command: "claude".to_string(),
        auto_mode: false,
        resume_mode: false,
        is_main: true,
        pid: None,
        tmux_pane_id: None,
    };

    let json = serde_json::to_string(&w).unwrap();
    // tmux_pane_id should not appear in JSON when None (skip_serializing_if)
    assert!(!json.contains("tmux_pane_id"));
}
