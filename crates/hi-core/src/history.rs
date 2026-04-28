use std::fs;
use std::path::{Path, PathBuf};
use sqlx::Row;

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

pub async fn read_latest_response(tool: &str, cwd: &Path) -> Option<String> {
    let tool_lower = tool.to_lowercase();
    let cwd_buf = cwd.to_path_buf();
    
    match tool_lower.as_str() {
        "claude" | "claude-code" => {
            tokio::task::spawn_blocking(move || read_claude_code_history(&cwd_buf)).await.ok().flatten()
        }
        "codex" => {
            tokio::task::spawn_blocking(move || read_codex_history(&cwd_buf)).await.ok().flatten()
        }
        "gemini" => {
            tokio::task::spawn_blocking(move || read_gemini_history(&cwd_buf)).await.ok().flatten()
        }
        "opencode" => {
            read_opencode_history(cwd).await
        }
        "qwen" => {
            tokio::task::spawn_blocking(move || read_qwen_history(&cwd_buf)).await.ok().flatten()
        }
        _ => None,
    }
}

fn encode_cwd_for_claude(cwd: &Path) -> Option<String> {
    let abs = cwd.canonicalize().ok()?;
    let path_str = abs.to_string_lossy();
    // /path/to/project -> -path-to-project
    // 经确认为 Claude Code 实际采用的编码方式（带前导 -）
    Some(path_str.replace('/', "-"))
}

fn read_claude_code_history(cwd: &Path) -> Option<String> {
    let home = home_dir()?;
    let encoded = encode_cwd_for_claude(cwd)?;
    let projects_dir = home.join(".claude").join("projects").join(&encoded);
    
    if !projects_dir.exists() {
        return None;
    }
    
    // 仅选取 .jsonl 文件，避免解析目录或其他文件
    let jsonl_files: Vec<_> = fs::read_dir(&projects_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() && path.extension().map(|ext| ext == "jsonl").unwrap_or(false)
        })
        .collect();
    
    let latest_file = jsonl_files
        .into_iter()
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())?;
    
    let content = fs::read_to_string(latest_file.path()).ok()?;
    
    let texts: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let entry: serde_json::Value = serde_json::from_str(line).ok()?;
            if entry.get("type")?.as_str()? != "assistant" {
                return None;
            }
            let message = entry.get("message")?;
            let content_arr = message.get("content")?.as_array()?;
            
            // 拼接同一消息中的所有 text 部分，忽略 thinking 等
            let msg_text = content_arr
                .iter()
                .filter_map(|c| {
                    if c.get("type")?.as_str()? == "text" {
                        c.get("text")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("");
            
            if msg_text.is_empty() {
                None
            } else {
                Some(msg_text)
            }
        })
        .collect();
    
    if texts.is_empty() {
        None
    } else {
        // 使用双换行符分隔不同轮次的回复，模拟对话历史
        Some(texts.join("\n\n"))
    }
}

/// 优化：优先进入最新的日期子目录（YYYY/MM/DD），避免全量递归扫描
fn find_latest_codex_session_file(sessions_dir: &Path) -> Option<PathBuf> {
    let mut current = sessions_dir.to_path_buf();
    
    // 尝试深入三层：YYYY, MM, DD
    for _ in 0..3 {
        let mut subdirs: Vec<_> = fs::read_dir(&current)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        
        if subdirs.is_empty() {
            return None;
        }
        
        subdirs.sort_by_key(|e| e.file_name());
        if let Some(latest_subdir) = subdirs.pop() {
            current = latest_subdir.path();
        } else {
            return None;
        }
    }
    
    let mut files: Vec<_> = fs::read_dir(&current)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() && path.extension().map(|ext| ext == "jsonl").unwrap_or(false)
        })
        .collect();
    
    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    files.pop().map(|e| e.path())
}

