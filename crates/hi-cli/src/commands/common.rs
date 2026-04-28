use anyhow::{Context, Result};
use hi_core::{ipc::send_message, message::Message, session::SessionInfo};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn load_session() -> Result<SessionInfo> {
    let hione_dir = env::current_dir()?.join(".hione");
    load_session_from(&hione_dir)
}

pub fn load_session_from(hione_dir: &Path) -> Result<SessionInfo> {
    let json = fs::read_to_string(hione_dir.join("session.json"))
        .context(".hione/session.json not found — did you run `hi start`?")?;
    Ok(serde_json::from_str(&json)?)
}

pub fn hione_dir() -> Result<PathBuf> {
    Ok(env::current_dir()?.join(".hione"))
}

pub async fn send_to_monitor(socket_path: &str, msg: &Message) -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(socket_path)
            .await
            .context("Cannot connect to hi-monitor socket")?;
        send_message(&mut stream, msg).await?;
    }
    #[cfg(windows)]
    {
        anyhow::bail!("Windows named pipe not yet implemented");
    }
    Ok(())
}
