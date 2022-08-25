use async_tungstenite::tungstenite::Error as TungsteniteError;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::SinkExt;
use serde::Deserialize;

use crate::errors::Error;
use crate::manager::types::ShardManagerMessage;
use crate::shard::socket::{RecieverExt, SenderExt};

use crate::shard::types::{Event, GatewayEvent, ReconnectType, ShardAction};
use crate::{
    errors::Result,
    shard::{types::InterMessage, Shard},
};

pub struct ShardRunner {
    manager_tx: UnboundedSender<ShardManagerMessage>,
    runner_rx: UnboundedReceiver<InterMessage>,
    runner_tx: UnboundedSender<InterMessage>,
    pub shard: Shard,
}

impl ShardRunner {
    pub fn new(manager_tx: UnboundedSender<ShardManagerMessage>, shard: Shard) -> Self {
        let (runner_tx, runner_rx) = unbounded();

        Self {
            manager_tx,
            runner_rx,
            runner_tx,
            shard,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if !self.recieve_internal().await? {
                return Ok(());
            }

            if !self.shard.check_heartbeat().await {
                return self.request_restart().await;
            }

            let previous_stage = self.shard.stage();
            let (event, action, success) = self.recieve_event().await?;
            let current_stage = self.shard.stage();

            if previous_stage != current_stage {
                // TODO: update manager
            }

            match action {
                Some(ShardAction::Reconnect(ReconnectType::Reidentify)) => {
                    return self.request_restart().await;
                }
                Some(other) => {
                    if let Err(e) = self.action(&other).await {
                        match self.shard.reconnection_type() {
                            ReconnectType::Reidentify => return self.request_restart().await,
                        };
                    }
                }
                None => {}
            }

            if let Some(event) = event {
                self.manager_tx
                    .send(ShardManagerMessage::Event(event))
                    .await
                    .ok();
            }

            if !success && !self.shard.stage().is_connecting() {
                return self.request_restart().await;
            }
        }
    }

    async fn recieve_internal(&mut self) -> Result<bool> {
        loop {
            match self.runner_rx.try_next() {
                Ok(Some(message)) => {
                    if !self.handle_rx_message(message).await {
                        return Ok(false);
                    }
                }

                Ok(None) => {
                    drop(self.request_restart().await);

                    return Ok(false);
                }

                Err(_) => break,
            }
        }

        Ok(true)
    }

    async fn recieve_event(&mut self) -> Result<(Option<Event>, Option<ShardAction>, bool)> {
        let gateway_event = match self.shard.client.recieve_json().await {
            Ok(Some(event)) => GatewayEvent::deserialize(event)
                .map(Some)
                .map_err(From::from),

            Ok(None) => Ok(None),

            Err(Error::Tungstenite(TungsteniteError::Io(_))) => {
                return Ok((None, None, true));
            }

            Err(why) => Err(why),
        };

        let event = match gateway_event {
            Ok(Some(event)) => Ok(event),
            Ok(None) => return Ok((None, None, true)),
            Err(why) => Err(why),
        };

        let action = match self.shard.handle_event(&event) {
            Ok(Some(action)) => Some(action),
            Ok(None) => None,
            Err(_) => return Ok((None, None, true)),
        };

        log::debug!("[Shard] GatewayEvent: {event:?}");

        let event = match event {
            Ok(GatewayEvent::Dispatch(event)) => Some(event),
            _ => None,
        };

        Ok((event, action, true))
    }

    async fn action(&mut self, action: &ShardAction) -> Result<()> {
        match action {
            ShardAction::Reconnect(ReconnectType::Reidentify) => self.request_restart().await,
            ShardAction::Heartbeat(tag) => self.shard.heartbeat(tag.as_deref()).await,
            ShardAction::Identify => self.shard.identify().await,
        }
    }

    async fn dispatch(&mut self) {
        todo!()
    }

    async fn handle_rx_message(&mut self, message: InterMessage) -> bool {
        match message {
            InterMessage::Json(json) => self.shard.client.send_json(&json).await.is_ok(),
        }
    }

    async fn request_restart(&mut self) -> Result<()> {
        self.manager_tx
            .send(ShardManagerMessage::Restart)
            .await
            .ok();

        Ok(())
    }

    pub fn get_runner_tx(&self) -> UnboundedSender<InterMessage> {
        self.runner_tx.clone()
    }
}
