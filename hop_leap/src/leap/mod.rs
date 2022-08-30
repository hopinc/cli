pub mod types;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::spawn;

use self::types::Event;
use crate::errors::Result;
use crate::manager::{types::ShardManagerMessage, ManagerOptions, ShardManager};

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
    /// Creates a new Leap Edge instance.
    ///
    /// # Errors
    ///  - If the manager channel cannot connect to the Leap Edge.
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

        spawn(async move {
            if let Err(why) = manager.run().await {
                log::debug!("[Manager] Stopped: {why:?}");
            } else {
                log::debug!("[Manager] Stopped");
            }
        });

        Ok(Self {
            manager_tx,
            leap_rx,
        })
    }

    /// Send a service message to Leap Edge.
    ///
    /// # Errors
    /// - If the message cannot be sent.
    #[inline]
    pub async fn send_service_message<D>(&mut self, message: D) -> Result<()>
    where
        D: Serialize,
    {
        self.manager_tx
            .send(ShardManagerMessage::Json(json!({
                "op": 0,
                "d": message,
            })))
            .await?;

        Ok(())
    }

    /// Subscribe to a channel.
    ///
    /// # Errors
    /// - If the message cannot be sent.
    #[inline]
    pub async fn channel_subscribe(&mut self, channel: &str) -> Result<()> {
        self.channel_subscribe_with_data::<Option<Value>>(channel, None)
            .await
    }

    /// Subscribe to a channel with initial data.
    ///
    /// # Errors
    /// - If the message cannot be sent.
    #[inline]
    pub async fn channel_subscribe_with_data<D>(&mut self, channel: &str, data: D) -> Result<()>
    where
        D: Serialize,
    {
        self.send_service_message(&Event::Subscribe(json!({
            "c": channel,
            "d": data,
        })))
        .await
    }

    /// Listen for all events.
    #[inline]
    pub async fn listen(&mut self) -> Option<Event> {
        self.leap_rx.next().await
    }

    /// Close the Leap Edge connection and all related work threads.
    pub async fn close(&mut self) {
        self.manager_tx.send(ShardManagerMessage::Close).await.ok();
    }
}
