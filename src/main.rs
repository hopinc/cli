#![deny(clippy::pedantic, clippy::nursery, clippy::cargo)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    let now = tokio::time::Instant::now();

    // a lib level function
    // for proper type checking
    hop_cli::run().await?;

    #[cfg(debug_assertions)]
    log::debug!("Finished in {:#?}", now.elapsed());

    Ok(())
}
