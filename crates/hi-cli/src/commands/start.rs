use anyhow::{Context, Result};
use hi_core::{
    db::init_db,
    message::{Message, MessageType, TaskStatus},
    session::{SessionInfo, WindowInfo},
};
use std::{env, fs, io::Write, path::Path, path::PathBuf, process::Command};
use uuid::Uuid;

use super::common::send_to_monitor;

/// Get the multiplexer binary name for the current platform
fn mux_bin() -> &'static str {
    if cfg!(windows) { "psmux" } else { "tmux" }
}

/// Check if we're inside a multiplexer session
fn in_mux_session() -> bool {
    // TMUX must be set AND non-empty (empty string means no real tmux session)
    if env::var("TMUX").ok().filter(|v| !v.is_empty()).is_some() {
        return true;
    }
    if env::var("PSMUX_SESSION_NAME").is_ok() {
        return true;
    }
    // psmux on Windows may not set any env var; probe it directly
    #[cfg(windows)]
    {
        Command::new("psmux")
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    false
}

#[derive(serde::Deserialize, Default)]
struct ToolConfig {
    #[serde(default)]
    auto_flags: Vec<String>,
    #[serde(default)]
    resume_flags: Vec<String>,
}

#[derive(serde::Deserialize, Default)]
struct ToolsFileConfig {
    #[serde(default)]
    tools: std::collections::HashMap<String, ToolConfig>,
}

fn load_tool_config(name: &str, hione_dir: &Path) -> Option<ToolConfig> {
    let config_path = hione_dir.join("tools.toml");
    if !config_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&config_path).ok()?;
    let mut file_config: ToolsFileConfig = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: failed to parse .hione/tools.toml: {e}");
            return None;
        }
    };
    file_config.tools.remove(name)
}

fn build_launch_command_with_hione(name: &str, auto_mode: bool, resume_mode: bool, hione_dir: &Path) -> String {
    if resume_mode {
        let with_resume = build_single_cmd(name, auto_mode, true, hione_dir);
        let without_resume = build_single_cmd(name, auto_mode, false, hione_dir);
        if with_resume != without_resume {
            // PowerShell doesn't support || operator, use $? to check last command success
            if cfg!(windows) {
                format!("{}; if (-not $?) {{ {} }}", with_resume, without_resume)
            } else {
                format!("{} || {}", with_resume, without_resume)
            }
        } else {
            with_resume
        }
    } else {
        build_single_cmd(name, auto_mode, false, hione_dir)
    }
}

fn build_single_cmd(name: &str, auto_mode: bool, resume_mode: bool, hione_dir: &Path) -> String {
    match name.to_lowercase().as_str() {
        "claude" => {
            let mut parts = vec!["claude".to_string()];
            if auto_mode   { parts.push("--dangerously-skip-permissions".into()); }
            if resume_mode { parts.push("--continue".into()); }
            parts.join(" ")
        }
        "gemini" => {
            let mut parts = vec!["gemini".to_string()];
            if auto_mode   { parts.push("--yolo".into()); }
            if resume_mode { parts.push("--resume".into()); parts.push("latest".into()); }
            parts.join(" ")
        }
        "opencode" => {
            let mut parts = vec!["opencode".to_string()];
            if resume_mode { parts.push("--continue".into()); }
            parts.join(" ")
        }
        "qwen" => {
            let mut parts = vec!["qwen".to_string()];
            if auto_mode   { parts.push("--yolo".into()); }
            if resume_mode { parts.push("--continue".into()); }
            parts.join(" ")
        }
        "codex" => {
            if resume_mode {
                let mut parts = vec!["codex".to_string(), "resume".into(), "--last".into()];
                if auto_mode { parts.push("--full-auto".into()); }
                parts.join(" ")
            } else {
                let mut parts = vec!["codex".to_string()];
                if auto_mode { parts.push("--full-auto".into()); }
                parts.join(" ")
            }
        }
        other => {
            if let Some(config) = load_tool_config(other, hione_dir) {
                let mut parts = vec![other.to_string()];
                if auto_mode   { parts.extend(config.auto_flags); }
                if resume_mode { parts.extend(config.resume_flags); }
                parts.join(" ")
            } else {
                other.to_string()
            }
        }
    }
}

