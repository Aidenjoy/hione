use uuid::Uuid;

pub fn format_task_envelope(
    id: &Uuid,
    from: &str,
    to: &str,
    content: &str,
    peers: &[String],
) -> String {
    let collaboration = if peers.is_empty() {
        String::new()
    } else {
        let peer_list = peers.join(", ");
        format!(
            "\n\nCOLLABORATION:\n- Peers: {peer_list}\n- Delegate: hi push <peer> \"<task>\"  (one task per peer, wait Task DONE before next)\n- Details: .hione/CONTEXT.md"
        )
    };
    format!(
        "Task ID: {id}\nFrom: {from}\nTo: {to}\n\n{content}\n\nIMPORTANT:\n- Reply in English.\n- Provide ONLY the final answer or result. No reasoning, no explanation, no preamble.\n- End your reply with this exact line (verbatim, on its own line):\nTask DONE: {id}{collaboration}"
    )
}

pub fn format_result_envelope(id: &Uuid, from: &str, to: &str, content: &str) -> String {
    format!("Task DONE: {id}\nFrom: {from}\nTo: {to}\n\nResult:\n{content}")
}

pub fn extract_result(snapshot: &str, task_id: &Uuid) -> Option<String> {
    let done_marker = format!("Task DONE: {task_id}");
    let start_marker = format!("Task ID: {task_id}");

    // 始终找最后一个 done marker —— 即 AI 实际写下的完成标志
    let last_done_pos = snapshot.rfind(&done_marker)?;

    let response_start = if let Some(task_pos) = snapshot[..last_done_pos].rfind(&start_marker) {
        // task header 还在屏幕内：跳过 header 中的示例 done marker，取两者之间的内容
        let segment = &snapshot[task_pos..last_done_pos];
        let first_done_rel = segment.find(&done_marker)?;
        task_pos + first_done_rel + done_marker.len()
    } else {
        // task header 已滚出屏幕：最后一个 done marker 之前的内容即为 AI 回复
        0
    };

    let ai_reply = snapshot[response_start..last_done_pos].trim().to_string();
    if ai_reply.is_empty() { None } else { Some(ai_reply) }
}
