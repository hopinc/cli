use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_tungstenite::tokio::connect_async_with_config;
use async_tungstenite::tungstenite::protocol::WebSocketConfig;
use async_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::{interval, timeout};

use super::types::{ArisuEvent, ArisuMessage, ConnectionStage, OpCode, WsStream};

const ARISU_URL: &str = "wss://arisu.hop.io/ws";

#[derive(Debug)]
pub struct ArisuShardInfo {
    pub container_id: String,
    pub token: String,
    pub arisu_out_rx: UnboundedReceiver<Value>,
    pub arisu_in_tx: UnboundedSender<ArisuMessage>,
    pub logs_requested: Arc<AtomicBool>,
    pub metrics_requested: Arc<AtomicBool>,
}

pub struct ArisuShard {
    client: WsStream,
    container_id: String,
    token: String,
    arisu_out_rx: UnboundedReceiver<Value>,
    arisu_in_tx: UnboundedSender<ArisuMessage>,
    heartbeat_tx: UnboundedSender<()>,
    heartbeat_rx: UnboundedReceiver<()>,
    stage: ConnectionStage,
    heartbeat_interval: Option<JoinHandle<()>>,
    logs_requested: Arc<AtomicBool>,
    metrics_requested: Arc<AtomicBool>,
}

impl ArisuShard {
    pub async fn new(info: ArisuShardInfo) -> Result<Self> {
        let (heartbeat_tx, heartbeat_rx) = unbounded_channel::<()>();

        let client =
            connect(&std::env::var("ARISU_URL").unwrap_or_else(|_| ARISU_URL.to_string())).await?;

        Ok(Self {
            stage: ConnectionStage::Handshake,
            container_id: info.container_id.to_string(),
            token: info.token.to_string(),
            client,
            arisu_in_tx: info.arisu_in_tx,
            arisu_out_rx: info.arisu_out_rx,
            heartbeat_tx,
            heartbeat_rx,
            heartbeat_interval: None,
            logs_requested: info.logs_requested,
            metrics_requested: info.metrics_requested,
        })
    }

    async fn send_json<T>(&mut self, json: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let body = serde_json::to_string(&json)?;

        log::debug!("Sending message: {body}");

        self.client
            .send(Message::Text(body))
            .await
            .map_err(|e| e.into())
    }

    async fn receive_json(&mut self) -> Result<Option<ArisuEvent>> {
        match timeout(Duration::from_millis(1), self.client.next()).await {
            Ok(Some(Ok(message))) => match message {
                Message::Text(text) => {
                    log::debug!("Received message: {text}");

                    match serde_json::from_str(&text) {
                        Ok(data) => Ok(Some(data)),
                        Err(error) => {
                            log::debug!("Failed to parse message: {}", error);

                            Ok(None)
                        }
                    }
                }

                Message::Close(frame) => {
                    if let Some(close) = frame {
                        bail!("Received close frame {}: {}", close.code, close.reason);
                    }

                    Err(anyhow!("Received close frame"))
                }

                _ => Err(anyhow!("Unexpected message type")),
            },
            Ok(Some(Err(error))) => Err(error.into()),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn identify(&mut self) -> Result<()> {
        let msg = json!({
            "op": OpCode::Identify,
            "d": {
                "container_id": self.container_id,
                "token": self.token,
            }
        });

        self.send_json(msg).await
    }

    async fn heartbeat(&mut self) -> Result<()> {
        let msg = json!({
            "op": OpCode::Heartbeat,
        });

        self.send_json(msg).await
    }

    async fn handle_event(&mut self, event: ArisuEvent) -> bool {
        match event {
            ArisuEvent::Hello(heartbeat) => {
                if self.stage != ConnectionStage::Handshake {
                    // cant recover from this
                    return false;
                }

                self.stage = ConnectionStage::Identifying;

                let heartbeat_tx = self.heartbeat_tx.clone();

                let interval = spawn(async move {
                    let mut tokio_interval = interval(Duration::from_millis(heartbeat));
                    // skip first tick
                    tokio_interval.tick().await;

                    loop {
                        tokio_interval.tick().await;

                        // if the heartbeat fails, the interval will be dropped
                        if heartbeat_tx.send(()).is_err() {
                            break;
                        }
                    }
                });

                self.heartbeat_interval = Some(interval);

                self.identify().await.is_ok()
            }

            ArisuEvent::ServiceMessage(message) => {
                if self.stage == ConnectionStage::Identifying {
                    self.stage = ConnectionStage::Connected;

                    if self.arisu_in_tx.send(ArisuMessage::Open).is_err() {
                        return false;
                    }
                } else if self.stage != ConnectionStage::Connected {
                    // cant recover from this
                    return false;
                }

                self.arisu_in_tx
                    .send(ArisuMessage::ServiceMessage(message))
                    .is_ok()
            }

            ArisuEvent::Logs(log) => self.arisu_in_tx.send(log.into()).is_ok(),

            ArisuEvent::Metrics(metrics) => self.arisu_in_tx.send(metrics.into()).is_ok(),

            ArisuEvent::HeartbeatAck => true,

            ArisuEvent::LogsRequestedAck => {
                self.logs_requested.store(true, Relaxed);

                true
            }

            ArisuEvent::MetricsRequestedAck => {
                self.metrics_requested.store(true, Relaxed);

                true
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if self.heartbeat_rx.try_recv().is_ok() {
                self.heartbeat().await?;
            }

            if let Ok(data) = self.arisu_out_rx.try_recv() {
                self.send_json(data).await?;
            }

            match self.receive_json().await {
                Ok(Some(event)) => {
                    if !self.handle_event(event).await {
                        log::error!("Failed to handle arisu event");

                        self.stage = ConnectionStage::Disconnected;
                        self.client.close(None).await?;
                    }
                }

                Err(error) => {
                    return Err(error);
                }

                Ok(None) => {}
            }
        }
    }
}

async fn connect(base_url: &str) -> Result<WsStream> {
    let url = format!("{base_url}?encoding=json&compression=none");

    log::debug!("{url}");

    let config = WebSocketConfig {
        max_message_size: None,
        max_frame_size: None,
        accept_unmasked_frames: false,
        ..Default::default()
    };

    let (stream, _) = connect_async_with_config(url, Some(config)).await?;

    Ok(stream)
}
