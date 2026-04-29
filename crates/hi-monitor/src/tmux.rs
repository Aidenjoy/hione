use std::{io, process::Command, time::Duration};
use tempfile::NamedTempFile;
use std::io::Write;

/// Get the multiplexer binary name for the current platform
fn mux_bin() -> &'static str {
    if cfg!(windows) { "psmux" } else { "tmux" }
}

pub fn deliver_to_pane(pane_id: &str, content: &str) -> io::Result<()> {
    // 1. 包装 Bracketed Paste
    let bracketed = format!("\x1b[200~{content}\x1b[201~");

    // 2. 使用 NamedTempFile 自动管理临时文件生命周期，避免泄漏
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(bracketed.as_bytes())?;
    let tmp_path = tmp.path().to_str().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Invalid temporary file path")
    })?;

    // 3. 将内容加载到 mux buffer
    Command::new(mux_bin())
        .args(["load-buffer", tmp_path])
        .status()?;

    // 4. 将 buffer 内容粘贴到目标 pane
    Command::new(mux_bin())
        .args(["paste-buffer", "-t", pane_id])
        .status()?;

    // 5. 稍微延迟后发送回车，触发 AI 处理内容
    std::thread::sleep(Duration::from_millis(300));
    Command::new(mux_bin())
        .args(["send-keys", "-t", pane_id, "Enter"])
        .status()?;

    Ok(())
}