pub async fn run(
    auto_mode: bool,
    resume_mode: bool,
    terminal_mode: bool,
    desktop_mode: bool,
    monitor_only: bool,
    names: Vec<String>,
) -> Result<()> {
    if names.is_empty() {
        anyhow::bail!("No commands specified. Usage: hi start [-a] [-r] [-T] [-D] claude,opencode,gemini");
    }

    // 1. 确定运行模式
    let in_mux = in_mux_session();
    let has_tauri = find_bin("hi-tauri").is_ok();

    // 在清理前先获取当前 multiplexer session 名称（如果有的话）
    let current_mux_session = if in_mux {
        Command::new(mux_bin())
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    let use_desktop = if monitor_only {
        false
    } else if terminal_mode {
        false
    } else if desktop_mode {
        if !has_tauri {
            anyhow::bail!("hi-tauri not found. Cannot use desktop mode.");
        }
        true
    } else {
        // 自动选择逻辑：
        // 1. 如果在 multiplexer 中，优先使用终端模式（满足 CLI 连续性）
        // 2. 否则如果找到 hi-tauri，使用桌面模式
        // 3. 否则回退到终端模式（会在 start_mux_fallback 中报错提示进入 multiplexer）
        !in_mux && has_tauri
    };

    // 2. 创建 .hione 目录
    let work_dir = env::current_dir()?;
    let hione_dir = work_dir.join(".hione");

    // 2.1 清理旧 session（如果存在）
    if let Some(old_session) = SessionInfo::load_from(&hione_dir) {
        println!("Cleaning up old session...");
        old_session.cleanup(current_mux_session.as_deref());
        // 给进程一点时间退出
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    fs::create_dir_all(&hione_dir).context("Failed to create .hione directory")?;
    fs::create_dir_all(hione_dir.join("logs"))?;

    // 阶段1：生成 .hione/CONTEXT.md
    generate_context_md(&hione_dir, &names)?;

    // 阶段2：追加到工具配置文件（忽略错误，不阻断启动）
    let _ = append_hi_section(&work_dir.join("CLAUDE.md"), &hione_dir);

    // 3. 初始化 SQLite
    init_db(&hione_dir).await?;

    // 4. 构建 SessionInfo
    let windows: Vec<WindowInfo> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let launch_command = build_launch_command_with_hione(name, auto_mode, resume_mode, &hione_dir);
            WindowInfo {
                index: i + 1,
                name: name.clone(),
                command: name.clone(),
                launch_command,
                auto_mode,
                resume_mode,
                is_main: i == 0,
                pid: None,
                tmux_pane_id: None,
            }
        })
        .collect();

    let socket_path = SessionInfo::socket_path_for(&hione_dir);
    let mut session = SessionInfo {
        id: Uuid::new_v4().to_string(),
        windows,
        work_dir: work_dir.clone(),
        hione_dir: hione_dir.clone(),
        socket_path: socket_path.clone(),
        monitor_pid: None,
        tmux_session_name: None,
    };

    // 5. 写入初始 session.json
    let session_json = serde_json::to_string_pretty(&session)?;
    fs::write(hione_dir.join("session.json"), &session_json)?;

    // 6. 启动 monitor 守护进程
    let monitor_bin = find_bin("hi-monitor")?;
    let mut monitor_cmd = Command::new(&monitor_bin);
    monitor_cmd.arg("--hione-dir").arg(&hione_dir);
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        monitor_cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let monitor_child = monitor_cmd.spawn()
        .context("Failed to start hi-monitor")?;

    let monitor_pid = monitor_child.id();
    tracing::info!("hi-monitor started with PID {}", monitor_pid);

    session.monitor_pid = Some(monitor_pid);
    let updated_json = serde_json::to_string_pretty(&session)?;
    fs::write(hione_dir.join("session.json"), &updated_json)?;

    if monitor_only {
        println!("Hi monitor daemon started with PID {}", monitor_pid);
        return Ok(());
    }

    // 7. 启动 UI (Tauri 或 Tmux)
    if use_desktop {
        let tauri_bin = find_bin("hi-tauri")?;
        Command::new(&tauri_bin)
            .arg("--session")
            .arg(hione_dir.join("session.json"))
            .spawn()
            .context("Failed to start hi-tauri")?;

        println!("Hi session started in Desktop mode: {}", work_dir.display());
    } else {
        // Multiplexer 模式
        start_mux_fallback(&names, &mut session, &hione_dir)?;

        // 写入更新后的 session.json（含 mux_session_name 和 pane_id）
        let updated_json = serde_json::to_string_pretty(&session)?;
        fs::write(hione_dir.join("session.json"), &updated_json)?;

        // mux 启动完成后，通知 monitor 更新 session（携带 pane_id）
        notify_session_ready(&session).await?;

        println!("Hi session started in Terminal mode");
    }

    Ok(())
}

