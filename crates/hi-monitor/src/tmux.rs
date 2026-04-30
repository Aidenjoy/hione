use std::io::Write;
use std::{io, process::Command, time::Duration};
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
    deliver_to_pane_windows_via_buffer(pane_id, content)
        .or_else(|_| deliver_to_pane_windows_literal(pane_id, content))
}

#[cfg(windows)]
fn deliver_to_pane_windows_via_buffer(pane_id: &str, content: &str) -> io::Result<()> {
    let original_pane = run_mux_output(["display-message", "-p", "#{pane_id}"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut tmp = NamedTempFile::new()?;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;
    let tmp_path = tmp.into_temp_path();
    let tmp_path_str = tmp_path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid temporary file path"))?
        .to_string();

    let delivery = (|| {
        run_mux_command(["load-buffer", &tmp_path_str])?;
        run_mux_command(["select-pane", "-t", pane_id])?;
        run_mux_command(["paste-buffer"])?;
        std::thread::sleep(Duration::from_millis(300));
        run_mux_command(["send-keys", "-t", pane_id, "Enter"])?;
        Ok(())
    })();

    if let Some(original_pane) = original_pane {
        if original_pane != pane_id {
            let _ = run_mux_command(["select-pane", "-t", &original_pane]);
        }
    }

    let _ = run_mux_command(["delete-buffer"]);
    delivery
}

#[cfg(windows)]
fn deliver_to_pane_windows_literal(pane_id: &str, content: &str) -> io::Result<()> {
    run_mux_command(["send-keys", "-t", pane_id, "-l", content])?;
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

#[cfg(windows)]
fn run_mux_output<const N: usize>(args: [&str; N]) -> io::Result<String> {
    let output = Command::new(mux_bin()).args(args).output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
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
fn windows_buffer_delivery_commands(
    pane_id: &str,
    tmp_path: &str,
    original_pane: Option<&str>,
) -> Vec<Vec<String>> {
    let mut commands = vec![
        vec!["display-message".to_string(), "-p".to_string(), "#{pane_id}".to_string()],
        vec!["load-buffer".to_string(), tmp_path.to_string()],
        vec!["select-pane".to_string(), "-t".to_string(), pane_id.to_string()],
        vec!["paste-buffer".to_string()],
        vec!["send-keys".to_string(), "-t".to_string(), pane_id.to_string(), "Enter".to_string()],
    ];
    if let Some(original_pane) = original_pane {
        if original_pane != pane_id {
            commands.push(vec![
                "select-pane".to_string(),
                "-t".to_string(),
                original_pane.to_string(),
            ]);
        }
    }
    commands.push(vec!["delete-buffer".to_string()]);
    commands
}

#[cfg(test)]
fn windows_literal_fallback_commands(pane_id: &str, content: &str) -> Vec<Vec<String>> {
    vec![
        vec![
            "send-keys".to_string(),
            "-t".to_string(),
            pane_id.to_string(),
            "-l".to_string(),
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
    use super::{windows_buffer_delivery_commands, windows_literal_fallback_commands};

    #[test]
    fn windows_delivery_uses_buffer_paste_after_selecting_target() {
        let commands = windows_buffer_delivery_commands("%2", "task.txt", Some("%1"));

        assert_eq!(commands[0], vec!["display-message", "-p", "#{pane_id}"]);
        assert_eq!(commands[1], vec!["load-buffer", "task.txt"]);
        assert_eq!(commands[2], vec!["select-pane", "-t", "%2"]);
        assert_eq!(commands[3], vec!["paste-buffer"]);
        assert_eq!(commands[4], vec!["send-keys", "-t", "%2", "Enter"]);
        assert_eq!(commands[5], vec!["select-pane", "-t", "%1"]);
        assert_eq!(commands[6], vec!["delete-buffer"]);
    }

    #[test]
    fn windows_delivery_fallback_uses_targeted_literal_send_keys() {
        let commands = windows_literal_fallback_commands("%2", "hello\nworld");

        assert_eq!(commands[1], vec!["send-keys", "-t", "%2", "Enter"]);
        assert_eq!(commands[0][0], "send-keys");
        assert_eq!(commands[0][1], "-t");
        assert_eq!(commands[0][2], "%2");
        assert_eq!(commands[0][3], "-l");
        assert!(commands[0][4].contains("hello\nworld"));
        assert!(!commands[0][4].contains("\x1b[200~"));
    }
}
