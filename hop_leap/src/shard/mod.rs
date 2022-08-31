pub mod error;
pub mod socket;
pub mod types;

use std::sync::Arc;
use std::time::Duration;

use async_tungstenite::tokio::connect_async_with_config;
use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use async_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::SinkExt;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::time::Instant;

use self::error::Error as GatewayError;
use self::socket::{WsStream, WsStreamExt};
use self::types::{close_codes, ConnectionStage, GatewayEvent, ReconnectType, ShardAction};
use crate::errors::{Error, Result};
use crate::heartbeat::types::HeartbeatManagerEvent;
use crate::heartbeat::HeartbeatManager;
use crate::leap::types::Event;

#[cfg(feature = "zlib")]
const ENCODING: &str = "none";
#[cfg(not(feature = "zlib"))]
const ENCODING: &str = "zlib";

pub struct Shard {
    pub client: WsStream,
    heartbeat_trigger: UnboundedReceiver<()>,
    heartbeat_manager: UnboundedSender<HeartbeatManagerEvent>,
    heartbeat_instants: (Option<Instant>, Option<Instant>),
    last_heartbeat_acknowledged: bool,
    stage: ConnectionStage,
    pub token: Option<String>,
    pub project: String,
}

impl Shard {
    pub async fn new(
        ws_url: Arc<Mutex<String>>,
        project: &str,
        token: Option<&str>,
    ) -> Result<Self> {
        let ws_url = ws_url.lock().await.clone();
        let client = connect(&ws_url).await?;

        let heartbeat_instants = (None, None);
        let last_heartbeat_acknowledged = true;
        let stage = ConnectionStage::Handshake;

        let (trigger_tx, heartbeat_trigger) = unbounded();
        let mut heartbeat = HeartbeatManager::new(trigger_tx);
        let heartbeat_manager = heartbeat.heartbeat_tx();

        spawn(async move { heartbeat.run().await });

        Ok(Self {
            client,
            heartbeat_trigger,
            heartbeat_instants,
            heartbeat_manager,
            last_heartbeat_acknowledged,
            stage,
            project: project.to_string(),
            token: token.map(std::string::ToString::to_string),
        })
    }

    pub async fn heartbeat(&mut self, tag: Option<&str>) -> Result<()> {
        match self.client.send_heartbeat(tag).await {
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

                Err(Error::Gateway(GatewayError::HeartbeatFailed))
            }
        }
    }

    pub async fn identify(&mut self) -> Result<()> {
        self.client
            .send_identify(&self.project, self.token.as_deref())
            .await?;

        self.heartbeat_instants.0 = Some(Instant::now());
        self.stage = ConnectionStage::Identifying;

        Ok(())
    }

    pub async fn check_heartbeat(&mut self) -> bool {
        match self.heartbeat_trigger.try_next() {
            // continue to send a heartbeat if it was sent from the thread
            Ok(Some(_)) => {}
            // close since we failed to get a heartbeat from the thread
            Ok(None) => {
                log::warn!("[Shard] Err checking heartbeat: channel closed");
                return false;
            }
            // no heartbeat was waiting, so continue
            Err(_) => return true,
        };

        if !self.last_heartbeat_acknowledged {
            return false;
        }

        if let Err(why) = self.heartbeat(None).await {
            log::warn!("[Shard] Err heartbeating: {why:?}");

            false
        } else {
            true
        }
    }

    pub(crate) async fn handle_event(
        &mut self,
        event: &Result<GatewayEvent>,
    ) -> Result<Option<ShardAction>> {
        match event {
            Ok(GatewayEvent::Dispatch(ref event)) => Ok(self.handle_dispatch(event)),
            Ok(GatewayEvent::Heartbeat(tag)) => Ok(self.handle_heartbeat_event(tag).await),
            Ok(GatewayEvent::HeartbeatAck(..)) => {
                self.heartbeat_instants.1 = Some(Instant::now());
                self.last_heartbeat_acknowledged = true;

                log::trace!("[Shard] Received heartbeat ack");

                Ok(Some(ShardAction::Update))
            }
            Ok(GatewayEvent::Hello(interval)) => {
                if interval > &0 {
                    self.heartbeat_manager
                        .send(HeartbeatManagerEvent::UpdateInterval(*interval))
                        .await?;
                }

                Ok(Some(if self.stage == ConnectionStage::Handshake {
                    ShardAction::Identify
                } else {
                    log::debug!("[Shard] Received late Hello; autoreconnecting");

                    ShardAction::Reconnect(self.reconnection_type())
                }))
            }

            Err(Error::Gateway(GatewayError::Closed(ref data))) => self.handle_closed(data),
            Err(Error::Tungstenite(ref why)) => {
                log::warn!("[Shard] Websocket error: {why:?}");
                log::info!("[Shard] Will attempt to auto-reconnect",);

                Ok(Some(ShardAction::Reconnect(self.reconnection_type())))
            }

            Err(ref why) => {
                log::warn!("[Shard] Unhandled error: {why:?}");

                Ok(None)
            }
        }
    }

    async fn handle_heartbeat_event(&mut self, tag: &Option<String>) -> Option<ShardAction> {
        if let Some(tag) = tag {
            return Some(ShardAction::Heartbeat(Some(tag.clone())));
        }

        None
    }

    fn handle_dispatch(&mut self, event: &Event) -> Option<ShardAction> {
        if matches!(event, Event::Init(_)) {
            self.stage = ConnectionStage::Connected;
        }

        None
    }

    fn handle_closed(&mut self, data: &Option<CloseFrame<'static>>) -> Result<Option<ShardAction>> {
        self.stage = ConnectionStage::Disconnected;

        let num = data.as_ref().map(|d| d.code.into());
        let clean = num == Some(1000);

        match num {
            Some(close_codes::AUTHENTICATION_FAILED) => {
                log::debug!("[Shard] Authentication failed; closing connection");

                return Err(Error::Gateway(GatewayError::InvalidAuthentication));
            }

            Some(other) if !clean => {
                log::warn!("[Shard] Received unexpected close code: {other}");
            }

            _ => {}
        }

        Ok(Some(ShardAction::Reconnect(ReconnectType::Reidentify)))
    }

    pub fn reconnection_type(&self) -> ReconnectType {
        // resumes are not supported yet
        ReconnectType::Reidentify
    }

    pub fn stage(&self) -> ConnectionStage {
        self.stage.clone()
    }

    pub fn latency(&self) -> Option<Duration> {
        match self.heartbeat_instants {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    pub async fn shutdown(&mut self) {
        self.heartbeat_manager
            .send(HeartbeatManagerEvent::Shutdown)
            .await
            .ok();
        self.client.close(None).await.ok();
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
