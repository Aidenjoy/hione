use hi_tauri_lib::types::{SetupStatus, ToolInfo, Agent, McpServer, Skill, SkillRepo, RecentSession, TaskRecord, CustomTool, AppSettings};

#[test]
fn setup_status_default() {
    let s = SetupStatus::default();
    assert!(!s.tmux);
    assert!(!s.node);
    assert!(!s.rust);
    assert!(!s.hi);
    assert!(!s.hi_monitor);
}

#[test]
fn tool_info_default() {
    let t = ToolInfo::default();
    assert!(t.name.is_empty());
    assert!(!t.installed);
    assert!(t.version.is_none());
}

#[test]
fn agent_default_has_uuid() {
    let a = Agent::default();
    assert!(!a.id.is_empty());
    assert!(a.name.is_empty());
    assert!(a.api_key.is_none());
    assert!(a.enabled);
}

#[test]
fn agent_serde_roundtrip() {
    let a = Agent {
        id: "test-id".to_string(),
        name: "claude".to_string(),
        api_key: Some("sk-xxx".to_string()),
        api_base_url: Some("https://api.example.com".to_string()),
        model: Some("claude-opus".to_string()),
        extra_config: serde_json::json!({"max_tokens": 4096}),
        enabled: true,
        created_at: 1234567890,
        updated_at: 1234567890,
    };

    let json = serde_json::to_string(&a).unwrap();
    let back: Agent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "test-id");
    assert_eq!(back.name, "claude");
    assert_eq!(back.api_key, Some("sk-xxx".to_string()));
    assert_eq!(back.model, Some("claude-opus".to_string()));
}

#[test]
fn mcp_server_default() {
    let m = McpServer::default();
    assert!(!m.id.is_empty());
    assert!(m.name.is_empty());
    assert!(m.enabled_for.is_empty());
}

#[test]
fn skill_default() {
    let s = Skill::default();
    assert!(!s.id.is_empty());
    assert!(s.name.is_empty());
    assert!(s.repo_url.is_none());
    assert!(s.local_path.is_none());
    assert!(s.enabled_for.is_empty());
}

#[test]
fn skill_repo_default() {
    let sr = SkillRepo::default();
    assert!(!sr.id.is_empty());
    assert!(sr.url.is_empty());
    assert!(sr.name.is_empty());
}

#[test]
fn recent_session_default() {
    let rs = RecentSession::default();
    assert!(rs.work_dir.is_empty());
    assert!(rs.tools.is_empty());
    assert!(!rs.auto_mode);
    assert!(!rs.resume_mode);
}

#[test]
fn recent_session_serde_roundtrip() {
    let rs = RecentSession {
        work_dir: "/project/path".to_string(),
        tools: vec!["claude".to_string(), "opencode".to_string()],
        auto_mode: true,
        resume_mode: false,
        last_used: 1234567890,
    };

    let json = serde_json::to_string(&rs).unwrap();
    let back: RecentSession = serde_json::from_str(&json).unwrap();
    assert_eq!(back.work_dir, "/project/path");
    assert_eq!(back.tools.len(), 2);
    assert!(back.auto_mode);
}

#[test]
fn task_record_default() {
    let tr = TaskRecord::default();
    assert!(!tr.id.is_empty());
    assert!(tr.sender.is_empty());
    assert!(tr.receiver.is_empty());
    assert_eq!(tr.status, "pending");
}

#[test]
fn custom_tool_default() {
    let ct = CustomTool::default();
    assert!(ct.name.is_empty());
    assert!(ct.auto_flags.is_empty());
    assert!(ct.resume_flags.is_empty());
}

#[test]
fn app_settings_default() {
    let s = AppSettings::default();
    assert_eq!(s.language, "en");
    assert_eq!(s.theme, "system");
    assert!(s.hi_bin_path.is_none());
    assert!(s.hi_monitor_bin_path.is_none());
}

#[test]
fn app_settings_serde_roundtrip() {
    let s = AppSettings {
        language: "zh".to_string(),
        theme: "dark".to_string(),
        hi_bin_path: Some("/usr/local/bin/hi".to_string()),
        hi_monitor_bin_path: Some("/usr/local/bin/hi-monitor".to_string()),
    };

    let json = serde_json::to_string(&s).unwrap();
    let back: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.language, "zh");
    assert_eq!(back.theme, "dark");
    assert_eq!(back.hi_bin_path, Some("/usr/local/bin/hi".to_string()));
}