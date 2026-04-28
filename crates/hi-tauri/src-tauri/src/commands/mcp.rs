use crate::types::McpServer;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServer>, String> {
    crate::services::mcp_config::get_all_mcp_servers(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_mcp_server(state: State<'_, AppState>, server: McpServer) -> Result<(), String> {
    crate::services::mcp_config::create_mcp_server(&state.db, &server)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_mcp_server(state: State<'_, AppState>, server: McpServer) -> Result<(), String> {
    crate::services::mcp_config::update_mcp_server(&state.db, &server)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_mcp_server(state: State<'_, AppState>, id: String) -> Result<(), String> {
    crate::services::mcp_config::delete_mcp_server(&state.db, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_mcp_for_agent(
    state: State<'_, AppState>,
    server_id: String,
    agent_name: String,
    enabled: bool,
) -> Result<(), String> {
    crate::services::mcp_config::toggle_for_agent(&state.db, &server_id, &agent_name, enabled)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_mcp_to_tools(state: State<'_, AppState>) -> Result<(), String> {
    crate::services::mcp_config::sync_to_tools(&state.db)
        .await
        .map_err(|e| e.to_string())
}