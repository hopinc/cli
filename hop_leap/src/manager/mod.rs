pub mod types;

use std::sync::Arc;

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use tokio::{spawn, sync::Mutex};

use self::types::{ShardManagerMessage, ShardRunnerInfo};
use crate::errors::Result;
use crate::leap::types::Event;
use crate::messenger::{types::ShardMessengerMessage, ShardMessenger};
use crate::runner::ShardRunner;
use crate::shard::types::InterMessage;
use crate::shard::Shard;

#[derive(Debug)]
pub struct ManagerOptions<'a> {
    pub project: &'a str,
    pub ws_url: &'a str,
    pub token: Option<&'a str>,
    pub event_tx: UnboundedSender<Event>,
}

#[derive(Debug)]
struct RunnerOptions<'a> {
    ws_url: Arc<Mutex<String>>,
    project: &'a str,
    token: Option<&'a str>,
    manager_tx: UnboundedSender<ShardManagerMessage>,
}

#[derive(Debug)]
pub struct ShardManager {
    pub runner_info: Arc<Mutex<ShardRunnerInfo>>,
    manager_rx: UnboundedReceiver<ShardManagerMessage>,
    manager_tx: UnboundedSender<ShardManagerMessage>,
    event_tx: UnboundedSender<Event>,
    messenger_tx: UnboundedSender<ShardMessengerMessage>,
    token: Option<String>,
    project: String,
    ws_url: Arc<Mutex<String>>,
}

impl ShardManager {
    pub async fn new(options: ManagerOptions<'_>) -> Result<Self> {
        let (manager_tx, manager_rx) = unbounded();

        let ws_url = Arc::new(Mutex::new(options.ws_url.to_string()));

        let runner_info = Self::create_runner(&RunnerOptions {
            ws_url: ws_url.clone(),
            manager_tx: manager_tx.clone(),
            project: options.project,
            token: options.token,
        })
        .await?;

        let mut messenger = ShardMessenger::new(runner_info.clone()).await;
        let messenger_tx = messenger.get_tx();

        spawn(async move {
            if let Err(why) = messenger.run().await {
                log::debug!("[Messenger] Stopped: {why:?}");
            } else {
                log::debug!("[Messenge] Stopped");
            }
        });

        Ok(Self {
            ws_url,
            manager_rx,
            manager_tx,
            messenger_tx,
            event_tx: options.event_tx,
            project: options.project.to_string(),
            token: options.token.map(std::string::ToString::to_string),
            runner_info,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.manager_rx.next().await {
                Some(ShardManagerMessage::Event(data)) => {
                    self.event_tx.send(data).await.ok();
                }

                Some(ShardManagerMessage::Json(data)) => {
                    self.messenger_tx
                        .send(ShardMessengerMessage::Json(data))
                        .await
                        .ok();
                }

                Some(ShardManagerMessage::Restart) => {
                    self.runner_info = Self::create_runner(&RunnerOptions {
                        ws_url: self.ws_url.clone(),
                        manager_tx: self.manager_tx.clone(),
                        project: &self.project,
                        token: self.token.as_deref(),
                    })
                    .await?;
                }

                Some(ShardManagerMessage::Update(data)) => {
                    self.messenger_tx
                        .send(ShardMessengerMessage::Update(data.clone()))
                        .await
                        .ok();

                    let runner = ShardRunnerInfo {
                        latency: data.latency,
                        runner_tx: self.runner_info.lock().await.runner_tx.clone(),
                        stage: data.stage,
                    };

                    self.runner_info = Arc::new(Mutex::new(runner));
                }

                Some(ShardManagerMessage::Close) => {
                    self.messenger_tx
                        .send(ShardMessengerMessage::Close)
                        .await
                        .ok();

                    self.runner_info
                        .lock()
                        .await
                        .runner_tx
                        .send(InterMessage::Close)
                        .await
                        .ok();

                    break Ok(());
                }

                _ => {}
            }
        }
    }

    async fn create_runner(options: &RunnerOptions<'_>) -> Result<Arc<Mutex<ShardRunnerInfo>>> {
        let shard = Shard::new(options.ws_url.clone(), options.project, options.token).await?;

        let mut runner = ShardRunner::new(options.manager_tx.clone(), shard);

        let stage = runner.shard.stage();
        let runner_tx = runner.get_tx();

        spawn(async move {
            if let Err(why) = runner.run().await {
                log::debug!("[Shard] Runner error: {:?}", why);
            }

            log::debug!("[Shard] Stopped");
        });

        Ok(Arc::new(Mutex::new(ShardRunnerInfo {
            latency: None,
            runner_tx,
            stage,
        })))
    }

    pub fn get_manager_tx(&self) -> UnboundedSender<ShardManagerMessage> {
        self.manager_tx.clone()
    }
}
