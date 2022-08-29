use hop_leap::{LeapEdge, LeapOptions};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    fern::Dispatch::new()
        .format(|out, message, _| {
            out.finish(format_args!(
                "({}): {}",
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

    manager
        .channel_subscribe("gsdgsdhghsjdkfsgdgsdgsdfg", &None)
        .await
        .ok();

    if let Ok(channel) = std::env::var("CHANNEL") {
        for _ in 1..10 {
            manager.channel_subscribe(&channel, &None).await.ok();
        }
    }

    while let Some(event) = manager.listen().await {
        if matches!(event.e.as_str(), "MESSAGE" | "DIRECT_MESSAGE") {
            println!("[EXAMPLE] Event: {event:?}");
        }
    }

    Ok(())
}
