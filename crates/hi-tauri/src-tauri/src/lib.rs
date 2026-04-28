pub mod db;
pub mod commands;
pub mod services;
pub mod error;
pub mod types;

use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Arc<SqlitePool>,
    pub current_work_dir: Mutex<Option<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            crate::commands::setup::auto_install_cli(app.handle());
            
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let pool = db::init_db().await.expect("Failed to init database");
                let state = AppState { 
                    db: Arc::new(pool),
                    current_work_dir: Mutex::new(None),
                };
                app_handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_setup,
            commands::install_dependency,
            commands::install_bundled_cli,
            commands::list_tools,
            commands::install_tool,
            commands::uninstall_tool,
            commands::check_tool_update,
            commands::list_agents,
            commands::save_agent,
            commands::test_agent_connection,
            commands::list_mcp_servers,
            commands::create_mcp_server,
            commands::update_mcp_server,
            commands::delete_mcp_server,
            commands::toggle_mcp_for_agent,
            commands::sync_mcp_to_tools,
            commands::list_skills,
            commands::list_skill_repos,
            commands::add_skill_repo,
            commands::remove_skill_repo,
            commands::install_skill,
            commands::delete_skill,
            commands::toggle_skill_for_agent,
            commands::sync_skills_to_tools,
            commands::list_recent_sessions,
            commands::launch_session,
            commands::connect_session,
            commands::disconnect_session,
            commands::kill_session,
            commands::detect_session,
            commands::list_tasks,
            commands::push_task,
            commands::cancel_task,
            commands::check_agent,
            commands::read_custom_tools,
            commands::write_custom_tools,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}