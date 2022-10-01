#![warn(clippy::pedantic)]

use anyhow::Result;
use clap::Parser;
use hop_cli::commands::handle_command;
use hop_cli::commands::update::util::version_notice;
#[cfg(feature = "update")]
use hop_cli::commands::Commands;
use hop_cli::state::{State, StateOptions};
use hop_cli::{util, CLI};

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let now = tokio::time::Instant::now();

    // create a new CLI instance
    let cli = CLI::parse();

    // setup panic hook
    util::set_hook();

    util::logs(cli.verbose);

    let state = State::new(StateOptions {
        override_project: cli.project,
        override_token: std::env::var("HOP_TOKEN").ok(),
    })
    .await;

    match cli.commands {
        #[cfg(feature = "update")]
        Commands::Update(_) => None,
        // its okay for the notice to fail
        _ => version_notice(state.ctx.clone()).await.ok(),
    };

    if let Err(error) = handle_command(cli.commands, state).await {
        log::error!("{}", error);
        std::process::exit(1);
    }

    util::clean_term();

    #[cfg(debug_assertions)]
    log::debug!("Finished in {:#?}", now.elapsed());

    Ok(())
}
