use crate::types::Agent;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    crate::services::agent_config::get_all_agents(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_agent(state: State<'_, AppState>, agent: Agent) -> Result<(), String> {
    crate::services::agent_config::upsert_agent(&state.db, &agent)
        .await
        .map_err(|e| e.to_string())?;
    
    crate::services::agent_config::write_tool_config(&agent)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn test_agent_connection(agent: Agent) -> Result<bool, String> {
    crate::services::agent_config::test_connection(&agent)
        .await
        .map_err(|e| e.to_string())
}