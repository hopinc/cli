use futures::channel::mpsc::UnboundedSender;
use tokio::time::Duration;

use crate::shard::types::{ConnectionStage, Event, InterMessage};

#[derive(Debug)]
pub enum ShardManagerMessage {
    Restart,
    Event(Event),
}

#[derive(Debug)]
pub struct ShardRunnerInfo {
    /// The latency between when a heartbeat was sent and when the
    /// acknowledgement was received.
    pub latency: Option<Duration>,
    /// The channel used to communicate with the shard runner, telling it
    /// what to do with regards to its status.
    pub runner_tx: UnboundedSender<InterMessage>,
    /// The current connection stage of the shard.
    pub stage: ConnectionStage,
}
