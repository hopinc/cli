mod shard;
mod types;

use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc,
};

use anyhow::Result;
use futures_util::Stream;
use serde_json::{json, Value};
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver, UnboundedSender, /* , UnboundedSender */
};

pub use self::types::ArisuMessage;
use self::{
    shard::{ArisuShard, ArisuShardInfo},
    types::OpCode,
};

pub struct ArisuClient {
    tx: UnboundedSender<Value>,
    rx: UnboundedReceiver<ArisuMessage>,
    logs_requested: Arc<AtomicBool>,
    metrics_requested: Arc<AtomicBool>,
}

impl ArisuClient {
    pub async fn new(container_id: &str, token: &str) -> Result<Self> {
        let (tx, arisu_out_rx) = unbounded_channel::<_>();
        let (arisu_in_tx, rx) = unbounded_channel::<_>();
        let logs_requested = Arc::new(AtomicBool::new(false));
        let metrics_requested = Arc::new(AtomicBool::new(false));

        let shard_info = ArisuShardInfo {
            arisu_in_tx,
            arisu_out_rx,
            container_id: container_id.to_string(),
            token: token.to_string(),
            logs_requested: logs_requested.clone(),
            metrics_requested: metrics_requested.clone(),
        };

        let mut shard = ArisuShard::new(shard_info).await?;

        tokio::spawn(async move {
            if let Err(why) = shard.run().await {
                log::error!("Error in Arisu shard: {why}");
            }
        });

        Ok(Self {
            tx,
            rx,
            logs_requested,
            metrics_requested,
        })
    }

    pub async fn request_logs(&self) -> Result<()> {
        if self.logs_requested.load(Relaxed) {
            return Ok(());
        }

        self.tx.send(json!({
            "op": OpCode::RequestLogs,
        }))?;

        while !self.logs_requested.load(Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn unsubscribe_logs(&self) -> Result<()> {
        if !self.logs_requested.load(Relaxed) {
            return Ok(());
        }

        self.tx.send(json!({
            "op": OpCode::UnsubscribeLogs,
        }))?;

        while self.logs_requested.load(Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn request_metrics(&self) -> Result<()> {
        if self.metrics_requested.load(Relaxed) {
            return Ok(());
        }

        self.tx.send(json!({
            "op": OpCode::RequestMetrics,
        }))?;

        while !self.metrics_requested.load(Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn unsubscribe_metrics(&self) -> Result<()> {
        if !self.metrics_requested.load(Relaxed) {
            return Ok(());
        }

        self.tx.send(json!({
            "op": OpCode::UnsubscribeMetrics,
        }))?;

        while self.metrics_requested.load(Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

impl Stream for ArisuClient {
    type Item = ArisuMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
