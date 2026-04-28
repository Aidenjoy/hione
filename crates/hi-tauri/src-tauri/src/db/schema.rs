use sqlx::SqlitePool;
use crate::error::AppError;

pub const SCHEMA_VERSION: i32 = 1;

pub async fn create_tables(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL)"
    )
    .execute(pool)
    .await?;
    
    let version: Option<i32> = sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_optional(pool)
        .await?;
    
    if version.is_none() {
        sqlx::query("INSERT INTO schema_version (version) VALUES (?)")
            .bind(SCHEMA_VERSION)
            .execute(pool)
            .await?;
    }
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agents (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL UNIQUE,
            api_key      TEXT,
            api_base_url TEXT,
            model        TEXT,
            extra_config TEXT NOT NULL DEFAULT '{}',
            enabled      INTEGER NOT NULL DEFAULT 1,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id            TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            server_config TEXT NOT NULL,
            enabled_for   TEXT NOT NULL DEFAULT '[]',
            created_at    INTEGER NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS skills (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            repo_url     TEXT,
            local_path   TEXT,
            enabled_for  TEXT NOT NULL DEFAULT '[]',
            installed_at INTEGER NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS skill_repos (
            id   TEXT PRIMARY KEY,
            url  TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recent_sessions (
            work_dir    TEXT PRIMARY KEY,
            tools       TEXT NOT NULL,
            auto_mode   INTEGER NOT NULL DEFAULT 0,
            resume_mode INTEGER NOT NULL DEFAULT 0,
            last_used   INTEGER NOT NULL
        )"
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn insert_default_agents(pool: &SqlitePool) -> Result<(), AppError> {
    let default_agents = ["claude", "gemini", "opencode", "codex", "qwen"];
    let now = chrono::Utc::now().timestamp();
    
    for name in default_agents {
        let exists: Option<String> = sqlx::query_scalar(
            "SELECT name FROM agents WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        
        if exists.is_none() {
            sqlx::query(
                "INSERT INTO agents (id, name, extra_config, enabled, created_at, updated_at)
                 VALUES (?, ?, '{}', 1, ?, ?)"
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }
    }
    
    Ok(())
}