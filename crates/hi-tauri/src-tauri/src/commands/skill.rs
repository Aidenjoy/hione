use crate::types::{Skill, SkillRepo};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, String> {
    crate::services::skill_manager::get_all_skills(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_skill_repos(state: State<'_, AppState>) -> Result<Vec<SkillRepo>, String> {
    crate::services::skill_manager::get_all_repos(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_skill_repo(state: State<'_, AppState>, url: String) -> Result<SkillRepo, String> {
    crate::services::skill_manager::add_repo(&state.db, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_skill_repo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    crate::services::skill_manager::remove_repo(&state.db, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_skill(
    state: State<'_, AppState>,
    repo_id: String,
    skill_name: String,
) -> Result<(), String> {
    crate::services::skill_manager::install_skill(&state.db, &repo_id, &skill_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill(state: State<'_, AppState>, id: String) -> Result<(), String> {
    crate::services::skill_manager::delete_skill(&state.db, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_skill_for_agent(
    state: State<'_, AppState>,
    skill_id: String,
    agent_name: String,
    enabled: bool,
) -> Result<(), String> {
    crate::services::skill_manager::toggle_for_agent(&state.db, &skill_id, &agent_name, enabled)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_skills_to_tools(state: State<'_, AppState>) -> Result<(), String> {
    crate::services::skill_manager::sync_to_tools(&state.db)
        .await
        .map_err(|e| e.to_string())
}