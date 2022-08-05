use anyhow::Result;

use super::types::WsStream;
use super::HOP_LEAP_EDGE_URL;

pub async fn connect() -> Result<WsStream> {
    let url = HOP_LEAP_EDGE_URL;

    let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
        max_message_size: None,
        max_frame_size: None,
        max_send_queue: None,
        accept_unmasked_frames: false,
    };

    let (stream, _) = tokio_tungstenite::connect_async_with_config(url, Some(config)).await?;

    Ok(stream)
}
