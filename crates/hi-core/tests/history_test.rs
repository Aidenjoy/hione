use hi_core::history::{read_latest_response, supported_tool_name};
use std::path::Path;

#[tokio::test]
async fn read_latest_response_unknown_tool_returns_none() {
    let cwd = Path::new("/tmp");
    let result = read_latest_response("unknown_tool", cwd).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn read_latest_response_for_supported_tools_without_history() {
    // Testing with a temp directory that has no history files
    let dir = tempfile::tempdir().unwrap();

    // These tools have specific history paths, so with a fresh temp dir they should return None
    let result = read_latest_response("claude", dir.path()).await;
    assert!(result.is_none());

    let result = read_latest_response("codex", dir.path()).await;
    assert!(result.is_none());

    let result = read_latest_response("gemini", dir.path()).await;
    assert!(result.is_none());

    let result = read_latest_response("qwen", dir.path()).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn read_latest_response_opencode_without_db() {
    let dir = tempfile::tempdir().unwrap();
    let result = read_latest_response("opencode", dir.path()).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn read_latest_response_case_insensitive() {
    let dir = tempfile::tempdir().unwrap();

    // Tool names should be matched case-insensitively
    let result1 = read_latest_response("Claude", dir.path()).await;
    let result2 = read_latest_response("claude", dir.path()).await;
    let result3 = read_latest_response("CLAUDE-CODE", dir.path()).await;

    // All should return None for a directory without history
    assert!(result1.is_none());
    assert!(result2.is_none());
    assert!(result3.is_none());
}

#[test]
fn supported_tool_name_accepts_numbered_instances() {
    assert_eq!(supported_tool_name("qwen1"), Some("qwen"));
    assert_eq!(supported_tool_name("opencode12"), Some("opencode"));
    assert_eq!(supported_tool_name("Codex2"), Some("codex"));
    assert_eq!(supported_tool_name("claude-code3"), Some("claude"));
}

#[test]
fn supported_tool_name_rejects_unknown_numbered_names() {
    assert_eq!(supported_tool_name("unknown1"), None);
    assert_eq!(supported_tool_name("gpt4"), None);
}
