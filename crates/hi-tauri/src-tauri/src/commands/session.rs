use crate::types::RecentSession;
use crate::AppState;
use crate::services::session;
use hi_core::session::SessionInfo;
use tauri::{State, Window};

#[tauri::command]
pub async fn list_recent_sessions(state: State<'_, AppState>) -> Result<Vec<RecentSession>, String> {
    session::get_recent_sessions(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn launch_session(
    state: State<'_, AppState>,
    work_dir: String,
    tools: Vec<String>,
    auto: bool,
    resume: bool,
    window: Window,
) -> Result<(), String> {
    session::launch_session(&work_dir, &tools, auto, resume, window)
        .await
        .map_err(|e| e.to_string())?;
    
    session::update_recent_session(&state.db, &work_dir, &tools, auto, resume)
        .await
        .map_err(|e| e.to_string())?;
    
    {
        let mut current = state.current_work_dir.lock().unwrap();
        *current = Some(work_dir.clone());
    }
    
    Ok(())
}

#[tauri::command]
pub async fn connect_session(
    state: State<'_, AppState>,
    work_dir: String,
) -> Result<SessionInfo, String> {
    let session = session::read_session_info(&work_dir)
        .map_err(|e| e.to_string())?;
    
    match session {
        Some(s) => {
            {
                let mut current = state.current_work_dir.lock().unwrap();
                *current = Some(work_dir.clone());
            }
            Ok(s)
        }
        None => Err(format!("No session found in {}", work_dir)),
    }
}

#[tauri::command]
pub async fn disconnect_session(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut current = state.current_work_dir.lock().unwrap();
        *current = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn detect_session(work_dir: String) -> Result<Option<SessionInfo>, String> {
    session::detect_running_session(&work_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn kill_session(
    state: State<'_, AppState>,
    work_dir: String,
) -> Result<(), String> {
    use std::process::Command;
    use std::path::PathBuf;

    /// Get the multiplexer binary name for the current platform
    fn mux_bin() -> &'static str {
        if cfg!(windows) { "psmux" } else { "tmux" }
    }

    let session_path = PathBuf::from(&work_dir).join(".hione").join("session.json");
    let _tmux_killed = false;

    if session_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&session_path) {
            if let Ok(session) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut session_name: Option<String> = session
                    .get("tmux_session_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if session_name.is_none() {
                    if let Some(windows) = session.get("windows").and_then(|v| v.as_array()) {
                        for w in windows {
                            if let Some(pane_id) = w.get("tmux_pane_id").and_then(|v| v.as_str()) {
                                if let Ok(out) = Command::new(mux_bin())
                                    .args(["display-message", "-p", "-t", pane_id, "#{session_name}"])
                                    .output()
                                {
                                    if out.status.success() {
                                        let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                        if !name.is_empty() {
                                            session_name = Some(name);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(ref name) = session_name {
                    let _ = Command::new(mux_bin())
                        .args(["kill-session", "-t", name])
                        .status();

                    #[cfg(target_os = "macos")]
                    {
                        let close_script = format!(
                            r#"
                            try
                                tell application "iTerm2"
                                    repeat with aWindow in windows
                                        repeat with aTab in tabs of aWindow
                                            repeat with aSession in sessions of aTab
                                                set sessionTTY to tty of aSession
                                                try
                                                    set sessionName to do shell script "/usr/local/bin/tmux display-message -p -t " & quoted form of sessionTTY & " '#{{session_name}}' 2>/dev/null || /opt/homebrew/bin/tmux display-message -p -t " & quoted form of sessionTTY & " '#{{session_name}}' 2>/dev/null"
                                                    if sessionName is "{}" then
                                                        close aTab
                                                        return
                                                    end if
                                                end try
                                            end repeat
                                        end repeat
                                    end repeat
                                end tell
                            end try
                            try
                                tell application "Terminal"
                                    repeat with w in windows
                                        set tabCount to count of tabs of w
                                        repeat with i from tabCount to 1 by -1
                                            set t to tab i of w
                                            try
                                                set ttyContents to contents of t
                                                if ttyContents contains "tmux" and ttyContents contains "{}" then
                                                    close t
                                                end if
                                            end try
                                        end repeat
                                    end repeat
                                end tell
                            end try
                            "#,
                            name, name
                        );
                        let _ = Command::new("osascript")
                            .arg("-e")
                            .arg(&close_script)
                            .status();
                    }
                }

                if let Some(pid) = session.get("monitor_pid").and_then(|v| v.as_u64()) {
                    #[cfg(unix)]
                    let _ = Command::new("kill").arg(pid.to_string()).status();
                    #[cfg(windows)]
                    let _ = Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .status();
                }

                #[cfg(unix)]
                if let Some(socket) = session.get("socket_path").and_then(|v| v.as_str()) {
                    let _ = std::fs::remove_file(socket);
                }

                let _ = std::fs::remove_file(&session_path);
            }
        }
    }

    {
        let mut current = state.current_work_dir.lock().unwrap();
        if current.as_deref() == Some(&work_dir) {
            *current = None;
        }
    }

    Ok(())
}