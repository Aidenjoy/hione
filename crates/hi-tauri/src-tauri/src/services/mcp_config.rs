use sqlx::SqlitePool;
use sqlx::Row;
use crate::types::McpServer;
use crate::error::AppError;
use std::path::PathBuf;
use std::fs;

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub async fn get_all_mcp_servers(pool: &SqlitePool) -> Result<Vec<McpServer>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, server_config, enabled_for, created_at FROM mcp_servers ORDER BY name"
    )
    .fetch_all(pool)
    .await?;
    
    let servers: Vec<McpServer> = rows
        .into_iter()
        .map(|row| {
            let config_json: String = row.try_get::<String, _>("server_config").unwrap_or_else(|_| "{}".to_string());
            let enabled_json: String = row.try_get::<String, _>("enabled_for").unwrap_or_else(|_| "[]".to_string());
            
            McpServer {
                id: row.try_get::<String, _>("id").unwrap_or_default(),
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                server_config: serde_json::from_str(&config_json).unwrap_or_else(|_| serde_json::json!({})),
                enabled_for: serde_json::from_str(&enabled_json).unwrap_or_default(),
                created_at: row.try_get::<i64, _>("created_at").unwrap_or_default(),
            }
        })
        .collect();
    
    Ok(servers)
}

pub async fn create_mcp_server(pool: &SqlitePool, server: &McpServer) -> Result<(), AppError> {
    let config_json = serde_json::to_string(&server.server_config)?;
    let enabled_json = serde_json::to_string(&server.enabled_for)?;
    
    sqlx::query(
        "INSERT INTO mcp_servers (id, name, server_config, enabled_for, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&server.id)
    .bind(&server.name)
    .bind(&config_json)
    .bind(&enabled_json)
    .bind(server.created_at)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn update_mcp_server(pool: &SqlitePool, server: &McpServer) -> Result<(), AppError> {
    let config_json = serde_json::to_string(&server.server_config)?;
    let enabled_json = serde_json::to_string(&server.enabled_for)?;
    
    sqlx::query(
        "UPDATE mcp_servers SET name = ?, server_config = ?, enabled_for = ? WHERE id = ?"
    )
    .bind(&server.name)
    .bind(&config_json)
    .bind(&enabled_json)
    .bind(&server.id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn delete_mcp_server(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn toggle_for_agent(
    pool: &SqlitePool,
    server_id: &str,
    agent_name: &str,
    enabled: bool,
) -> Result<(), AppError> {
    let row = sqlx::query(
        "SELECT enabled_for FROM mcp_servers WHERE id = ?"
    )
    .bind(server_id)
    .fetch_optional(pool)
    .await?;
    
    let row = match row {
        Some(r) => r,
        None => return Err(AppError::Database("Server not found".to_string())),
    };
    
    let enabled_for_str = row.try_get::<String, _>("enabled_for").unwrap_or_else(|_| "[]".to_string());
    let mut agents: Vec<String> = serde_json::from_str(&enabled_for_str)?;
    
    if enabled {
        if !agents.contains(&agent_name.to_string()) {
            agents.push(agent_name.to_string());
        }
    } else {
        agents.retain(|a| a != agent_name);
    }
    
    let new_enabled_for = serde_json::to_string(&agents)?;
    
    sqlx::query(
        "UPDATE mcp_servers SET enabled_for = ? WHERE id = ?"
    )
    .bind(&new_enabled_for)
    .bind(server_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn sync_to_tools(pool: &SqlitePool) -> Result<(), AppError> {
    let servers = get_all_mcp_servers(pool).await?;
    
    let mcp_servers_map: serde_json::Map<String, serde_json::Value> = servers
        .iter()
        .filter_map(|s| {
            if s.enabled_for.is_empty() {
                return None;
            }
            
            Some((s.name.clone(), s.server_config.clone()))
        })
        .collect();
    
    let mcp_servers_obj = serde_json::Value::Object(mcp_servers_map);
    
    write_claude_mcp(&mcp_servers_obj)?;
    write_gemini_mcp(&mcp_servers_obj)?;
    write_opencode_mcp(&mcp_servers_obj)?;
    
    Ok(())
}

fn write_claude_mcp(mcp_servers: &serde_json::Value) -> Result<(), AppError> {
    let path = home_dir().join(".claude").join("settings.json");
    write_json_config_merge(&path, "mcpServers", mcp_servers)?;
    Ok(())
}

fn write_gemini_mcp(mcp_servers: &serde_json::Value) -> Result<(), AppError> {
    let path = home_dir().join(".gemini").join("settings.json");
    write_json_config_merge(&path, "mcpServers", mcp_servers)?;
    Ok(())
}

fn write_opencode_mcp(mcp_servers: &serde_json::Value) -> Result<(), AppError> {
    let path = home_dir().join(".config").join("opencode").join("config.json");
    write_json_config_merge(&path, "mcpServers", mcp_servers)?;
    Ok(())
}

fn write_json_config_merge(
    path: &PathBuf,
    key: &str,
    value: &serde_json::Value,
) -> Result<(), AppError> {
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
    merged[key] = value.clone();
    
    let content = serde_json::to_string_pretty(&merged)?;
    fs::write(path, content)?;
    
    Ok(())
}