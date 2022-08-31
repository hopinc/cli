use hop_leap::{LeapEdge, LeapOptions};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    fern::Dispatch::new()
        .format(|out, message, _| {
            out.finish(format_args!(
                "{} - {}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .ok();

    let mut manager = LeapEdge::new(LeapOptions {
        project: &std::env::var("PROJECT").unwrap(),
        token: std::env::var("TOKEN").ok().as_deref(),
        ..Default::default()
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if let Ok(channel) = std::env::var("CHANNEL") {
        manager.channel_subscribe(&channel).await.ok();
    }

    while (manager.listen().await).is_some() {}

    sleep(Duration::from_secs(5)).await;

    log::debug!("Done :D");

    Ok(())
}
