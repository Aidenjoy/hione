use which::which;
use std::process::Command;
use tauri::{Window, Emitter};
use serde_json::json;
use crate::error::AppError;
use crate::types::ToolInfo;

#[cfg(not(windows))]
use std::process::Stdio;
#[cfg(not(windows))]
use std::io::{BufRead, BufReader};

pub struct KnownTool {
    pub name: &'static str,
    pub install_cmd: &'static str,
    pub uninstall_cmd: &'static str,
    pub bin_name: &'static str,
    pub version_cmd: &'static str,
}

pub const KNOWN_TOOLS: &[KnownTool] = &[
    KnownTool {
        name:        if cfg!(windows) { "psmux" } else { "tmux" },
        bin_name:    if cfg!(windows) { "psmux" } else { "tmux" },
        install_cmd: if cfg!(target_os = "macos") { "brew install tmux" }
                     else if cfg!(windows)         { "winget install psmux" }
                     else                          { "sudo apt install -y tmux" },
        uninstall_cmd: if cfg!(target_os = "macos") { "brew uninstall tmux" }
                       else if cfg!(windows)         { "winget uninstall psmux" }
                       else                          { "sudo apt remove -y tmux" },
        version_cmd: if cfg!(windows) { "psmux version" } else { "tmux -V" },
    },
    KnownTool {
        name: "claude",
        bin_name: "claude",
        install_cmd: "npm install -g @anthropic-ai/claude-code",
        uninstall_cmd: "npm uninstall -g @anthropic-ai/claude-code",
        version_cmd: "claude --version",
    },
    KnownTool {
        name: "gemini",
        bin_name: "gemini",
        install_cmd: "npm install -g @google/gemini-cli",
        uninstall_cmd: "npm uninstall -g @google/gemini-cli",
        version_cmd: "gemini -v",
    },
    KnownTool {
        name: "opencode",
        bin_name: "opencode",
        install_cmd: "npm install -g opencode-ai",
        uninstall_cmd: "npm uninstall -g opencode-ai",
        version_cmd: "opencode -v",
    },
    KnownTool {
        name: "codex",
        bin_name: "codex",
        install_cmd: "npm install -g @openai/codex",
        uninstall_cmd: "npm uninstall -g @openai/codex",
        version_cmd: "codex --version",
    },
    KnownTool {
        name: "qwen",
        bin_name: "qwen",
        install_cmd: "npm install -g @qwen-code/qwen-code",
        uninstall_cmd: "npm uninstall -g @qwen-code/qwen-code",
        version_cmd: "qwen -v",
    },
];

fn resolve_nvm_alias(alias: &str, aliases_dir: &std::path::Path, depth: u32) -> Option<String> {
    if depth == 0 {
        return None;
    }
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('v') || trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return Some(trimmed.to_string());
    }
    let alias_file = aliases_dir.join(trimmed);
    if let Ok(content) = std::fs::read_to_string(&alias_file) {
        let content = content.trim();
        if content != trimmed {
            return resolve_nvm_alias(content, aliases_dir, depth - 1);
        }
    }
    None
}

