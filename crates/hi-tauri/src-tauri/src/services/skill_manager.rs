use sqlx::SqlitePool;
use sqlx::Row;
use crate::types::{Skill, SkillRepo};
use crate::error::AppError;
use std::path::PathBuf;

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub async fn get_all_skills(pool: &SqlitePool) -> Result<Vec<Skill>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, repo_url, local_path, enabled_for, installed_at FROM skills ORDER BY name"
    )
    .fetch_all(pool)
    .await?;
    
    let skills: Vec<Skill> = rows
        .into_iter()
        .map(|row| {
            let enabled_json: String = row.try_get::<String, _>("enabled_for").unwrap_or_else(|_| "[]".to_string());
            Skill {
                id: row.try_get::<String, _>("id").unwrap_or_default(),
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                repo_url: row.try_get::<Option<String>, _>("repo_url").ok().flatten(),
                local_path: row.try_get::<Option<String>, _>("local_path").ok().flatten(),
                enabled_for: serde_json::from_str(&enabled_json).unwrap_or_default(),
                installed_at: row.try_get::<i64, _>("installed_at").unwrap_or_default(),
            }
        })
        .collect();
    
    Ok(skills)
}

pub async fn get_all_repos(pool: &SqlitePool) -> Result<Vec<SkillRepo>, AppError> {
    let rows = sqlx::query(
        "SELECT id, url, name FROM skill_repos ORDER BY name"
    )
    .fetch_all(pool)
    .await?;
    
    let repos: Vec<SkillRepo> = rows
        .into_iter()
        .map(|row| {
            SkillRepo {
                id: row.try_get::<String, _>("id").unwrap_or_default(),
                url: row.try_get::<String, _>("url").unwrap_or_default(),
                name: row.try_get::<String, _>("name").unwrap_or_default(),
            }
        })
        .collect();
    
    Ok(repos)
}

pub async fn add_repo(pool: &SqlitePool, url: &str) -> Result<SkillRepo, AppError> {
    let name = url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .to_string();
    
    let repo = SkillRepo {
        id: uuid::Uuid::new_v4().to_string(),
        url: url.to_string(),
        name,
    };
    
    sqlx::query(
        "INSERT INTO skill_repos (id, url, name) VALUES (?, ?, ?)"
    )
    .bind(&repo.id)
    .bind(&repo.url)
    .bind(&repo.name)
    .execute(pool)
    .await?;
    
    Ok(repo)
}

pub async fn remove_repo(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM skills WHERE repo_url IN (SELECT url FROM skill_repos WHERE id = ?)")
        .bind(id)
        .execute(pool)
        .await?;
    
    sqlx::query("DELETE FROM skill_repos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn install_skill(pool: &SqlitePool, repo_id: &str, skill_name: &str) -> Result<(), AppError> {
    let row = sqlx::query(
        "SELECT url FROM skill_repos WHERE id = ?"
    )
    .bind(repo_id)
    .fetch_optional(pool)
    .await?;
    
    let repo_url = match row {
        Some(r) => r.try_get::<String, _>("url").ok(),
        None => None,
    };
    
    let local_path = home_dir()
        .join(".hione")
        .join("skills")
        .join(skill_name)
        .to_string_lossy()
        .to_string();
    
    let now = chrono::Utc::now().timestamp();
    
    sqlx::query(
        "INSERT INTO skills (id, name, repo_url, local_path, enabled_for, installed_at)
         VALUES (?, ?, ?, ?, '[]', ?)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(skill_name)
    .bind(repo_url)
    .bind(local_path)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn delete_skill(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM skills WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(())
}

pub async fn toggle_for_agent(
    pool: &SqlitePool,
    skill_id: &str,
    agent_name: &str,
    enabled: bool,
) -> Result<(), AppError> {
    let row = sqlx::query(
        "SELECT enabled_for FROM skills WHERE id = ?"
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await?;
    
    let row = match row {
        Some(r) => r,
        None => return Err(AppError::Database("Skill not found".to_string())),
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
        "UPDATE skills SET enabled_for = ? WHERE id = ?"
    )
    .bind(&new_enabled_for)
    .bind(skill_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn sync_to_tools(_pool: &SqlitePool) -> Result<(), AppError> {
    Ok(())
}