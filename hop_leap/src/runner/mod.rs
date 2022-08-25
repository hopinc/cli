use async_tungstenite::tungstenite::Error as TungsteniteError;
use futures::channel::mpsc::UnboundedReceiver;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::errors::Error;
use crate::manager::types::ShardManagerMessage;
use crate::shard::socket::{RecieverExt, SenderExt};

use crate::shard::types::{Event, GatewayEvent, ShardAction};
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
    pub fn new(
        manager_tx: UnboundedSender<ShardManagerMessage>,
        runner_rx: UnboundedReceiver<InterMessage>,
        runner_tx: UnboundedSender<InterMessage>,
        shard: Shard,
    ) -> Self {
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
            Err(why) => match why {
                _ => return Ok((None, None, true)),
            },
        };

        let event = match event {
            Ok(GatewayEvent::Dispatch(event)) => Some(event),
            _ => None,
        };

        Ok((event, action, true))
    }

    async fn handle_rx_message(&mut self, message: InterMessage) -> bool {
        match message {
            InterMessage::Json(json) => self.shard.client.send_json(&json).await.is_ok(),
        }
    }

    async fn request_restart(&mut self) -> Result<()> {
        self.manager_tx.send(ShardManagerMessage::Restart).ok();

        Ok(())
    }
}
