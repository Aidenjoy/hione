use anyhow::{Context, Result};
use hi_core::{ipc::send_message, message::Message, session::SessionInfo};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn load_session() -> Result<SessionInfo> {
    let hione_dir = find_hione_dir()?;
    load_session_from(&hione_dir)
}

pub fn load_session_from(hione_dir: &Path) -> Result<SessionInfo> {
    let json = fs::read_to_string(hione_dir.join("session.json"))
        .context(format!(
            ".hione/session.json not found in {} or parents — did you run `hi start`?",
            env::current_dir()?.display()
        ))?;
    Ok(serde_json::from_str(&json)?)
}

pub fn hione_dir() -> Result<PathBuf> {
    find_hione_dir()
}

pub fn find_hione_dir() -> Result<PathBuf> {
    let mut curr = env::current_dir()?;
    loop {
        let hione = curr.join(".hione");
        if hione.is_dir() {
            return Ok(hione);
        }
        if !curr.pop() {
            break;
        }
    }
    // 如果没找到，回退到当前目录，让后续 load_session_from 抛出包含详细信息的 context 错误
    Ok(env::current_dir()?.join(".hione"))
}

/// Connect to the monitor and send a message, auto-restarting the monitor
/// if a stale socket/pipe is detected (ECONNREFUSED / pipe not found).
pub async fn send_to_monitor(socket_path: &str, msg: &Message) -> Result<()> {
    // Try once; on stale-socket errors, attempt to revive the monitor and retry.
    for attempt in 0..2u8 {
        match try_send(socket_path, msg).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt == 0 && is_stale_socket_error(&e) => {
                eprintln!("hi-monitor is not responding, attempting to restart…");
                if let Err(restart_err) = restart_monitor(socket_path).await {
                    anyhow::bail!(
                        "hi-monitor is not running and could not be restarted: {restart_err}\n\
                         Please run: hi s <tools>"
                    );
                }
                // Brief pause for the monitor to bind the socket
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn is_stale_socket_error(e: &anyhow::Error) -> bool {
    // anyhow::Error::to_string() only returns the top-level context.
    // We need to check the full error chain to find the underlying OS error.
    for cause in e.chain() {
        let s = cause.to_string();
        // ECONNREFUSED (61 macOS / 111 Linux) = socket exists but nobody listening
        // ENOENT      (2)                     = socket file gone entirely
        // Windows named pipe not found        = OS error 2 on pipe connect
        if s.contains("os error 61")   // ECONNREFUSED macOS
            || s.contains("os error 111") // ECONNREFUSED Linux
            || s.contains("os error 2")   // ENOENT / pipe not found
            || s.contains("Connection refused")
            || s.contains("No such file or directory")
            || s.contains("named pipe")
        {
            return true;
        }
    }
    false
}

async fn restart_monitor(socket_path: &str) -> Result<()> {
    // Derive hione_dir from socket_path (Unix: "/.../.hione/hi.sock")
    #[cfg(unix)]
    let hione_dir = std::path::PathBuf::from(socket_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot derive hione_dir from socket_path"))?;

    #[cfg(windows)]
    let hione_dir = {
        // On Windows socket_path is a pipe name like "hione_<hash>".
        // Read it from session.json in the current directory.
        let session = load_session()?;
        session.hione_dir.clone()
    };

    // Remove stale socket so the new monitor can bind cleanly
    #[cfg(unix)]
    let _ = std::fs::remove_file(socket_path);

    // Locate and spawn hi-monitor
    let monitor_bin = locate_monitor_bin()?;
    std::process::Command::new(&monitor_bin)
        .arg("--hione-dir")
        .arg(&hione_dir)
        .spawn()
        .with_context(|| format!("Failed to spawn hi-monitor at {}", monitor_bin.display()))?;
    Ok(())
}

fn locate_monitor_bin() -> Result<std::path::PathBuf> {
    if let Ok(p) = which::which("hi-monitor") {
        return Ok(p);
    }
    let exe = std::env::current_exe()?;
    let parent = exe.parent().ok_or_else(|| anyhow::anyhow!("no exe parent"))?;
    #[cfg(windows)]
    {
        let p = parent.join("hi-monitor.exe");
        if p.exists() { return Ok(p); }
    }
    let p = parent.join("hi-monitor");
    if p.exists() { return Ok(p); }
    anyhow::bail!("hi-monitor not found in PATH or alongside hi binary")
}

async fn try_send(socket_path: &str, msg: &Message) -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path)
            .await
            .with_context(|| format!("Cannot connect to hi-monitor socket ({socket_path})"))?;
        send_message(&mut stream, msg).await?;
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::tokio::prelude::LocalSocketStream;
        use interprocess::local_socket::traits::tokio::Stream;
        use interprocess::local_socket::{ToNsName, GenericNamespaced};
        let pipe_name = socket_path.split('\\').last().unwrap_or(socket_path);
        let name = pipe_name.to_ns_name::<GenericNamespaced>()
            .context("Failed to create named pipe name")?;
        let mut stream = LocalSocketStream::connect(name)
            .await
            .context("Cannot connect to hi-monitor named pipe")?;
        send_message(&mut stream, msg).await?;
    }
    Ok(())
}
