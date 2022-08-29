use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::spawn;

use crate::errors::Result;
use crate::manager::{types::ShardManagerMessage, ManagerOptions, ShardManager};
use crate::shard::types::Event;

pub struct LeapOptions<'a> {
    pub token: Option<&'a str>,
    pub project: &'a str,
    pub ws_url: &'a str,
}

impl Default for LeapOptions<'_> {
    fn default() -> Self {
        Self {
            token: None,
            project: "",
            ws_url: "wss://leap.hop.io/ws",
        }
    }
}

pub struct LeapEdge {
    manager_tx: UnboundedSender<ShardManagerMessage>,
    leap_rx: UnboundedReceiver<Event>,
}

impl LeapEdge {
    pub async fn new(options: LeapOptions<'_>) -> Result<Self> {
        let (leap_tx, leap_rx) = unbounded();

        let mut manager = ShardManager::new(ManagerOptions {
            project: options.project,
            ws_url: options.ws_url,
            token: options.token,
            event_tx: leap_tx.clone(),
        })
        .await?;

        let manager_tx = manager.get_manager_tx();

        spawn(async move { manager.run().await });

        Ok(Self {
            manager_tx,
            leap_rx,
        })
    }

    pub async fn send_service_message<D>(&mut self, message: D) -> Result<()>
    where
        D: Into<Value>,
    {
        self.manager_tx
            .send(ShardManagerMessage::Json(message.into()))
            .await
            .ok();

        Ok(())
    }

    #[inline]
    pub async fn channel_subscribe(&mut self, channel: &str, data: &Option<Value>) -> Result<()> {
        self.send_service_message(json!({
            "op": 0,
            "d": {
                "c": channel,
                "e": "SUBSCRIBE",
                "d": data
            }
        }))
        .await
    }

    pub async fn listen(&mut self) -> Option<Event> {
        self.leap_rx.next().await
    }
}
