use crate::{error::HiResult, message::Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 写入一帧：[4字节长度][JSON payload]
pub async fn send_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &Message,
) -> HiResult<()> {
    let payload = serde_json::to_vec(msg)?;
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

/// 读取一帧：[4字节长度][JSON payload]
pub async fn recv_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> HiResult<Message> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    // 防止恶意/损坏数据导致 OOM（上限 16 MB）
    if len > 16 * 1024 * 1024 {
        return Err(crate::error::HiError::IpcConnect(
            format!("Message too large: {len} bytes (max 16 MB)")
        ));
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    let msg = serde_json::from_slice(&payload)?;
    Ok(msg)
}