fn generate_context_md(hione_dir: &Path, names: &[String]) -> Result<()> {
    const TEMPLATE: &str = include_str!("../templates/context_template.md");

    let peer_list = names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("- {} (index {})", n, i + 1))
        .collect::<Vec<_>>()
        .join("\n");

    let content = TEMPLATE.replace("{peer_list}", &peer_list);
    fs::write(hione_dir.join("CONTEXT.md"), content)?;
    Ok(())
}

fn append_hi_section(config_path: &Path, hione_dir: &Path) -> Result<()> {
    let start_marker = "<!-- hi-collaboration-start -->";
    let end_marker = "<!-- hi-collaboration-end -->";

    // 已存在时替换旧内容（peer 列表可能变化）
    if config_path.exists() {
        let existing = fs::read_to_string(config_path)?;
        if existing.contains(start_marker) {
            if let (Some(s), Some(e)) = (existing.find(start_marker), existing.find(end_marker)) {
                let new_content = format!(
                    "{}{}",
                    &existing[..s],
                    &existing[e + end_marker.len()..]
                );
                fs::write(config_path, &new_content)?;
            } else {
                return Ok(());
            }
        }
    }

    // 用相对路径表示 CONTEXT.md（相对于项目根目录）
    let rel_context = hione_dir
        .join("CONTEXT.md")
        .strip_prefix(config_path.parent().unwrap_or(Path::new(".")))
        .unwrap_or(&hione_dir.join("CONTEXT.md"))
        .to_string_lossy()
        .to_string();

    let section = format!(
        "\n{start_marker}\n\
        ## Hi Multi-Agent Collaboration\n\
        @{rel_context}\n\
        {end_marker}\n"
    );

    // 追加（文件不存在则创建）
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)?;
    file.write_all(section.as_bytes())?;
    Ok(())
}

fn find_bin(name: &str) -> Result<PathBuf> {
    if let Ok(path) = which::which(name) {
        return Ok(path);
    }

    let exe = env::current_exe()?;
    let parent = exe.parent().ok_or_else(|| anyhow::anyhow!("Cannot get exe parent dir"))?;

    // On Windows, sibling executables have .exe extension
    #[cfg(windows)]
    {
        let with_exe = parent.join(format!("{}.exe", name));
        if with_exe.exists() {
            return Ok(with_exe);
        }
    }

    let sibling = parent.join(name);
    if sibling.exists() {
        return Ok(sibling);
    }

    anyhow::bail!("{name} not found in PATH")
}

