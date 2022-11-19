#![warn(clippy::pedantic)]

use anyhow::Result;
use clap::Parser;
use hop_cli::commands::handle_command;
use hop_cli::commands::update::util::version_notice;
#[cfg(feature = "update")]
use hop_cli::commands::Commands::Update;
use hop_cli::state::{State, StateOptions};
use hop_cli::{utils, CLI};

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let now = tokio::time::Instant::now();

    // create a new CLI instance
    let cli = CLI::parse();

    // setup panic hook
    utils::set_hook();

    utils::logs(cli.debug);

    let state = State::new(StateOptions {
        override_project: cli.project,
        override_token: std::env::var("HOP_TOKEN").ok(),
    })
    .await;

    match cli.commands {
        #[cfg(feature = "update")]
        Update(_) => None,
        // its okay for the notice to fail
        _ => version_notice(state.ctx.clone()).await.ok(),
    };

    if let Err(error) = handle_command(cli.commands, state).await {
        log::error!("{}", error);
        std::process::exit(1);
    }

    utils::clean_term();

    #[cfg(debug_assertions)]
    log::debug!("Finished in {:#?}", now.elapsed());

    Ok(())
}
