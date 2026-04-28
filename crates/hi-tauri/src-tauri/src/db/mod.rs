pub mod schema;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::PathBuf;
use crate::error::AppError;

pub fn get_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    PathBuf::from(home).join(".hione").join("desktop.db")
}

pub async fn init_db() -> Result<SqlitePool, AppError> {
    let db_path = get_db_path();
    
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    
    schema::create_tables(&pool).await?;
    schema::insert_default_agents(&pool).await?;
    
    Ok(pool)
}