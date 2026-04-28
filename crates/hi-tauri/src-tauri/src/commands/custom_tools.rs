use crate::types::CustomTool;

#[tauri::command]
pub fn read_custom_tools(work_dir: String) -> Result<Vec<CustomTool>, String> {
    crate::services::custom_tools::read_tools(&work_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_custom_tools(work_dir: String, tools: Vec<CustomTool>) -> Result<(), String> {
    crate::services::custom_tools::write_tools(&work_dir, &tools)
        .map_err(|e| e.to_string())
}