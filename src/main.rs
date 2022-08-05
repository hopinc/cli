#![warn(clippy::pedantic)]

use anyhow::Result;
use clap::Parser;
use hop_cli::commands::update::util::version_notice;
use hop_cli::commands::{handle_command, Commands};
use hop_cli::state::{State, StateOptions};
use hop_cli::utils;

#[derive(Debug, Parser)]
#[structopt(
    name = "hop",
    about = "🐇 Interact with Hop via command line",
    version,
    author
)]
pub struct CLI {
    #[clap(subcommand)]
    pub commands: Commands,

    #[clap(
        short = 'p',
        long = "project",
        help = "Namespace or ID of the project to use",
        global = true
    )]
    pub project: Option<String>,

    #[clap(
        short = 'v',
        long = "verbose",
        help = "Print more information",
        global = true
    )]
    pub verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // setup panic hook
    utils::set_hook();

    // create a new CLI instance
    let cli = CLI::from_args();

    utils::logs(cli.verbose);

    let state = State::new(StateOptions {
        override_project_id: cli.project,
        override_token: option_env!("HOP_TOKEN").map(std::string::ToString::to_string),
    })
    .await;

    // its okay for the notice to fail
    version_notice(state.ctx.clone()).await.ok();

    if let Err(error) = handle_command(cli.commands, state).await {
        log::error!("{}", error);
        std::process::exit(1);
    }

    Ok(())
}
