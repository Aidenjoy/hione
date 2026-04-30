#[cfg(not(windows))]
use std::io::Write;
use std::{io, process::Command, time::Duration};
#[cfg(not(windows))]
use tempfile::NamedTempFile;

/// Get the multiplexer binary name for the current platform
fn mux_bin() -> &'static str {
    if cfg!(windows) {
        "psmux"
    } else {
        "tmux"
    }
}

pub fn deliver_to_pane(pane_id: &str, content: &str) -> io::Result<()> {
    let bracketed = bracketed_paste(content);

    #[cfg(windows)]
    {
        deliver_to_pane_windows(pane_id, content)
    }

    #[cfg(not(windows))]
    {
        deliver_to_pane_unix(pane_id, &bracketed)
    }
}

fn bracketed_paste(content: &str) -> String {
    format!("\x1b[200~{content}\x1b[201~")
}

#[cfg(not(windows))]
fn deliver_to_pane_unix(pane_id: &str, bracketed: &str) -> io::Result<()> {
    // 1. 使用 NamedTempFile 自动管理临时文件生命周期，避免泄漏
    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(bracketed.as_bytes())?;
    let tmp_path = tmp
        .path()
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid temporary file path"))?;

    // 2. 将内容加载到 tmux buffer
    run_mux_command(["load-buffer", tmp_path])?;

    // 3. 将 buffer 内容粘贴到目标 pane
    run_mux_command(["paste-buffer", "-t", pane_id])?;

    // 4. 稍微延迟后发送回车，触发 AI 处理内容
    std::thread::sleep(Duration::from_millis(300));
    run_mux_command(["send-keys", "-t", pane_id, "Enter"])?;

    Ok(())
}

#[cfg(windows)]
fn deliver_to_pane_windows(pane_id: &str, content: &str) -> io::Result<()> {
    // psmux documents -p as paste text. That fits hi's multi-line task envelope
    // better than literal key input.
    run_mux_command(["send-keys", "-p", "-t", pane_id, content])?;

    std::thread::sleep(Duration::from_millis(300));
    run_mux_command(["send-keys", "-t", pane_id, "Enter"])?;

    Ok(())
}

fn run_mux_command<const N: usize>(args: [&str; N]) -> io::Result<()> {
    let output = Command::new(mux_bin()).args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("{} exited with {}", mux_bin(), output.status)
    };
    Err(io::Error::new(io::ErrorKind::Other, details))
}

#[cfg(test)]
fn windows_delivery_commands(pane_id: &str, content: &str) -> Vec<Vec<String>> {
    vec![
        vec![
            "send-keys".to_string(),
            "-p".to_string(),
            "-t".to_string(),
            pane_id.to_string(),
            content.to_string(),
        ],
        vec![
            "send-keys".to_string(),
            "-t".to_string(),
            pane_id.to_string(),
            "Enter".to_string(),
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::windows_delivery_commands;

    #[test]
    fn windows_delivery_uses_targeted_paste_send_keys() {
        let commands = windows_delivery_commands("%2", "hello\nworld");

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0][0], "send-keys");
        assert_eq!(commands[0][1], "-p");
        assert_eq!(commands[0][2], "-t");
        assert_eq!(commands[0][3], "%2");
        assert!(commands[0][4].contains("hello\nworld"));
        assert!(!commands[0][4].contains("\x1b[200~"));

        assert_eq!(commands[1], vec!["send-keys", "-t", "%2", "Enter"]);
    }
}
