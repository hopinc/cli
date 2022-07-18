mod cli;
mod commands;
mod config;
mod macros;
mod state;
mod store;
mod types;

use clap::Parser;
use cli::CLI;
use commands::handle_command;
use state::{State, StateOptions};

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
