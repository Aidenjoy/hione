use hi_cli::commands::load_session_from;
use hi_core::session::{SessionInfo, WindowInfo};

#[test]
fn load_session_from_reads_and_parses() {
    let dir = tempfile::tempdir().unwrap();
    let session = SessionInfo {
        id: "x".into(),
        windows: vec![WindowInfo {
            index: 1,
            name: "claude".into(),
            command: "claude".into(),
            launch_command: "claude".into(),
            auto_mode: false,
            resume_mode: false,
            is_main: true,
            pid: None,
            tmux_pane_id: None,
        }],
        work_dir: dir.path().to_path_buf(),
        hione_dir: dir.path().to_path_buf(),
        socket_path: "/tmp/whatever.sock".into(),
        monitor_pid: None,
        tmux_session_name: None,
    };
    std::fs::write(
        dir.path().join("session.json"),
        serde_json::to_string(&session).unwrap(),
    )
    .unwrap();

    let got = load_session_from(dir.path()).unwrap();
    assert_eq!(got.id, "x");
    assert_eq!(got.windows[0].name, "claude");
}

#[test]
fn load_session_from_missing_file_errors() {
    let dir = tempfile::tempdir().unwrap();
    let err = load_session_from(dir.path()).unwrap_err();
    assert!(format!("{err}").contains("session.json"));
}

#[test]
fn load_session_from_bad_json_errors() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("session.json"), "{not json").unwrap();
    let err = load_session_from(dir.path()).unwrap_err();
    assert!(!format!("{err}").is_empty());
}
