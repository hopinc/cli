#![warn(clippy::pedantic)]

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let now = tokio::time::Instant::now();

    // a lib level function
    // for proper type checking
    hop_cli::run().await?;

    #[cfg(debug_assertions)]
    log::debug!("Finished in {:#?}", now.elapsed());

    Ok(())
}
