use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn notify_task_completed(app: &AppHandle, task_id: &str, receiver: &str, result_preview: &str) {
    let body = format!(
        "任务完成：{} 已完成任务\n预览：{}",
        receiver,
        if result_preview.len() > 50 {
            format!("{}...", &result_preview[..50])
        } else {
            result_preview.to_string()
        }
    );
    
    let _ = app
        .notification()
        .builder()
        .title("hione")
        .body(&body)
        .show();
}

pub fn notify_task_timeout(app: &AppHandle, task_id: &str, receiver: &str) {
    let body = format!("任务超时：{} 未在规定时间内响应", receiver);
    
    let _ = app
        .notification()
        .builder()
        .title("hione")
        .body(&body)
        .show();
}