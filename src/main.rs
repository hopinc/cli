#![warn(clippy::pedantic)]

use clap::Parser;
use hop_cli::commands::update::util::check_version;
use hop_cli::commands::{handle_command, Commands};
use hop_cli::state::{State, StateOptions};
use hop_cli::utils;
use tokio::task;

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
async fn main() -> Result<(), std::io::Error> {
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

    // only run the check if it's not the update command
    match cli.commands {
        Commands::Update(_) => {}
        _ => {
            task::spawn(async move {
                let (update, latest) = check_version(false, true).await;

                if update {
                    log::warn!("A new version is available: {}", latest);
                }
            });
        }
    }

    handle_command(cli.commands, state).await
}
