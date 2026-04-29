use crate::error::AppError;
use crate::types::{RecentSession, TaskRecord};
use hi_core::session::SessionInfo;
use tauri::{Window, Emitter};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use sqlx::Row;

pub async fn launch_session(
    work_dir: &str,
    tools: &[String],
    auto: bool,
    resume: bool,
    window: Window,
) -> Result<(), AppError> {
    if !std::path::Path::new(work_dir).exists() {
        return Err(AppError::CommandFailed(format!("目录不存在: {}", work_dir)));
    }

    let session_name = format!(
        "hi_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    let mut hi_args = vec!["start".to_string()];
    // On Windows, always force terminal mode (-T) so hi start uses psmux instead
    // of trying to launch another desktop Tauri instance.
    #[cfg(windows)]
    hi_args.push("-T".to_string());
    if auto {
        hi_args.push("-a".to_string());
    }
    if resume {
        hi_args.push("-r".to_string());
    }
    hi_args.push(tools.join(","));
    let hi_cmd = format!("hi {}", hi_args.join(" "));

    #[cfg(windows)]
    let shell_cmd = format!(
        "cd /d \"{}\" & psmux new-session -s {} -- cmd /K \"{}\"",
        work_dir, session_name, hi_cmd
    );

    #[cfg(not(windows))]
    let shell_cmd = format!(
        "cd '{}' && tmux new-session -s '{}' '{}'",
        work_dir.replace('\'', "'\\''"),
        session_name,
        hi_cmd.replace('\'', "'\\''")
    );

    #[cfg(target_os = "macos")]
    {
        let iterm_check = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to (name of processes) contains \"iTerm2\"")
            .output();

        let use_iterm =
            matches!(iterm_check, Ok(out) if String::from_utf8_lossy(&out.stdout).trim() == "true");

        let applescript = if use_iterm {
            format!(
                r#"tell application "iTerm2"
                    activate
                    tell current window
                        create tab with default profile
                        tell current session of current tab
                            write text "{}"
                        end tell
                    end tell
                end tell"#,
                shell_cmd.replace('"', "\\\"")
            )
        } else {
            format!(
                r#"tell application "Terminal"
                    activate
                    do script "{}"
                end tell"#,
                shell_cmd.replace('"', "\\\"")
            )
        };

        Command::new("osascript")
            .arg("-e")
            .arg(&applescript)
            .spawn()
            .map_err(|e| AppError::CommandFailed(format!("无法打开终端: {}", e)))?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows Terminal 如果可用则使用，否则使用 cmd
        let wt_available = which::which("wt").is_ok();
        if wt_available {
            // wt 需要完整的命令行参数
            Command::new("wt")
                .args(["new-tab", "cmd", "/K", &shell_cmd])
                .spawn()
                .map_err(|e| AppError::CommandFailed(format!("无法打开终端: {}", e)))?;
        } else {
            Command::new("cmd")
                .args(["/C", "start", "cmd", "/K", &shell_cmd])
                .spawn()
                .map_err(|e| AppError::CommandFailed(format!("无法打开终端: {}", e)))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let terminals = ["gnome-terminal", "xterm", "konsole", "alacritty"];
        let mut launched = false;
        for term in &terminals {
            if which::which(term).is_ok() {
                let args: Vec<&str> = match *term {
                    "gnome-terminal" => vec!["--", "bash", "-c", &shell_cmd],
                    "konsole" => vec!["-e", "bash", "-c", &shell_cmd],
                    _ => vec!["-e", &shell_cmd],
                };
                if Command::new(term).args(&args).spawn().is_ok() {
                    launched = true;
                    break;
                }
            }
        }
        if !launched {
            return Err(AppError::CommandFailed(
                "未找到可用的终端模拟器".to_string(),
            ));
        }
    }

    // 异步等待 hi-monitor 启动后发 connected: true
    let window_clone = window.clone();
    let work_dir_owned = work_dir.to_string();

    #[cfg(windows)]
    let sock_check = async move {
        // Windows: detect monitor by probing the named pipe.
        // Read the pipe name from session.json (written by `hi start`) rather than
        // recomputing it — the hash in socket_path_for is deterministic but reading
        // from the file is always authoritative.
        use interprocess::local_socket::tokio::prelude::LocalSocketStream;
        use interprocess::local_socket::traits::tokio::Stream;
        use interprocess::local_socket::{ToNsName, GenericNamespaced};

        let session_json_path = PathBuf::from(&work_dir_owned).join(".hione").join("session.json");

        for _ in 0..30 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Read socket_path from session.json written by `hi start`
            let pipe_name_opt = std::fs::read_to_string(&session_json_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| {
                    v.get("socket_path")
                        .and_then(|p| p.as_str())
                        .map(|s| s.split('\\').last().unwrap_or(s).to_string())
                });

            if let Some(pipe_name) = pipe_name_opt {
                if let Ok(ns_name) = pipe_name.to_ns_name::<GenericNamespaced>() {
                    if LocalSocketStream::connect(ns_name).await.is_ok() {
                        let _ = window_clone.emit(
                            "session://status",
                            json!({ "connected": true, "work_dir": work_dir_owned }),
                        );
                        return;
                    }
                }
            }
        }
        let _ = window_clone.emit(
            "session://status",
            json!({ "connected": false, "work_dir": work_dir_owned, "error": "hi-monitor 未在 30 秒内启动" }),
        );
    };

    #[cfg(not(windows))]
    let sock_check = async move {
        let sock_path = PathBuf::from(&work_dir_owned).join(".hione").join("hi.sock");
        for _ in 0..30 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if sock_path.exists() {
                let _ = window_clone.emit(
                    "session://status",
                    json!({ "connected": true, "work_dir": work_dir_owned }),
                );
                return;
            }
        }
        let _ = window_clone.emit(
            "session://status",
            json!({ "connected": false, "work_dir": work_dir_owned, "error": "hi-monitor 未在 30 秒内启动" }),
        );
    };

    tokio::spawn(sock_check);

    Ok(())
}

