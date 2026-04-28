use crate::{
    error::{HiError, HiResult},
    message::Message,
};
use sqlx::sqlite::SqlitePool;
use std::path::Path;

pub async fn init_db(hione_dir: &Path) -> HiResult<SqlitePool> {
    let db_path = hione_dir.join("hi.db");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&url).await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id          TEXT PRIMARY KEY,
            sender      TEXT NOT NULL,
            receiver    TEXT NOT NULL,
            timestamp   TEXT NOT NULL,
            content     TEXT NOT NULL,
            msg_type    TEXT NOT NULL,
            status      TEXT NOT NULL,
            parent_id   TEXT
        );

        CREATE TABLE IF NOT EXISTS snapshots (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            window_name TEXT NOT NULL,
            content     TEXT NOT NULL,
            captured_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_queue (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id  TEXT NOT NULL,
            target      TEXT NOT NULL,
            queued_at   TEXT NOT NULL,
            position    INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

pub async fn insert_message(pool: &SqlitePool, msg: &Message) -> HiResult<()> {
    let msg_type_str = serde_json::to_string(&msg.msg_type)?;
    let status_str = serde_json::to_string(&msg.status)?;
    sqlx::query(
        r#"INSERT INTO messages (id, sender, receiver, timestamp, content, msg_type, status, parent_id)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
    )
    .bind(msg.id.to_string())
    .bind(&msg.sender)
    .bind(&msg.receiver)
    .bind(msg.timestamp.to_rfc3339())
    .bind(&msg.content)
    .bind(msg_type_str)
    .bind(status_str)
    .bind(msg.parent_id.map(|id| id.to_string()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_message_by_id(pool: &SqlitePool, id: &str) -> HiResult<Option<Message>> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, String, Option<String>)>(
        "SELECT id, sender, receiver, timestamp, content, msg_type, status, parent_id FROM messages WHERE id = ?1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let msg = match row {
        None => return Ok(None),
        Some((id_str, sender, receiver, timestamp_str, content, msg_type_str, status_str, parent_id_str)) => {
            let id = id_str.parse().map_err(|e| HiError::IpcConnect(format!("Invalid UUID in db: {e}")))?;
            let timestamp = timestamp_str.parse().map_err(|e| HiError::IpcConnect(format!("Invalid timestamp in db: {e}")))?;
            let msg_type = serde_json::from_str(&msg_type_str)?;
            let status = serde_json::from_str(&status_str)?;
            let parent_id = match parent_id_str {
                None => None,
                Some(s) => Some(s.parse().map_err(|e| HiError::IpcConnect(format!("Invalid parent_id UUID in db: {e}")))?),
            };
            Message { id, sender, receiver, timestamp, content, msg_type, status, parent_id }
        }
    };
    Ok(Some(msg))
}
