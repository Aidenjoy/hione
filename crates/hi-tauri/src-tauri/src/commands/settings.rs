use crate::types::AppSettings;
use std::path::PathBuf;
use std::fs;

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    home.join(".hione").join("desktop_settings.json")
}

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    let path = settings_path();
    
    if !path.exists() {
        return Ok(AppSettings {
            language: "zh-CN".to_string(),
            theme: "system".to_string(),
            hi_bin_path: None,
            hi_monitor_bin_path: None,
        });
    }
    
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let settings: AppSettings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = settings_path();
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}