fn nvm_bin_path(home: &str, name: &str) -> Option<std::path::PathBuf> {
    let nvm_dir = std::path::PathBuf::from(format!("{}/.nvm", home));
    let versions_dir = nvm_dir.join("versions/node");
    if !versions_dir.exists() {
        return None;
    }

    let aliases_dir = nvm_dir.join("alias");
    let default_alias_file = aliases_dir.join("default");
    if default_alias_file.exists() {
        if let Ok(default) = std::fs::read_to_string(&default_alias_file) {
            if let Some(version) = resolve_nvm_alias(&default, &aliases_dir, 5) {
                let version_candidates: Vec<String> = if version.starts_with('v') {
                    vec![version.clone(), version[1..].to_string()]
                } else {
                    vec![version.clone(), format!("v{}", version)]
                };
                for v in version_candidates {
                    let candidate = versions_dir.join(&v).join("bin").join(name);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    let mut entries: Vec<_> = std::fs::read_dir(&versions_dir)
        .ok()?
        .flatten()
        .collect();
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    for entry in entries {
        let candidate = entry.path().join("bin").join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn fnm_bin_path(home: &str, name: &str) -> Option<std::path::PathBuf> {
    let fnm_dir = std::env::var("FNM_DIR")
        .unwrap_or_else(|_| format!("{}/.local/share/fnm", home));

    let fnm_dir_path = std::path::PathBuf::from(&fnm_dir);
    let versions_dir = fnm_dir_path.join("node-versions");
    if !versions_dir.exists() {
        return None;
    }

    if let Ok(multishell) = std::env::var("FNM_MULTISHELL_PATH") {
        if let Ok(resolved) = std::fs::canonicalize(&multishell) {
            let candidate = resolved.join("bin").join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    let default_alias = fnm_dir_path.join("aliases").join("default");
    if default_alias.exists() {
        if let Ok(resolved) = std::fs::canonicalize(&default_alias) {
            let candidate = resolved.join("bin").join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    let mut entries: Vec<_> = std::fs::read_dir(&versions_dir)
        .ok()?
        .flatten()
        .collect();
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    for entry in entries {
        let candidate = entry.path().join("installation/bin").join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn which_via_shell(name: &str) -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("zsh")
        .args(["-i", "-l", "-c", &format!("hash -r 2>/dev/null; command -v {} 2>/dev/null", name)])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && !path.contains("command not found") {
            return Some(std::path::PathBuf::from(path));
        }
    }
    let output = std::process::Command::new("sh")
        .args(["-i", "-l", "-c", &format!("hash -r 2>/dev/null; command -v {} 2>/dev/null", name)])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(std::path::PathBuf::from(path));
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn which_via_shell(name: &str) -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("sh")
        .args(["-i", "-l", "-c", &format!("hash -r 2>/dev/null; command -v {} 2>/dev/null", name)])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(std::path::PathBuf::from(path));
        }
    }
    None
}

fn find_binary(name: &str) -> Option<std::path::PathBuf> {
    if let Ok(path) = which(name) {
        return Some(path);
    }

    // Windows: check npm global dir (%APPDATA%\npm), WinGet packages dir, and use `where.exe` for fresh PATH lookup.
    // Must come before the Unix HOME block because HOME is not set on Windows.
    #[cfg(windows)]
    {
        // Check npm global directory
        if let Ok(appdata) = std::env::var("APPDATA") {
            let npm_dir = std::path::PathBuf::from(&appdata).join("npm");
            for ext in &["", ".cmd", ".ps1", ".exe"] {
                let p = npm_dir.join(format!("{}{}", name, ext));
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // Check WinGet Packages directory (winget installs here but may not add to PATH immediately)
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            let winget_base = std::path::PathBuf::from(&localappdata)
                .join("Microsoft")
                .join("WinGet")
                .join("Packages");
            if winget_base.exists() {
                // WinGet package folders have names like "marlocarlo.psmux_Microsoft.Winget.Source_8wekyb3d8bbwe"
                if let Ok(entries) = std::fs::read_dir(&winget_base) {
                    for entry in entries.flatten() {
                        let pkg_dir = entry.path();
                        // Look for the executable directly in the package folder
                        let exe_path = pkg_dir.join(format!("{}.exe", name));
                        if exe_path.exists() {
                            return Some(exe_path);
                        }
                        // Also check for name without .exe extension
                        let bin_path = pkg_dir.join(name);
                        if bin_path.exists() {
                            return Some(bin_path);
                        }
                    }
                }
            }
        }

        // Use PowerShell where.exe to find binary (fresh PATH lookup)
        if let Ok(out) = Command::new("where.exe").arg(name).output() {
            if out.status.success() {
                let first = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !first.is_empty() {
                    return Some(std::path::PathBuf::from(first));
                }
            }
        }
        return None;
    }

    // Unix-only paths below
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").ok()?;
        let common_paths = [
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
            "/opt/homebrew/bin",
            &format!("{}/.local/bin", home),
            &format!("{}/.cargo/bin", home),
            &format!("{}/.npm-global/bin", home),
        ];

        for path in common_paths {
            let full_path = std::path::Path::new(path).join(name);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        if let Some(p) = nvm_bin_path(&home, name) {
            return Some(p);
        }

        if let Some(p) = fnm_bin_path(&home, name) {
            return Some(p);
        }

        let volta = std::path::PathBuf::from(format!("{}/.volta/bin/{}", home, name));
        if volta.exists() {
            return Some(volta);
        }

        if let Some(p) = which_via_shell(name) {
            if p.exists() {
                return Some(p);
            }
        }

        None
    }
}

fn companion_npm(tool_bin_name: &str) -> Option<std::path::PathBuf> {
    let bin_path = find_binary(tool_bin_name)?;
    let parent = bin_path.parent()?;
    let npm = parent.join("npm");
    if npm.exists() {
        return Some(npm);
    }
    #[cfg(windows)]
    {
        let npm_cmd = parent.join("npm.cmd");
        if npm_cmd.exists() {
            return Some(npm_cmd);
        }
    }
    None
}

fn resolve_cmd(cmd_str: &str) -> String {
    let first = cmd_str.split_whitespace().next().unwrap_or("");
    if first.is_empty() {
        return cmd_str.to_string();
    }

    #[cfg(target_os = "macos")]
    if first == "brew" {
        let is_rosetta = std::process::Command::new("sysctl")
            .args(["-n", "sysctl.proc_translated"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
            .unwrap_or(false);

        let arm_brew = "/opt/homebrew/bin/brew";
        let intel_brew = "/usr/local/bin/brew";
        let rest = cmd_str[first.len()..].trim_start();

        if is_rosetta && std::path::Path::new(arm_brew).exists() {
            return format!("arch -arm64 {} {}", arm_brew, rest);
        } else if std::path::Path::new(arm_brew).exists() {
            return format!("{} {}", arm_brew, rest);
        } else if std::path::Path::new(intel_brew).exists() {
            return format!("{} {}", intel_brew, rest);
        }
        return cmd_str.to_string();
    }

    if let Ok(p) = which(first) {
        return cmd_str.replacen(first, &p.to_string_lossy(), 1);
    }
    if let Some(p) = find_binary(first) {
        return cmd_str.replacen(first, &p.to_string_lossy(), 1);
    }
    cmd_str.to_string()
}

/// Check if WSL is available on this Windows machine.
#[cfg(windows)]
pub fn is_wsl_available() -> bool {
    Command::new("wsl")
        .args(["--", "echo", "ok"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn parse_version_from_output(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    let v = String::from_utf8_lossy(stdout).trim().to_string();
    let v = if v.is_empty() {
        String::from_utf8_lossy(stderr).trim().to_string()
    } else {
        v
    };
    if v.is_empty() {
        None
    } else {
        Some(v.trim_start_matches('v').to_string())
    }
}

pub fn detect_tool(name: &str) -> (bool, Option<String>) {
    let tool = KNOWN_TOOLS.iter().find(|t| t.name == name);

    if tool.is_none() {
        return (false, None);
    }

    let tool = tool.unwrap();

    // On Windows: check WSL first, fall back to native
    #[cfg(windows)]
    {
        if is_wsl_available() {
            let wsl_check = Command::new("wsl")
                .args(["--", "which", tool.bin_name])
                .output();
            if let Ok(out) = wsl_check {
                if out.status.success() {
                    let version = Command::new("wsl")
                        .args(["--", "bash", "-c", tool.version_cmd])
                        .output()
                        .ok()
                        .and_then(|o| parse_version_from_output(&o.stdout, &o.stderr));
                    return (true, version);
                }
            }
        }
        // Fall back to native Windows detection
        let bin_path = match find_binary(tool.bin_name) {
            Some(p) => p,
            None => return (false, None),
        };
        let bin_path_quoted = format!("\"{}\"", bin_path.to_string_lossy());
        // Windows: use PowerShell to get version
        let cmd_to_run = tool.version_cmd.replace(tool.bin_name, &bin_path_quoted);
        let version = Command::new("powershell")
            .args(["-Command", &cmd_to_run])
            .output()
            .ok()
            .and_then(|o| parse_version_from_output(&o.stdout, &o.stderr));
        return (true, version);
    }

    // Unix
    #[cfg(not(windows))]
    {
        let bin_path = match find_binary(tool.bin_name) {
            Some(p) => p,
            None => return (false, None),
        };
        let cmd_to_run = tool.version_cmd.replace(tool.bin_name, &bin_path.to_string_lossy());

        #[cfg(target_os = "macos")]
        let output = Command::new("zsh")
            .args(["-i", "-l", "-c", &cmd_to_run])
            .output()
            .or_else(|_| {
                Command::new("sh")
                    .args(["-i", "-l", "-c", &cmd_to_run])
                    .output()
            });

        #[cfg(not(target_os = "macos"))]
        let output = Command::new("sh")
            .args(["-i", "-l", "-c", &cmd_to_run])
            .output();

        let version = output.ok()
            .and_then(|o| parse_version_from_output(&o.stdout, &o.stderr));
        (true, version)
    }
}

pub fn detect_all_tools() -> Vec<ToolInfo> {
    KNOWN_TOOLS
        .iter()
        .map(|t| {
            let (installed, version) = detect_tool(t.name);
            ToolInfo {
                name: t.name.to_string(),
                installed,
                version,
            }
        })
        .collect()
}

/// Find binary path after installation, searching WinGet packages directory
#[cfg(windows)]
fn find_binary_after_install(name: &str) -> Option<std::path::PathBuf> {
    // First try the regular find_binary which checks PATH and where.exe
    if let Some(path) = find_binary(name) {
        return Some(path);
    }

    // Then check WinGet Packages directory
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        let winget_base = std::path::PathBuf::from(&localappdata)
            .join("Microsoft")
            .join("WinGet")
            .join("Packages");
        if winget_base.exists() {
            if let Ok(entries) = std::fs::read_dir(&winget_base) {
                for entry in entries.flatten() {
                    let pkg_dir = entry.path();
                    let exe_path = pkg_dir.join(format!("{}.exe", name));
                    if exe_path.exists() {
                        return Some(exe_path);
                    }
                }
            }
        }
    }

    None
}

/// Get tool version by running version command
#[cfg(windows)]
fn get_tool_version(name: &str) -> Option<String> {
    let bin_path = find_binary_after_install(name)?;

    let tool = KNOWN_TOOLS.iter().find(|t| t.bin_name == name)?;
    // Use PowerShell call operator `&` to execute quoted path with arguments
    let ps_cmd = format!("& '{}' {}", bin_path.to_string_lossy(), tool.version_cmd.replace(name, ""));

    Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .output()
        .ok()
        .and_then(|o| parse_version_from_output(&o.stdout, &o.stderr))
}

#[allow(unused_variables)]
pub async fn install_tool_async(name: String, window: Window) -> Result<(), AppError> {
    let tool = KNOWN_TOOLS.iter().find(|t| t.name == name);

    if tool.is_none() {
        return Err(AppError::CommandFailed(format!("Unknown tool: {}", name)));
    }

    let tool = tool.unwrap();

    // Windows: run install command, then add to PATH if needed
    #[cfg(windows)]
    {
        let bin_name = tool.bin_name;
        let install_cmd = tool.install_cmd;

        // Emit install command
        let _ = window.emit("tool://install-log", json!({
            "name": name.clone(),
            "line": format!("执行安装命令: {}", install_cmd)
        }));

        // Run winget install with auto-accept flags
        let install_result = if install_cmd.starts_with("winget install") {
            let package_name = install_cmd.split_whitespace().nth(2).unwrap_or("");
            Command::new("winget")
                .args(["install", package_name, "--accept-package-agreements", "--accept-source-agreements"])
                .output()
        } else {
            Command::new("powershell")
                .args(["-Command", install_cmd])
                .output()
        };

        match install_result {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let _ = window.emit("tool://install-log", json!({ "name": name.clone(), "line": line }));
                }

                // Find the installed binary path
                let bin_path = find_binary_after_install(bin_name);
                if let Some(path) = bin_path {
                    let path_dir = path.parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Check if path_dir is already in User PATH (permanent)
                    let user_path_result = Command::new("powershell")
                        .args(["-Command", "[Environment]::GetEnvironmentVariable('PATH', 'User')"])
                        .output();

                    let user_path = user_path_result
                        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                        .unwrap_or_default();

                    if !user_path.contains(&path_dir) {
                        // Add to user PATH permanently
                        let ps_script = format!(
                            "$currentPath = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
                             [Environment]::SetEnvironmentVariable('PATH', $currentPath + ';' + '{}', 'User'); \
                             Write-Host '已将 {} 添加到 PATH'",
                            path_dir.replace('\\', "\\\\"),
                            path_dir.replace('\\', "\\\\")
                        );
                        let _ = Command::new("powershell")
                            .args(["-Command", &ps_script])
                            .output();

                        let _ = window.emit("tool://install-log", json!({
                            "name": name.clone(),
                            "line": format!("已将 {} 添加到 PATH，请重启终端后使用", path_dir)
                        }));
                    } else {
                        let _ = window.emit("tool://install-log", json!({
                            "name": name.clone(),
                            "line": format!("{} 已在 PATH 中，请重启终端后使用", path_dir)
                        }));
                    }

                    // Get and show version
                    if let Some(version) = get_tool_version(bin_name) {
                        let _ = window.emit("tool://install-log", json!({
                            "name": name.clone(),
                            "line": format!("{} 安装成功，版本: {}", name, version)
                        }));
                    } else {
                        let _ = window.emit("tool://install-log", json!({
                            "name": name.clone(),
                            "line": format!("{} 安装成功", name)
                        }));
                    }
                } else {
                    let _ = window.emit("tool://install-log", json!({
                        "name": name.clone(),
                        "line": format!("{} 安装完成，请重启终端后验证", name)
                    }));
                }
                Ok(())
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines() {
                    let _ = window.emit("tool://install-log", json!({ "name": name.clone(), "line": line }));
                }
                Err(AppError::CommandFailed(format!("Install failed for {}", name)))
            }
            Err(e) => {
                Err(AppError::CommandFailed(format!("Install command failed: {}", e)))
            }
        }
    }

    // Unix platforms
    #[cfg(not(windows))]
    {
        let cmd_str = tool.install_cmd;

        #[cfg(target_os = "macos")]
        let mut child = {
            let resolved_cmd = if cmd_str.starts_with("npm install") {
                match companion_npm(tool.bin_name) {
                    Some(npm_path) => cmd_str.replacen("npm", &npm_path.to_string_lossy(), 1),
                    None => resolve_cmd(cmd_str),
                }
            } else {
                resolve_cmd(cmd_str)
            };
            Command::new("zsh")
                .args(["-i", "-l", "-c", &resolved_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .or_else(|_| {
                    Command::new("sh")
                        .args(["-i", "-l", "-c", &resolved_cmd])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                })?
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut child = {
            let resolved_cmd = if cmd_str.starts_with("npm install") {
                match companion_npm(tool.bin_name) {
                    Some(npm_path) => cmd_str.replacen("npm", &npm_path.to_string_lossy(), 1),
                    None => resolve_cmd(cmd_str),
                }
            } else {
                resolve_cmd(cmd_str)
            };
            Command::new("sh")
                .args(["-i", "-l", "-c", &resolved_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window.emit("tool://install-log", json!({ "name": name.clone(), "line": line }));
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window.emit("tool://install-log", json!({ "name": name.clone(), "line": line }));
                }
            }
        }

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::CommandFailed(format!("Install failed for {}", name)))
        }
    }
}

#[allow(unused_variables)]
pub async fn uninstall_tool_async(name: String, window: Window) -> Result<(), AppError> {
    let tool = KNOWN_TOOLS.iter().find(|t| t.name == name);

    if tool.is_none() {
        return Err(AppError::CommandFailed(format!("Unknown tool: {}", name)));
    }

    let tool = tool.unwrap();

    // Windows: run uninstall command and log output
    #[cfg(windows)]
    {
        let uninstall_cmd = tool.uninstall_cmd;

        let _ = window.emit("tool://uninstall-log", json!({
            "name": name.clone(),
            "line": format!("执行卸载命令: {}", uninstall_cmd)
        }));

        let uninstall_result = if uninstall_cmd.starts_with("winget uninstall") {
            let package_name = uninstall_cmd.split_whitespace().nth(2).unwrap_or("");
            Command::new("winget")
                .args(["uninstall", package_name, "--silent"])
                .output()
        } else {
            Command::new("powershell")
                .args(["-Command", uninstall_cmd])
                .output()
        };

        match uninstall_result {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let _ = window.emit("tool://uninstall-log", json!({ "name": name.clone(), "line": line }));
                }
                let _ = window.emit("tool://uninstall-log", json!({
                    "name": name.clone(),
                    "line": format!("{} 已卸载", name)
                }));
                Ok(())
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stderr.lines() {
                    let _ = window.emit("tool://uninstall-log", json!({ "name": name.clone(), "line": line }));
                }
                Err(AppError::CommandFailed(format!("Uninstall failed for {}", name)))
            }
            Err(e) => {
                Err(AppError::CommandFailed(format!("Uninstall command failed: {}", e)))
            }
        }
    }

    // Unix platforms
    #[cfg(not(windows))]
    {
        let cmd_str = tool.uninstall_cmd;

        #[cfg(target_os = "macos")]
        let mut child = {
            let resolved_cmd = if cmd_str.starts_with("npm uninstall") {
                match companion_npm(tool.bin_name) {
                    Some(npm_path) => cmd_str.replacen("npm", &npm_path.to_string_lossy(), 1),
                    None => resolve_cmd(cmd_str),
                }
            } else {
                resolve_cmd(cmd_str)
            };
            Command::new("zsh")
                .args(["-i", "-l", "-c", &resolved_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .or_else(|_| {
                    Command::new("sh")
                        .args(["-i", "-l", "-c", &resolved_cmd])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                })?
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut child = {
            let resolved_cmd = if cmd_str.starts_with("npm uninstall") {
                match companion_npm(tool.bin_name) {
                    Some(npm_path) => cmd_str.replacen("npm", &npm_path.to_string_lossy(), 1),
                    None => resolve_cmd(cmd_str),
                }
            } else {
                resolve_cmd(cmd_str)
            };
            Command::new("sh")
                .args(["-i", "-l", "-c", &resolved_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window.emit("tool://uninstall-log", json!({ "name": name.clone(), "line": line }));
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window.emit("tool://uninstall-log", json!({ "name": name.clone(), "line": line }));
                }
            }
        }

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::CommandFailed(format!("Uninstall failed for {}", name)))
        }
    }
}