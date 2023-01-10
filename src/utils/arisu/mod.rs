mod shard;
mod types;

use anyhow::Result;
use futures_util::Stream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver /* , UnboundedSender */};

use self::shard::{ArisuShard, ArisuShardInfo};
pub use self::types::ArisuMessage;

pub struct ArisuClient {
    // tx: UnboundedSender<String>,
    rx: UnboundedReceiver<ArisuMessage>,
}

impl ArisuClient {
    pub async fn new(container_id: &str, token: &str) -> Result<Self> {
        let (_arisu_out_tx, arisu_out_rx) = unbounded_channel::<String>();
        let (arisu_in_tx, arisu_in_rx) = unbounded_channel::<ArisuMessage>();

        let shard_info = ArisuShardInfo {
            arisu_in_tx,
            arisu_out_rx,
            container_id: container_id.to_string(),
            token: token.to_string(),
        };

        let mut shard = ArisuShard::new(shard_info).await?;

        tokio::spawn(async move {
            if let Err(why) = shard.run().await {
                log::error!("Error in Arisu shard: {why}");
            }
        });

        Ok(Self {
            // tx: arisu_out_tx,
            rx: arisu_in_rx,
        })
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
