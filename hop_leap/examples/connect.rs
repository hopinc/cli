use hop_leap::manager::{ManagerOptions, ShardManager};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // fern::Dispatch::new()
    //     .level(log::LevelFilter::Debug)
    //     .chain(std::io::stdout())
    //     .apply()
    //     .ok();

    let mut manager = ShardManager::new(ManagerOptions {
        project: &std::env::var("PROJECT").unwrap(),
        token: std::env::var("TOKEN").ok().as_deref(),
        ..Default::default()
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    while let Some(event) = manager.listen().await {
        println!("{event:?}");
    }

    Ok(())
}
