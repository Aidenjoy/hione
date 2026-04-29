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
        use interprocess::local_socket::tokio::prelude::LocalSocketStream;
        use interprocess::local_socket::traits::tokio::Stream;
        use interprocess::local_socket::{ToNsName, GenericNamespaced};
        // socket_path is "hione_<hash>" format, extract the name part for to_ns_name
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
