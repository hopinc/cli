mod commands;
mod config;
mod macros;
mod state;
mod store;
mod types;

use crate::commands::update::util::check_version;
use clap::Parser;
use commands::{handle_command, Commands};
use state::{State, StateOptions};

#[derive(Debug, Parser)]
#[structopt(name = "hop", about = "🐇 Interact with Hop via command line", version)]
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
    macros::set_hook();

    // create a new CLI instance
    let cli = CLI::from_args();

    macros::logs(cli.verbose);

    let state = State::new(StateOptions {
        override_project_id: cli.project,
        override_token: option_env!("HOP_TOKEN").map(|s| s.to_string()),
    })
    .await
    .unwrap();

    // only run the check if it's not the update command
    match cli.commands {
        Commands::Update(_) => {}
        _ => {
            let (update, latest) = check_version(false).await;

            if update {
                log::warn!("A new version of hop_cli is available: {}", latest);
            }
        }
    }

    handle_command(cli.commands, state).await
}
