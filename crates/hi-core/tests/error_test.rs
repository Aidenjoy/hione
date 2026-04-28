use hi_core::error::HiError;

#[test]
fn display_messages() {
    let e = HiError::IpcConnect("socket refused".to_string());
    assert_eq!(format!("{e}"), "IPC connection failed: socket refused");

    let e = HiError::TaskNotFound("xyz".to_string());
    assert_eq!(format!("{e}"), "Task not found: xyz");

    let e = HiError::TargetNotReachable("opencode".to_string());
    assert_eq!(format!("{e}"), "Target not reachable: opencode");
}

#[test]
fn io_conversion() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
    let e: HiError = io.into();
    assert!(format!("{e}").starts_with("IO error:"));
}

#[test]
fn serde_conversion() {
    let err: serde_json::Error = serde_json::from_str::<i32>("not json").unwrap_err();
    let e: HiError = err.into();
    assert!(format!("{e}").starts_with("Serialization error:"));
}
