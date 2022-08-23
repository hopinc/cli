use async_trait::async_trait;
use async_tungstenite::{tokio::ConnectStream, tungstenite::Message, WebSocketStream};
use futures::SinkExt;
use serde_json::{json, to_string, Value};

use crate::errors::{Error, Result};

pub type WsStream = WebSocketStream<ConnectStream>;

#[async_trait]
pub trait SenderExt {
    async fn send_json(&mut self, value: &Value) -> Result<()>;
}

#[async_trait]
impl SenderExt for WsStream {
    async fn send_json(&mut self, value: &Value) -> Result<()> {
        Ok(to_string(value)
            .map(Message::Text)
            .map_err(Error::from)
            .map(|m| self.send(m))?
            .await?)
    }
}

#[async_trait]
pub trait WsStreamExt {
    async fn send_heartbeat(&mut self, tag: Option<&str>) -> Result<()>;
}

#[async_trait]
impl WsStreamExt for WsStream {
    async fn send_heartbeat(&mut self, tag: Option<&str>) -> Result<()> {
        self.send_json(&json!({
            "op": 3,
            "d": {
                "tag": tag,
            },
        }))
        .await?;

        Ok(())
    }
}
