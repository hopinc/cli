pub mod error;
pub mod socket;
pub mod types;

use std::sync::Arc;

use async_tungstenite::tokio::connect_async_with_config;
use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use async_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio::sync::Mutex;
use tokio::time::Instant;

use self::error::Error as LeapError;
use self::socket::WsStreamExt;
use self::{socket::WsStream, types::ConnectionStage};
use crate::errors::{Error, Result};

#[cfg(feature = "zlib")]
const ENCODING: &str = "none";
#[cfg(not(feature = "zlib"))]
const ENCODING: &str = "zlib";

pub struct Shard {
    pub client: WsStream,
    heartbeat_interval: Option<u64>,
    heartbeat_instants: (Option<Instant>, Option<Instant>),
    last_heartbeat_acknowledged: bool,
    stage: ConnectionStage,
    pub started: Instant,
    pub token: String,
    ws_url: String,
}

impl Shard {
    pub async fn new(ws_url: Arc<Mutex<String>>, token: &str) -> Result<Self> {
        let ws_url = ws_url.lock().await.clone();
        let client = connect(&ws_url).await?;

        let heartbeat_instants = (None, None);
        let heartbeat_interval = None;
        let last_heartbeat_acknowledged = true;
        let stage = ConnectionStage::Handshake;

        Ok(Self {
            client,
            heartbeat_interval,
            heartbeat_instants,
            last_heartbeat_acknowledged,
            stage,
            ws_url,
            started: Instant::now(),
            token: token.to_string(),
        })
    }

    pub async fn heartbeat(&mut self) -> Result<()> {
        match self.client.send_heartbeat(None).await {
            Ok(()) => {
                self.heartbeat_instants.0 = Some(Instant::now());
                self.last_heartbeat_acknowledged = false;

                Ok(())
            }

            Err(why) => {
                match why {
                    Error::Tungstenite(TungsteniteError::Io(err)) => {
                        if err.raw_os_error() != Some(32) {
                            log::debug!("[Shard] Err heartbeating: {err:?}");
                        }
                    }

                    other => {
                        log::warn!("[Shard] Other err w/ keepalive: {other:?}");
                    }
                }

                Err(Error::Leap(LeapError::HeartbeatFailed))
            }
        }
    }
}

async fn connect(base_url: &str) -> Result<WsStream> {
    let url = format!("{base_url}?encoding=json&compression={ENCODING}");

    let config = WebSocketConfig {
        max_message_size: None,
        max_frame_size: None,
        max_send_queue: None,
        accept_unmasked_frames: false,
    };

    let (stream, _) = connect_async_with_config(url, Some(config)).await?;

    Ok(stream)
}
