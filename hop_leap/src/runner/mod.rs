use futures::channel::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::shard::socket::SenderExt;

use crate::{
    errors::Result,
    shard::{types::InterMessage, Shard},
};

pub struct ShardRunner {
    manager_tx: UnboundedSender<String>,
    runner_rx: UnboundedReceiver<InterMessage>,
    runner_tx: UnboundedSender<InterMessage>,
    pub shard: Shard,
}

impl ShardRunner {
    pub fn new(
        manager_tx: UnboundedSender<String>,
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
            if !self.recieve().await? {
                return Ok(());
            }
        }
    }

    async fn recieve(&mut self) -> Result<bool> {
        loop {
            match self.runner_rx.try_next() {
                Ok(Some(message)) => {
                    if !self.handle_rx_message(message).await {
                        return Ok(false);
                    }
                }

                Ok(None) => return Ok(false),

                Err(_) => break,
            }
        }

        Ok(true)
    }

    async fn handle_rx_message(&mut self, message: InterMessage) -> bool {
        match message {
            InterMessage::Json(json) => self.shard.client.send_json(&json).await.is_ok(),
        }
    }
}
