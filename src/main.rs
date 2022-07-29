mod commands;
mod config;
mod state;
mod store;
mod types;
mod utils;

use crate::commands::update::util::check_version;
use clap::Parser;
use commands::{handle_command, Commands};
use state::{State, StateOptions};
use tokio::task;

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
    utils::set_hook();

    // create a new CLI instance
    let cli = CLI::from_args();

    utils::logs(cli.verbose);

    let state = State::new(StateOptions {
        override_project_id: cli.project,
        override_token: option_env!("HOP_TOKEN").map(|s| s.to_string()),
    })
    .await;

    // only run the check if it's not the update command
    match cli.commands {
        Commands::Update(_) => {}
        _ => {
            task::spawn(async move {
                let (update, latest) = check_version(false).await;

                if update {
                    log::warn!("A new version is available: {}", latest);
                }
            });
        }
    }

    match handle_command(cli.commands, state).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error bruh");
            log::error!("{}", e);
            Err(e)
        }
    }
}
