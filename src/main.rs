mod commands;
mod config;
mod macros;
mod state;
mod store;
mod types;

use commands::{handle_command, Commands};
use state::{State, StateOptions};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop", about = "🐇 Interact with Hop via command line")]
pub struct CLI {
    #[structopt(subcommand)]
    pub commands: Commands,

    #[structopt(
        short = "p",
        long = "project",
        help = "Namespace or ID of the project to use",
        global = true
    )]
    pub project: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // setup panic hook
    macros::set_hook();

    // create a new CLI instance
    let cli = CLI::from_args();

    let state = State::new(StateOptions {
        override_project_id: cli.project,
        override_token: option_env!("HOP_TOKEN").map(|s| s.to_string()),
    })
    .await
    .unwrap();

    handle_command(cli.commands, state).await
}
