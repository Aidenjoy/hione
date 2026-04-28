use crate::types::TaskRecord;
use crate::AppState;
use crate::services::{ipc_client, session};
use tauri::State;

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>) -> Result<Vec<TaskRecord>, String> {
    let work_dir = {
        let current = state.current_work_dir.lock().unwrap();
        current.clone()
    };
    
    match work_dir {
        Some(wd) => session::get_task_records(&wd).await.map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn push_task(
    state: State<'_, AppState>,
    target: String,
    content: String,
) -> Result<String, String> {
    let work_dir = {
        let current = state.current_work_dir.lock().unwrap();
        current.clone()
    };
    
    let wd = work_dir.ok_or_else(|| "No active session")?;
    
    let session_info = session::read_session_info(&wd)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found")?;
    
    let client = ipc_client::IpcClient::new(session_info.socket_path);
    
    client
        .push_task("desktop", &target, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let work_dir = {
        let current = state.current_work_dir.lock().unwrap();
        current.clone()
    };
    
    let wd = work_dir.ok_or_else(|| "No active session")?;
    
    let session_info = session::read_session_info(&wd)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found")?;
    
    let client = ipc_client::IpcClient::new(session_info.socket_path);
    
    client
        .cancel_task(&task_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<bool, String> {
    let work_dir = {
        let current = state.current_work_dir.lock().unwrap();
        current.clone()
    };
    
    let wd = work_dir.ok_or_else(|| "No active session")?;
    
    let session_info = session::read_session_info(&wd)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found")?;
    
    let client = ipc_client::IpcClient::new(session_info.socket_path);
    
    client
        .check_agent(&name)
        .await
        .map_err(|e| e.to_string())
}