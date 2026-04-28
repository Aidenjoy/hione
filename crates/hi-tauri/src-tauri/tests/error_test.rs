use hi_tauri_lib::error::AppError;

#[test]
fn error_display_messages() {
    let e = AppError::Database("connection failed".to_string());
    assert!(format!("{e}").contains("Database error"));

    let e = AppError::HiNotFound;
    assert_eq!(format!("{e}"), "Hi not found");

    let e = AppError::SessionNotFound;
    assert_eq!(format!("{e}"), "Session not found");

    let e = AppError::CommandFailed("install failed".to_string());
    assert!(format!("{e}").contains("Command failed"));
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_err: AppError = io_err.into();
    assert!(format!("{app_err}").contains("IO error"));
}

#[test]
fn error_from_serde_json() {
    let json_err = serde_json::from_str::<i32>("not json").unwrap_err();
    let app_err: AppError = json_err.into();
    assert!(format!("{app_err}").contains("Serialization error"));
}

#[test]
fn error_to_string_conversion() {
    let e = AppError::IpcError("timeout".to_string());
    let s: String = e.into();
    assert!(s.contains("IPC error"));
}