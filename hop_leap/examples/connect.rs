use futures::{channel::mpsc::unbounded, SinkExt, StreamExt};
use hop_leap::manager::{types::ShardManagerMessage, ManagerOptions, ShardManager};
use serde_json::json;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .ok();

    let (event_tx, mut event_rx) = unbounded();

    let mut manager = ShardManager::new(ManagerOptions {
        project: &std::env::var("PROJECT").unwrap(),
        token: std::env::var("TOKEN").ok().as_deref(),
        ws_url: "wss://leap.hop.io/ws",
        event_tx,
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut manager_tx = manager.get_manager_tx();

    spawn(async move { manager.run().await.ok() });

    spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        manager_tx
            .send(ShardManagerMessage::Json(json!({
                "op": 0,
                "d": {
                    "c": "project_NDQ0NzA4NDg3NTU4MjI1OTM",
                    "e": "SUBSCRIBE",
                    "d": null
                }
            })))
            .await
            .ok();
    });

    while let Some(event) = event_rx.next().await {
        println!("[EXAMPLE] Event: {event:?}");
    }

    Ok(())
}