fn start_mux_fallback(names: &[String], session: &mut SessionInfo, hione_dir: &Path) -> Result<()> {
    let mux = mux_bin();
    if !in_mux_session() {
        anyhow::bail!(
            "hi-tauri not found. Install with: bash scripts/install.sh --with-desktop\n\
             Or run inside {} for terminal fallback.\n\
             On Windows: psmux new-session -s hi\n\
             On Unix: tmux new-session -s hi",
            mux
        );
    }

    let n = names.len();
    if n == 0 {
        return Ok(())
    }

    let session_name_output = Command::new(mux)
        .args(["display-message", "-p", "#{session_name}"])
        .output()
        .context("Failed to get mux session name")?;
    let mux_session_name = String::from_utf8(session_name_output.stdout)?.trim().to_string();
    if mux_session_name.is_empty() {
        anyhow::bail!("Failed to get mux session name: tmux returned empty result. Make sure you are inside a valid tmux session.");
    }

    let hi_panes_output = Command::new(mux)
        .args(["list-panes", "-s", "-t", &mux_session_name, "-F", "#{pane_id}:#{@hi_label}"])
        .output()
        .context("Failed to list mux panes")?;
    let hi_panes_str = String::from_utf8(hi_panes_output.stdout)?;
    let hi_panes_count = hi_panes_str
        .lines()
        .filter(|line| line.contains(":") && line.split(":").last().map(|s| !s.is_empty()).unwrap_or(false))
        .count();

    if hi_panes_count > 0 {
        anyhow::bail!(
            "Current {} session '{}' already has {} hi panes.\n\
             Please create a new session for this project:\n\
             {} new -s <name>\n\
             Or clean up the existing hi session first:\n\
             hi s --monitor-only",
            mux, mux_session_name, hi_panes_count, mux
        );
    }

    session.tmux_session_name = Some(mux_session_name.clone());

    let output = Command::new(mux)
        .args(["display-message", "-p", "#{pane_id}"])
        .output()
        .context("Failed to get initial mux pane ID")?;
    let ids_str = String::from_utf8(output.stdout)?;
    let initial_pane_id = ids_str.trim().to_string();

    let mut pane_ids: Vec<String> = Vec::with_capacity(n);

    if n == 1 {
        let output = Command::new(mux)
            .args(["split-window", "-h", "-p", "50", "-P", "-F", "#{pane_id}"])
            .output()
            .context("Failed to split mux window horizontally")?;
        let new_pane = String::from_utf8(output.stdout)?.trim().to_string();
        pane_ids.push(new_pane);
    } else {
        let l_count = n / 2;
        let r_count = n - l_count;

        // Right column: first pane from horizontal split, rest from vertical splits
        let output = Command::new(mux)
            .args(["split-window", "-h", "-p", "50", "-P", "-F", "#{pane_id}"])
            .output()
            .context("Failed to split mux window horizontally")?;
        let right_top = String::from_utf8(output.stdout)?.trim().to_string();
        let mut right_panes = vec![right_top.clone()];
        let mut last_right = right_top;
        for _ in 1..r_count {
            let output = Command::new(mux)
                .args(["split-window", "-v", "-t", &last_right, "-P", "-F", "#{pane_id}"])
                .output()
                .context("Failed to split mux window vertically in right column")?;
            last_right = String::from_utf8(output.stdout)?.trim().to_string();
            right_panes.push(last_right.clone());
        }

        // Left column: l_count panes created by splitting initial_pane (which gets killed)
        let mut left_panes = Vec::new();
        let mut last_left = initial_pane_id.clone();
        for _ in 0..l_count {
            let output = Command::new(mux)
                .args(["split-window", "-v", "-t", &last_left, "-P", "-F", "#{pane_id}"])
                .output()
                .context("Failed to split mux window vertically in left column")?;
            last_left = String::from_utf8(output.stdout)?.trim().to_string();
            left_panes.push(last_left.clone());
        }

        // Interleave columns: [left[0], right[0], left[1], right[1], ...]
        // Extra right panes (when r_count > l_count) are appended at the end.
        // This gives the layout: 2→(L,R), 3→(L,R,R), 4→(L,R,L,R),
        //   5→(L,R,L,R,R), 6→(L,R,L,R,L,R)
        for i in 0..l_count.max(r_count) {
            if i < left_panes.len()  { pane_ids.push(left_panes[i].clone()); }
            if i < right_panes.len() { pane_ids.push(right_panes[i].clone()); }
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(200));

    for (i, name) in names.iter().enumerate() {
        let pane_id = &pane_ids[i];
        let label = format!("{} {}", i + 1, name);

        Command::new(mux)
            .args(["set-option", "-p", "-t", pane_id, "@hi_label", &label])
            .status()
            .context("Failed to set mux pane label")?;

        let launch_command = &session.windows[i].launch_command;
        // 为子窗口追加 ; exit，为主窗口追加 ; tmux kill-session，确保 AI 退出时自动清理 tmux
        let full_launch_cmd = if i == 0 {
            if cfg!(windows) {
                // psmux on Windows might not support kill-session yet, use exit
                format!("{}; exit", launch_command)
            } else {
                format!("{}; tmux kill-session", launch_command)
            }
        } else {
            format!("{}; exit", launch_command)
        };

        Command::new(mux)
            .args(["send-keys", "-t", pane_id, &full_launch_cmd, "Enter"])
            .status()
            .context("Failed to send keys to mux pane")?;

        if let Some(window) = session.windows.iter_mut().find(|w| w.name == *name) {
            window.tmux_pane_id = Some(pane_id.clone());
        }
    }

    Command::new(mux)
        .args(["set-option", "pane-border-status", "top"])
        .status()
        .context("Failed to set mux pane-border-status")?;

    Command::new(mux)
        .args([
            "set-option",
            "pane-border-format",
            "#[bg=colour235,fg=colour214,bold] #{@hi_label} #[default]",
        ])
        .status()
        .context("Failed to set mux pane-border-format")?;

    Command::new(mux)
        .args(["set-option", "pane-border-style", "fg=colour240"])
        .status()
        .context("Failed to set mux pane-border-style")?;

    Command::new(mux)
        .args(["set-option", "pane-active-border-style", "fg=colour214"])
        .status()
        .context("Failed to set mux pane-active-border-style")?;

    Command::new(mux)
        .args(["set-option", "-g", "mouse", "on"])
        .status()
        .context("Failed to enable mux mouse support")?;

    // 8. 更新 session.json
    let updated_json = serde_json::to_string_pretty(session)?;
    fs::write(hione_dir.join("session.json"), &updated_json)?;

    // 终端窗口关闭时（无 client 附加），自动销毁 session
    let _ = Command::new(mux)
        .args(["set-option", "destroy-unattached", "on"])
        .status();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = Command::new(mux)
        .args(["kill-pane", "-t", &initial_pane_id])
        .status();

    println!("Hi session started in {} mode", mux);
    println!("Panes: {}", names.join(", "));
    Ok(())
}

/// 通知 monitor session 已就绪（携带 pane_id 映射）
async fn notify_session_ready(session: &SessionInfo) -> Result<()> {
    // 将完整的 session 序列化后发送给 monitor
    let session_json = serde_json::to_string(session)?;
    let msg = Message {
        id: Uuid::new_v4(),
        sender: "cli".to_string(),
        receiver: "monitor".to_string(),
        timestamp: chrono::Utc::now(),
        content: session_json,
        msg_type: MessageType::SessionReady,
        status: TaskStatus::Completed,
        parent_id: None,
    };

    // monitor 可能还没完全启动，重试几次
    for i in 0..5 {
        match send_to_monitor(&session.socket_path, &msg).await {
            Ok(()) => {
                tracing::info!("SessionReady sent to monitor");
                return Ok(());
            }
            Err(e) if i < 4 => {
                tracing::warn!("SessionReady send failed (attempt {}): {}, retrying...", i + 1, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
            Err(e) => {
                // 最后一次失败也不阻断启动，只是警告
                tracing::warn!("SessionReady send failed after 5 attempts: {}", e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(name: &str, auto: bool, resume: bool) -> String {
        // tests have no .hione/tools.toml, so custom tools pass through
        build_launch_command_with_hione(name, auto, resume, Path::new("/nonexistent/.hione"))
    }

    #[test]
    fn test_build_launch_command() {
        // Helper to get expected fallback syntax based on platform
        let fallback = |with: &str, without: &str| -> String {
            if cfg!(windows) {
                format!("{}; if (-not $?) {{ {} }}", with, without)
            } else {
                format!("{} || {}", with, without)
            }
        };

        // Claude
        assert_eq!(cmd("claude", false, false), "claude");
        assert_eq!(cmd("claude", true,  false), "claude --dangerously-skip-permissions");
        assert_eq!(cmd("claude", false, true),  fallback("claude --continue", "claude"));
        assert_eq!(cmd("claude", true,  true),  fallback("claude --dangerously-skip-permissions --continue", "claude --dangerously-skip-permissions"));

        // Gemini
        assert_eq!(cmd("gemini", false, false), "gemini");
        assert_eq!(cmd("gemini", true,  false), "gemini --yolo");
        assert_eq!(cmd("gemini", false, true),  fallback("gemini --resume latest", "gemini"));
        assert_eq!(cmd("gemini", true,  true),  fallback("gemini --yolo --resume latest", "gemini --yolo"));

        // OpenCode — no auto flag
        assert_eq!(cmd("opencode", false, false), "opencode");
        assert_eq!(cmd("opencode", true,  false), "opencode");
        assert_eq!(cmd("opencode", false, true),  fallback("opencode --continue", "opencode"));
        assert_eq!(cmd("opencode", true,  true),  fallback("opencode --continue", "opencode"));

        // Qwen
        assert_eq!(cmd("qwen", false, false), "qwen");
        assert_eq!(cmd("qwen", true,  false), "qwen --yolo");
        assert_eq!(cmd("qwen", false, true),  fallback("qwen --continue", "qwen"));
        assert_eq!(cmd("qwen", true,  true),  fallback("qwen --yolo --continue", "qwen --yolo"));

        // Codex — resume is a subcommand
        assert_eq!(cmd("codex", false, false), "codex");
        assert_eq!(cmd("codex", true,  false), "codex --full-auto");
        assert_eq!(cmd("codex", false, true),  fallback("codex resume --last", "codex"));
        assert_eq!(cmd("codex", true,  true),  fallback("codex resume --last --full-auto", "codex --full-auto"));

        // Unknown — pass through (no tools.toml in test)
        assert_eq!(cmd("mytool", false, false), "mytool");
        assert_eq!(cmd("mytool", true,  true),  "mytool");
    }

    #[test]
    fn test_build_launch_command_custom_tool() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let hione_dir = tmp.path().to_path_buf();
        let tools_toml = hione_dir.join("tools.toml");
        let mut f = std::fs::File::create(&tools_toml).unwrap();
        f.write_all(
            b"[tools.ccg]\nauto_flags = [\"--dangerously-skip-permissions\"]\nresume_flags = [\"--continue\"]\n",
        )
        .unwrap();

        assert_eq!(
            build_launch_command_with_hione("ccg", false, false, &hione_dir),
            "ccg"
        );
        assert_eq!(
            build_launch_command_with_hione("ccg", true, false, &hione_dir),
            "ccg --dangerously-skip-permissions"
        );
        // Resume fallback uses platform-specific syntax
        let expected_resume = if cfg!(windows) {
            "ccg --continue; if (-not $?) { ccg }"
        } else {
            "ccg --continue || ccg"
        };
        assert_eq!(
            build_launch_command_with_hione("ccg", false, true, &hione_dir),
            expected_resume
        );
        let expected_resume_auto = if cfg!(windows) {
            "ccg --dangerously-skip-permissions --continue; if (-not $?) { ccg --dangerously-skip-permissions }"
        } else {
            "ccg --dangerously-skip-permissions --continue || ccg --dangerously-skip-permissions"
        };
        assert_eq!(
            build_launch_command_with_hione("ccg", true, true, &hione_dir),
            expected_resume_auto
        );
        // tool not in config still passes through
        assert_eq!(
            build_launch_command_with_hione("other", true, true, &hione_dir),
            "other"
        );
    }
}