pub fn read_session_info(work_dir: &str) -> Result<Option<SessionInfo>, AppError> {
    let session_path = PathBuf::from(work_dir).join(".hione").join("session.json");
    
    if !session_path.exists() {
        return Ok(None);
    }
    
    let content = std::fs::read_to_string(&session_path)?;
    let session: SessionInfo = serde_json::from_str(&content)?;
    
    Ok(Some(session))
}

pub fn detect_running_session(work_dir: &str) -> Result<Option<SessionInfo>, AppError> {
    let session = read_session_info(work_dir)?;

    if session.is_none() {
        return Ok(None);
    }

    let session = session.unwrap();

    #[cfg(unix)]
    {
        let socket_path = PathBuf::from(work_dir).join(".hione").join("hi.sock");
        if socket_path.exists() {
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    #[cfg(windows)]
    {
        // Windows 使用命名管道，无法通过文件检测
        // 通过检查 monitor_pid 进程是否存在来判断
        if let Some(pid) = session.monitor_pid {
            // 检查进程是否存在
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // tasklist 返回包含 PID 的行表示进程存在
                if stdout.contains(&pid.to_string()) {
                    return Ok(Some(session));
                }
            }
        }
        Ok(None)
    }
}

pub async fn update_recent_session(
    pool: &sqlx::SqlitePool,
    work_dir: &str,
    tools: &[String],
    auto: bool,
    resume: bool,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    let tools_json = serde_json::to_string(tools)?;
    
    sqlx::query(
        "INSERT INTO recent_sessions (work_dir, tools, auto_mode, resume_mode, last_used)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(work_dir) DO UPDATE SET
           tools = excluded.tools,
           auto_mode = excluded.auto_mode,
           resume_mode = excluded.resume_mode,
           last_used = excluded.last_used"
    )
    .bind(work_dir)
    .bind(&tools_json)
    .bind(auto)
    .bind(resume)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_recent_sessions(pool: &sqlx::SqlitePool) -> Result<Vec<RecentSession>, AppError> {
    let rows = sqlx::query(
        "SELECT work_dir, tools, auto_mode, resume_mode, last_used FROM recent_sessions ORDER BY last_used DESC LIMIT 20"
    )
    .fetch_all(pool)
    .await?;
    
    let sessions: Vec<RecentSession> = rows
        .into_iter()
        .map(|row| {
            let tools_json: String = row.try_get::<String, _>("tools").unwrap_or_else(|_| "[]".to_string());
            let tools: Vec<String> = serde_json::from_str(&tools_json).unwrap_or_default();
            
            RecentSession {
                work_dir: row.try_get::<String, _>("work_dir").unwrap_or_default(),
                tools,
                auto_mode: row.try_get::<bool, _>("auto_mode").unwrap_or(false),
                resume_mode: row.try_get::<bool, _>("resume_mode").unwrap_or(false),
                last_used: row.try_get::<i64, _>("last_used").unwrap_or_default(),
            }
        })
        .collect();
    
    Ok(sessions)
}

pub async fn get_task_records(work_dir: &str) -> Result<Vec<TaskRecord>, AppError> {
    let db_path = PathBuf::from(work_dir).join(".hione").join("hi.db");

    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let db_url = format!("sqlite://{}?mode=ro", db_path.display());
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let rows = sqlx::query(
        "SELECT id, sender, receiver, content, status, timestamp
         FROM messages
         WHERE msg_type = '\"task\"'
         ORDER BY timestamp DESC
         LIMIT 100",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    pool.close().await;

    fn capitalize_first(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().to_string() + c.as_str(),
        }
    }

    let records = rows
        .into_iter()
        .filter_map(|row| {
            use sqlx::Row;
            let ts_str = row.try_get::<String, _>("timestamp").ok()?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&ts_str)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            let raw_status = row.try_get::<String, _>("status").ok()?;
            let status = capitalize_first(raw_status.trim_matches('"'));
            Some(TaskRecord {
                id: row.try_get::<String, _>("id").ok()?,
                sender: row.try_get::<String, _>("sender").ok()?,
                receiver: row.try_get::<String, _>("receiver").ok()?,
                content: row.try_get::<String, _>("content").ok()?,
                status,
                created_at,
            })
        })
        .collect();

    Ok(records)
}