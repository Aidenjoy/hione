use crate::types::ToolInfo;
use tauri::Window;

#[tauri::command]
pub async fn list_tools() -> Result<Vec<ToolInfo>, String> {
    Ok(crate::services::tool_manager::detect_all_tools())
}

#[tauri::command]
pub async fn install_tool(name: String, window: Window) -> Result<(), String> {
    crate::services::tool_manager::install_tool_async(name, window)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_tool(name: String, window: Window) -> Result<(), String> {
    crate::services::tool_manager::uninstall_tool_async(name, window)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_tool_update(name: String) -> Result<Option<String>, String> {
    Ok(None)
}