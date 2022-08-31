pub mod types;

use std::sync::Arc;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use self::types::ShardMessengerMessage;
use crate::errors::Result;
use crate::manager::types::ShardRunnerInfo;
use crate::shard::types::{ConnectionStage, InterMessage};

pub struct ShardMessenger {
    messenger_rx: UnboundedReceiver<ShardMessengerMessage>,
    messenger_tx: UnboundedSender<ShardMessengerMessage>,
    runner_tx: UnboundedSender<InterMessage>,
    stage: ConnectionStage,
}

impl ShardMessenger {
    pub async fn new(runner_info: Arc<Mutex<ShardRunnerInfo>>) -> Self {
        let runner = runner_info.lock().await;
        let (messenger_tx, messenger_rx) = unbounded();

        Self {
            messenger_rx,
            messenger_tx,
            runner_tx: runner.runner_tx.clone(),
            stage: runner.stage.clone(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if !self.handle_event().await {
                return Ok(());
            }
        }
    }

    async fn handle_event(&mut self) -> bool {
        match self.messenger_rx.next().await {
            Some(ShardMessengerMessage::Update(event)) => {
                self.stage = event.stage;

                true
            }

            Some(ShardMessengerMessage::Json(data)) => {
                if self.stage.is_connected() {
                    self.runner_tx.send(InterMessage::Json(data)).await.is_ok()
                } else {
                    self.messenger_tx
                        .send(ShardMessengerMessage::Json(data))
                        .await
                        .is_ok()
                }
            }

            Some(ShardMessengerMessage::Close) | None => false,
        }
    }

    pub fn get_tx(&self) -> UnboundedSender<ShardMessengerMessage> {
        self.messenger_tx.clone()
    }
}
