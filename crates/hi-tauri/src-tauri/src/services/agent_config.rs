use sqlx::{SqlitePool, Row};
use crate::types::Agent;
use crate::error::AppError;
use std::path::PathBuf;
use std::fs;

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub async fn get_all_agents(pool: &SqlitePool) -> Result<Vec<Agent>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, api_key, api_base_url, model, extra_config, enabled, created_at, updated_at FROM agents ORDER BY name"
    )
    .fetch_all(pool)
    .await?;
    
    let agents: Vec<Agent> = rows
        .into_iter()
        .map(|row| {
            let extra_json: String = row.try_get::<String, _>("extra_config").unwrap_or_else(|_| "{}".to_string());
            Agent {
                id: row.try_get::<String, _>("id").unwrap_or_default(),
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                api_key: row.try_get::<Option<String>, _>("api_key").ok().flatten(),
                api_base_url: row.try_get::<Option<String>, _>("api_base_url").ok().flatten(),
                model: row.try_get::<Option<String>, _>("model").ok().flatten(),
                extra_config: serde_json::from_str(&extra_json).unwrap_or_else(|_| serde_json::json!({})),
                enabled: row.try_get::<bool, _>("enabled").unwrap_or(true),
                created_at: row.try_get::<i64, _>("created_at").unwrap_or_default(),
                updated_at: row.try_get::<i64, _>("updated_at").unwrap_or_default(),
            }
        })
        .collect();
    
    Ok(agents)
}

pub async fn upsert_agent(pool: &SqlitePool, agent: &Agent) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    let extra_json = serde_json::to_string(&agent.extra_config)?;
    
    sqlx::query(
        "INSERT INTO agents (id, name, api_key, api_base_url, model, extra_config, enabled, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(name) DO UPDATE SET
           api_key = excluded.api_key,
           api_base_url = excluded.api_base_url,
           model = excluded.model,
           extra_config = excluded.extra_config,
           enabled = excluded.enabled,
           updated_at = excluded.updated_at"
    )
    .bind(&agent.id)
    .bind(&agent.name)
    .bind(&agent.api_key)
    .bind(&agent.api_base_url)
    .bind(&agent.model)
    .bind(&extra_json)
    .bind(agent.enabled)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub fn write_tool_config(agent: &Agent) -> Result<(), AppError> {
    let name = agent.name.to_lowercase();
    
    match name.as_str() {
        "claude" => write_claude_config(agent),
        "gemini" => write_gemini_config(agent),
        "opencode" => write_opencode_config(agent),
        "codex" => write_codex_config(agent),
        "qwen" => write_qwen_config(agent),
        _ => Ok(()),
    }
}

fn write_claude_config(agent: &Agent) -> Result<(), AppError> {
    let path = home_dir().join(".claude").join("settings.json");
    write_json_config_merge(&path, |existing| {
        if let Some(key) = &agent.api_key {
            existing["apiKey"] = serde_json::Value::String(key.clone());
        }
        if let Some(url) = &agent.api_base_url {
            existing["apiBaseUrl"] = serde_json::Value::String(url.clone());
        }
    })
}

fn write_gemini_config(agent: &Agent) -> Result<(), AppError> {
    let path = home_dir().join(".gemini").join("settings.json");
    write_json_config_merge(&path, |existing| {
        if let Some(key) = &agent.api_key {
            existing["apiKey"] = serde_json::Value::String(key.clone());
        }
    })
}

fn write_opencode_config(agent: &Agent) -> Result<(), AppError> {
    let path = home_dir().join(".config").join("opencode").join("config.json");
    write_json_config_merge(&path, |existing| {
        let mut openai = existing.get("openai").cloned().unwrap_or(serde_json::json!({}));
        if let Some(key) = &agent.api_key {
            openai["apiKey"] = serde_json::Value::String(key.clone());
        }
        if let Some(url) = &agent.api_base_url {
            openai["baseUrl"] = serde_json::Value::String(url.clone());
        }
        existing["openai"] = openai;
    })
}

fn write_codex_config(agent: &Agent) -> Result<(), AppError> {
    let path = home_dir().join(".codex").join("config.toml");
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut toml_content = String::new();
    if let Some(key) = &agent.api_key {
        toml_content.push_str(&format!("api_key = \"{}\"\n", key));
    }
    if let Some(model) = &agent.model {
        toml_content.push_str(&format!("model = \"{}\"\n", model));
    }
    
    fs::write(&path, toml_content)?;
    Ok(())
}

fn write_qwen_config(agent: &Agent) -> Result<(), AppError> {
    let path = home_dir().join(".qwen").join("settings.json");
    write_json_config_merge(&path, |existing| {
        if let Some(key) = &agent.api_key {
            existing["apiKey"] = serde_json::Value::String(key.clone());
        }
        if let Some(model) = &agent.model {
            existing["model"] = serde_json::Value::String(model.clone());
        }
    })
}

fn write_json_config_merge<F>(path: &PathBuf, merge_fn: F) -> Result<(), AppError>
where
    F: FnOnce(&mut serde_json::Value),
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let existing: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    let mut merged = existing;
    merge_fn(&mut merged);
    
    let content = serde_json::to_string_pretty(&merged)?;
    fs::write(path, content)?;
    
    Ok(())
}

pub async fn test_connection(agent: &Agent) -> Result<bool, AppError> {
    let base_url = agent.api_base_url.as_ref().ok_or_else(|| {
        AppError::CommandFailed("No api_base_url configured".to_string())
    })?;
    
    let api_key = agent.api_key.as_ref().ok_or_else(|| {
        AppError::CommandFailed("No api_key configured".to_string())
    })?;
    
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;
    
    match response {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}