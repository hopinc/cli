pub mod types;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use tokio::time::{self, Duration};

use self::types::HeartbeatManagerEvent;
use crate::errors::Result;

pub struct HeartbeatManager {
    interval: Option<u64>,
    trigger_tx: UnboundedSender<()>,
    heartbeat_rx: UnboundedReceiver<HeartbeatManagerEvent>,
    heartbeat_tx: UnboundedSender<HeartbeatManagerEvent>,
    first_beat: bool,
}

impl HeartbeatManager {
    pub fn new(trigger_tx: UnboundedSender<()>) -> Self {
        let (heartbeat_tx, heartbeat_rx) = unbounded();

        Self {
            interval: None,
            first_beat: true,
            trigger_tx,
            heartbeat_rx,
            heartbeat_tx,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut interval = time::interval(Duration::from_millis(self.interval()));

        // first heartbeat is immediate
        interval.tick().await;

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if self.first_beat {
                        self.first_beat = false;
                    } else {
                        self.trigger_tx.send(()).await?;
                    }
                }

                event = self.heartbeat_rx.next() => {
                    match event {
                        Some(HeartbeatManagerEvent::UpdateInterval(heartbeat_interval)) => {
                            self.interval = Some(heartbeat_interval);
                            interval = time::interval(Duration::from_millis(self.interval()));
                        }

                        None | Some(HeartbeatManagerEvent::Shutdown) => {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    fn interval(&self) -> u64 {
        // give a default value if the gateway fails to send one
        self.interval.unwrap_or(15_000)
    }

    pub fn heartbeat_tx(&self) -> UnboundedSender<HeartbeatManagerEvent> {
        self.heartbeat_tx.clone()
    }
}
