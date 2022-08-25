use std::sync::Arc;

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use tokio::{spawn, sync::Mutex};

use self::types::{ShardManagerMessage, ShardRunnerInfo};
use crate::{
    errors::Result,
    runner::ShardRunner,
    shard::{types::InterMessage, Shard},
};

pub(crate) mod types;

#[derive(Debug)]
pub struct ManagerOptions<'a> {
    pub project: &'a str,
    pub ws_url: &'a str,
    pub token: Option<&'a str>,
}

impl Default for ManagerOptions<'_> {
    fn default() -> Self {
        Self {
            project: "",
            token: None,
            ws_url: "wss://leap.hop.io/ws",
        }
    }
}

#[derive(Debug)]
pub struct ShardManager {
    pub runner_info: Arc<Mutex<ShardRunnerInfo>>,
    manager_rx: UnboundedReceiver<ShardManagerMessage>,
    manager_tx: UnboundedSender<ShardManagerMessage>,
    token: Option<String>,
    project: String,
    ws_url: Arc<Mutex<String>>,
}

impl ShardManager {
    pub async fn new(options: ManagerOptions<'_>) -> Result<Self> {
        let (manager_tx, manager_rx) = unbounded();

        let ws_url = Arc::new(Mutex::new(options.ws_url.to_string()));
        let shard = Shard::new(ws_url.clone(), options.project, options.token).await?;

        let mut runner = ShardRunner::new(manager_tx.clone(), shard);
        let stage = runner.shard.stage();
        let runner_tx = runner.get_runner_tx();

        spawn(async move { runner.run().await });

        Ok(Self {
            ws_url,
            manager_rx,
            manager_tx,
            project: options.project.to_string(),
            token: options.token.map(std::string::ToString::to_string),
            runner_info: Arc::new(Mutex::new(ShardRunnerInfo {
                latency: None,
                runner_tx,
                stage,
            })),
        })
    }

    pub async fn listen(&mut self) -> Option<ShardManagerMessage> {
        self.manager_rx.next().await
    }
}