fn read_codex_history(cwd: &Path) -> Option<String> {
    let home = home_dir()?;
    let sessions_dir = home.join(".codex").join("sessions");
    
    if !sessions_dir.exists() {
        return None;
    }
    
    let cwd_str = cwd.canonicalize().ok()?.to_string_lossy().to_string();
    
    let latest_file = find_latest_codex_session_file(&sessions_dir)?;
    let content = fs::read_to_string(&latest_file).ok()?;
    
    let mut cwd_matches = false;
    let texts: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let entry: serde_json::Value = serde_json::from_str(line).ok()?;
            
            // 检查 session_meta 确认是否匹配当前项目
            if entry.get("type")?.as_str()? == "session_meta" {
                let payload = entry.get("payload")?;
                if payload.get("cwd")?.as_str()? == cwd_str {
                    cwd_matches = true;
                }
                return None;
            }
            
            if !cwd_matches {
                return None;
            }
            
            if entry.get("type")?.as_str()? != "response_item" {
                return None;
            }
            
            let payload = entry.get("payload")?;
            if payload.get("role")?.as_str()? != "assistant" {
                return None;
            }
            
            let content_arr = payload.get("content")?.as_array()?;
            let msg_text = content_arr
                .iter()
                .filter_map(|c| {
                    if c.get("type")?.as_str()? == "output_text" {
                        c.get("text")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join("");
            
            if msg_text.is_empty() {
                None
            } else {
                Some(msg_text)
            }
        })
        .collect();
    
    if !cwd_matches || texts.is_empty() {
        None
    } else {
        Some(texts.join("\n\n"))
    }
}

fn read_gemini_history(cwd: &Path) -> Option<String> {
    let home = home_dir()?;
    let tmp_dir = home.join(".gemini").join("tmp");
    
    if !tmp_dir.exists() {
        return None;
    }
    
    let cwd_str = cwd.canonicalize().ok()?.to_string_lossy().to_string();
    
    // 通过 .project_root 文件内容定位项目目录
    let project_dir = fs::read_dir(&tmp_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| {
            let root_file = e.path().join(".project_root");
            if root_file.exists() {
                fs::read_to_string(&root_file)
                    .ok()
                    .map(|content| content.trim() == cwd_str)
                    .unwrap_or(false)
            } else {
                false
            }
        })?;
    
    let chats_dir = project_dir.path().join("chats");
    if !chats_dir.exists() {
        return None;
    }
    
    let session_files: Vec<_> = fs::read_dir(&chats_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            let n = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            path.is_file() && n.starts_with("session-") && n.ends_with(".json")
        })
        .collect();
    
    let latest_file = session_files
        .into_iter()
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())?;
    
    let content = fs::read_to_string(latest_file.path()).ok()?;
    let session: serde_json::Value = serde_json::from_str(&content).ok()?;
    
    let messages = session.get("messages")?.as_array()?;
    let texts: Vec<String> = messages
        .iter()
        .filter_map(|m| {
            if m.get("type")?.as_str()? == "gemini" {
                m.get("content")?.as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    
    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n\n"))
    }
}

fn read_qwen_history(_cwd: &Path) -> Option<String> {
    let home = home_dir()?;
    let tmp_dir = home.join(".qwen").join("tmp");
    if !tmp_dir.exists() {
        return None;
    }

    let latest_dir = fs::read_dir(&tmp_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().join("logs.json").exists())
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())?;

    let logs_path = latest_dir.path().join("logs.json");
    let content = fs::read_to_string(&logs_path).ok()?;
    let entries: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = entries.as_array()?;

    let texts: Vec<String> = arr
        .iter()
        .filter_map(|e| {
            let t = e.get("type")?.as_str()?;
            if t == "user" {
                return None;
            }
            e.get("message")?.as_str().map(|s| s.to_string())
        })
        .collect();

    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n\n"))
    }
}

async fn read_opencode_history(cwd: &Path) -> Option<String> {
    let home = home_dir()?;
    let db_path = home.join(".local").join("share").join("opencode").join("opencode.db");
    
    if !db_path.exists() {
        return None;
    }
    
    let cwd_str = cwd.canonicalize().ok()?.to_string_lossy().to_string();
    
    // 使用 sqlite:// 前缀并开启只读模式
    let db_url = format!("sqlite://{}?mode=ro", db_path.display());
    
    // 建立单连接池
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .ok()?;
    
    // 执行查询并确保及时关闭连接
    let result = (|| async {
        // 根据 directory 定位最新 session
        let session_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM session WHERE directory = ? ORDER BY time_created DESC LIMIT 1"
        )
        .bind(&cwd_str)
        .fetch_optional(&pool)
        .await
        .ok()?;
        
        let session_id = session_id?;
        
        // 提取 assistant 角色的 text 内容
        let texts: Vec<String> = sqlx::query(
            "SELECT json_extract(p.data, '$.text') FROM part p \
             JOIN message m ON p.message_id = m.id \
             WHERE m.session_id = ? \
               AND json_extract(m.data, '$.role') = 'assistant' \
               AND json_extract(p.data, '$.type') = 'text' \
             ORDER BY p.time_created ASC"
        )
        .bind(&session_id)
        .fetch_all(&pool)
        .await
        .ok()?
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>(0).ok())
        .collect();
        
        if texts.is_empty() {
            None
        } else {
            Some(texts.join("\n\n"))
        }
    })().await;
    
    pool.close().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_cwd_for_claude_with_existing_path() {
        // Use the current directory which should exist
        let cwd = std::env::current_dir().unwrap();
        let encoded = encode_cwd_for_claude(&cwd).unwrap();
        assert!(encoded.starts_with("-"));
    }

    #[test]
    fn test_encode_cwd_for_claude_format() {
        // Test the format logic directly without needing a real path
        // This tests the string replacement logic: /path/to/project -> -path-to-project
        let path_str = "/path/to/project";
        let expected = "-path-to-project";
        let result = path_str.replace('/', "-");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_claude_jsonl_parsing() {
        let jsonl = r#"{"type":"user","message":{"role":"user","content":"hello"}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello there!"}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"Let me think..."}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Final answer"}]}}
"#;
        let texts: Vec<String> = jsonl
            .lines()
            .filter_map(|line| {
                let entry: serde_json::Value = serde_json::from_str(line).ok()?;
                if entry.get("type")?.as_str()? != "assistant" {
                    return None;
                }
                let message = entry.get("message")?;
                let content_arr = message.get("content")?.as_array()?;
                let msg_text = content_arr
                    .iter()
                    .filter_map(|c| {
                        if c.get("type")?.as_str()? == "text" {
                            c.get("text")?.as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("");
                if msg_text.is_empty() { None } else { Some(msg_text) }
            })
            .collect();
        
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "Hello there!");
        assert_eq!(texts[1], "Final answer");
    }
}
