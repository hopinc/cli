#[cfg(feature = "zlib")]
use std::io::Cursor;

#[cfg(feature = "zlib")]
use async_compression::tokio::bufread::ZlibDecoder;
use async_trait::async_trait;
use async_tungstenite::{tokio::ConnectStream, tungstenite::Message, WebSocketStream};
use futures::{SinkExt, StreamExt};
use serde_json::{json, to_string, Value};
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};

use super::error::Error as LeapError;
use crate::{
    errors::{Error, Result},
    shard::types::OpCode,
};

pub type WsStream = WebSocketStream<ConnectStream>;

#[async_trait]
pub trait RecieverExt {
    async fn recieve_json(&mut self) -> Result<Option<Value>>;
}

#[async_trait]
pub trait SenderExt {
    async fn send_json(&mut self, value: &Value) -> Result<()>;
}

#[async_trait]
impl RecieverExt for WsStream {
    async fn recieve_json(&mut self) -> Result<Option<Value>> {
        const TIMEOUT: tokio::time::Duration = Duration::from_millis(500);

        let message = match timeout(TIMEOUT, self.next()).await {
            Ok(Some(Ok(message))) => Some(message),
            Ok(Some(Err(error))) => return Err(error.into()),
            Ok(None) | Err(_) => None,
        };

        convert_message(message).await
    }
}

pub(crate) async fn convert_message(message: Option<Message>) -> Result<Option<Value>> {
    let converted = match message {
        #[cfg(feature = "zlib")]
        Some(Message::Binary(binary)) => {
            let mut compressed = ZlibDecoder::new(Cursor::new(binary));
            let mut buffer = vec![];

            compressed.read_to_end(&mut buffer).await?;

            serde_json::from_slice(&buffer)?
        }

        Some(Message::Text(text)) => serde_json::from_str(&text)?,

        Some(Message::Close(Some(frame))) => {
            return Err(Error::Leap(LeapError::Closed(Some(frame))));
        }

        _ => None,
    };

    Ok(converted)
}

#[async_trait]
impl SenderExt for WsStream {
    async fn send_json(&mut self, value: &Value) -> Result<()> {
        log::debug!("[Shard] Sending: {value}");

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
    async fn send_identify(&mut self, project: &str, token: Option<&str>) -> Result<()>;
}

#[async_trait]
impl WsStreamExt for WsStream {
    async fn send_heartbeat(&mut self, tag: Option<&str>) -> Result<()> {
        let payload = if let Some(tag) = tag {
            json!({
                "op": OpCode::Heartbeat.number(),
                "d": {
                    "tag": tag,
                },
            })
        } else {
            json!({
                "op": OpCode::Heartbeat.number(),
            })
        };

        self.send_json(&payload).await
    }

    async fn send_identify(&mut self, project: &str, token: Option<&str>) -> Result<()> {
        let payload = if let Some(token) = token {
            json!({
                "op": OpCode::Identify.number(),
                "d": {
                    "project_id": project,
                    "token": token,
                },
            })
        } else {
            json!({
                "op": OpCode::Identify.number(),
                "d": {
                    "project_id": project,
                },
            })
        };

        self.send_json(&payload).await
    }
}
