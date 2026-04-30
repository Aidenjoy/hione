use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Get the multiplexer binary name for the current platform
fn mux_bin() -> &'static str {
    if cfg!(windows) { "psmux" } else { "tmux" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub index: usize,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub launch_command: String,
    #[serde(default)]
    pub auto_mode: bool,
    #[serde(default)]
    pub resume_mode: bool,
    pub is_main: bool,
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmux_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub windows: Vec<WindowInfo>,
    pub work_dir: PathBuf,
    pub hione_dir: PathBuf,
    pub socket_path: String,
    pub monitor_pid: Option<u32>,
    /// tmux session name（用于清理）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmux_session_name: Option<String>,
}

impl SessionInfo {
    pub fn socket_path_for(hione_dir: &Path) -> String {
        #[cfg(unix)]
        return hione_dir.join("hi.sock").to_string_lossy().to_string();
        #[cfg(windows)]
        {
            // Derive unique pipe name from hione_dir path
            // Windows named pipes names must be valid and unique per project
            // Use a hash of the path to create a unique identifier
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            hione_dir.hash(&mut hasher);
            let hash = hasher.finish();
            format!("hione_{}", hash)
        }
    }

    /// 从文件加载 session（如果存在）
    pub fn load_from(hione_dir: &Path) -> Option<Self> {
        let path = hione_dir.join("session.json");
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// 清理旧 session：kill monitor + 关闭 mux session + 删除 socket
    /// 如果 `current_session_name` 与 self.tmux_session_name 相同，则跳过关闭 mux session
    pub fn cleanup(&self, current_session_name: Option<&str>) {
        // 1. Kill monitor 进程
        if let Some(pid) = self.monitor_pid {
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill")
                    .arg(pid.to_string())
                    .status();
            }
            #[cfg(windows)]
            {
                use std::process::Command;
                // Windows 使用 taskkill
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .status();
            }
        }

        // 2. 关闭 mux session（但跳过当前正在使用的同名 session）
        if let Some(session_name) = &self.tmux_session_name {
            // 空字符串 session 名称会导致 `kill-session -t ""` 杀掉当前 session，必须跳过
            if !session_name.is_empty() {
                use std::process::Command;
                // 如果当前在同名 mux session 中，跳过关闭，避免把用户的 session 杀掉
                let is_current_session = current_session_name == Some(session_name.as_str());
                if is_current_session {
                    // 只清理 hi panes（带有 @hi_label 标记的）
                    let output = Command::new(mux_bin())
                        .args(["list-panes", "-s", "-t", session_name, "-F", "#{pane_id}:#{@hi_label}"])
                        .output()
                        .ok();
                    if let Some(output) = output {
                        if let Ok(stdout) = String::from_utf8(output.stdout) {
                            for line in stdout.lines() {
                                let parts: Vec<&str> = line.splitn(2, ':').collect();
                                if parts.len() == 2 && !parts[1].is_empty() {
                                    let _ = Command::new(mux_bin())
                                        .args(["kill-pane", "-t", parts[0]])
                                        .status();
                                }
                            }
                        }
                    }
                } else {
                    let _ = Command::new(mux_bin())
                        .args(["kill-session", "-t", session_name])
                        .status();
                }
            }
        }

        // 3. 删除 socket 文件
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
