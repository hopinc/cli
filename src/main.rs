mod commands;
mod config;
mod state;
mod store;
mod types;

use commands::{handle_command, Commands};
use state::{State, StateOptions};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop", about = "🐇 Interact with Hop via command line")]
struct CLI {
    #[structopt(subcommand)]
    commands: Commands,

    #[structopt(
        short = "p",
        long = "project",
        help = "Override the default project used for all commands",
        global = true
    )]
    project: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // setup a panic hook to easily exit the program on panic
    std::panic::set_hook(Box::new(|panic_info| {
        // print the panic message
        if let Some(error) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("{}", error);
        } else {
            eprintln!("Unknown error: {}", panic_info);
        }
    }));

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
