use crate::types::SetupStatus;
use tauri::{Window, Emitter, AppHandle, Manager};
use which::which;
use serde_json::json;
use std::path::PathBuf;

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    home.join(".hione").join("desktop_settings.json")
}

fn get_custom_bin_paths() -> (Option<String>, Option<String>) {
    let path = settings_path();
    if !path.exists() {
        return (None, None);
    }
    
    let content = std::fs::read_to_string(&path).ok();
    if let Some(c) = content {
        let settings: crate::types::AppSettings = serde_json::from_str(&c).ok().unwrap_or_default();
        return (settings.hi_bin_path, settings.hi_monitor_bin_path);
    }
    
    (None, None)
}

/// On Windows, check if a binary exists inside WSL.
#[cfg(windows)]
fn wsl_has(bin: &str) -> bool {
    std::process::Command::new("wsl")
        .args(["--", "which", bin])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub async fn check_setup() -> Result<SetupStatus, String> {
    let (hi_path, monitor_path) = get_custom_bin_paths();

    #[cfg(windows)]
    let (tmux, node, rust) = (
        wsl_has("tmux") || which("tmux").is_ok(),
        wsl_has("node") || which("node").is_ok(),
        wsl_has("rustc") || which("rustc").is_ok(),
    );

    #[cfg(not(windows))]
    let (tmux, node, rust) = (
        which("tmux").is_ok(),
        which("node").is_ok(),
        which("rustc").is_ok(),
    );

    let hi = if let Some(ref path) = hi_path {
        PathBuf::from(path).exists()
    } else {
        which("hi").is_ok()
    };

    let hi_monitor = if let Some(ref path) = monitor_path {
        PathBuf::from(path).exists()
    } else {
        which("hi-monitor").is_ok()
    };

    Ok(SetupStatus {
        tmux,
        node,
        rust,
        hi,
        hi_monitor,
    })
}

#[tauri::command]
pub async fn install_dependency(name: String, window: Window) -> Result<(), String> {
    let cmd_str = match name.as_str() {
        "tmux" => {
            #[cfg(target_os = "macos")]
            { "brew install tmux" }
            #[cfg(any(target_os = "linux", windows))]
            { "sudo apt install -y tmux" }
            #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
            { return Err("tmux installation not supported on this platform".to_string()); }
        }
        "node" => {
            "curl -fsSL https://fnm.vercel.app/install | bash && source ~/.bashrc && fnm install 20 && fnm use 20"
        }
        "rust" => {
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
        }
        _ => return Err(format!("Unknown dependency: {}", name)),
    };
    
    #[cfg(unix)]
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // On Windows: prefer WSL, fall back to cmd
    #[cfg(windows)]
    let mut child = if wsl_has("bash") {
        std::process::Command::new("wsl")
            .args(["--", "bash", "-ic", cmd_str])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?
    } else {
        std::process::Command::new("cmd")
            .args(["/C", cmd_str])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?
    };
    
    use std::io::{BufRead, BufReader};
    
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = window.emit("setup://install-log", json!({ "name": name.clone(), "line": line }));
            }
        }
    }
    
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                let _ = window.emit("setup://install-log", json!({ "name": name.clone(), "line": line }));
            }
        }
    }
    
    let status = child.wait().map_err(|e| e.to_string())?;
    
    if status.success() {
        Ok(())
    } else {
        Err(format!("Installation failed for {}", name))
    }
}

pub fn auto_install_cli(app: &AppHandle) {
    let resource_dir = match app.path().resource_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };

    #[cfg(windows)]
    let (hi_src, monitor_src, install_dir) = {
        let home = std::env::var("LOCALAPPDATA").unwrap_or_default();
        (
            resource_dir.join("cli/hi.exe"),
            resource_dir.join("cli/hi-monitor.exe"),
            PathBuf::from(&home).join("hione").join("bin"),
        )
    };

    #[cfg(not(windows))]
    let (hi_src, monitor_src, install_dir) = {
        let home = std::env::var("HOME").unwrap_or_default();
        (
            resource_dir.join("cli/hi"),
            resource_dir.join("cli/hi-monitor"),
            PathBuf::from(&home).join(".local/bin"),
        )
    };

    if !hi_src.exists() {
        return;
    }

    let hi_dest = install_dir.join(if cfg!(windows) { "hi.exe" } else { "hi" });
    let monitor_dest = install_dir.join(if cfg!(windows) { "hi-monitor.exe" } else { "hi-monitor" });

    if hi_dest.exists() {
        return;
    }

    let _ = std::fs::create_dir_all(&install_dir);

    if let Err(e) = std::fs::copy(&hi_src, &hi_dest) {
        eprintln!("Failed to install hi: {}", e);
        return;
    }
    if let Err(e) = std::fs::copy(&monitor_src, &monitor_dest) {
        eprintln!("Failed to install hi-monitor: {}", e);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&hi_dest, std::fs::Permissions::from_mode(0o755));
        let _ = std::fs::set_permissions(&monitor_dest, std::fs::Permissions::from_mode(0o755));
    }

    #[cfg(windows)]
    {
        let dir_str = install_dir.to_string_lossy().to_string();
        let ps_script = format!(
            "$p = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
             if ($p -notlike '*{d}*') {{ \
                 [Environment]::SetEnvironmentVariable('PATH', \"$p;{d}\", 'User'); \
                 Write-Host 'Added {d} to PATH' \
             }}",
            d = dir_str
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .status();
    }

    println!("CLI tools installed to {:?}", install_dir);
}

#[tauri::command]
pub async fn install_bundled_cli(app: AppHandle) -> Result<String, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;

    #[cfg(windows)]
    let (hi_src, monitor_src, install_dir) = {
        let home = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
        (
            resource_dir.join("cli/hi.exe"),
            resource_dir.join("cli/hi-monitor.exe"),
            PathBuf::from(&home).join("hione").join("bin"),
        )
    };

    #[cfg(not(windows))]
    let (hi_src, monitor_src, install_dir) = {
        let home = std::env::var("HOME").map_err(|e| e.to_string())?;
        (
            resource_dir.join("cli/hi"),
            resource_dir.join("cli/hi-monitor"),
            PathBuf::from(&home).join(".local/bin"),
        )
    };

    if !hi_src.exists() {
        return Err("CLI binaries not bundled in this build".to_string());
    }

    std::fs::create_dir_all(&install_dir).map_err(|e| e.to_string())?;

    let hi_dest = install_dir.join(if cfg!(windows) { "hi.exe" } else { "hi" });
    let monitor_dest = install_dir.join(if cfg!(windows) { "hi-monitor.exe" } else { "hi-monitor" });

    std::fs::copy(&hi_src, &hi_dest).map_err(|e| format!("Failed to copy hi: {}", e))?;
    std::fs::copy(&monitor_src, &monitor_dest).map_err(|e| format!("Failed to copy hi-monitor: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hi_dest, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
        std::fs::set_permissions(&monitor_dest, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    #[cfg(windows)]
    {
        let dir_str = install_dir.to_string_lossy().to_string();
        let ps_script = format!(
            "$p = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
             if ($p -notlike '*{d}*') {{ \
                 [Environment]::SetEnvironmentVariable('PATH', \"$p;{d}\", 'User'); \
                 Write-Host 'Added {d} to PATH' \
             }}",
            d = dir_str
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .status()
            .map_err(|e| format!("Failed to update PATH: {}", e))?;
    }

    Ok(format!("CLI tools installed to {}. Restart terminal to use 'hi' command.", install_dir.display()))
}