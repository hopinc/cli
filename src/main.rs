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
        help = "Namespace or ID of the project to use",
        global = true
    )]
    project: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // // setup a panic hook to easily exit the program on panic
    // std::panic::set_hook(Box::new(|panic_info| {
    //     // print the panic message
    //     let message = if let Some(message) = panic_info.payload().downcast_ref::<String>() {
    //         message.to_owned()
    //     } else if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
    //         message.to_string()
    //     } else {
    //         format!("{:?}", panic_info).to_string()
    //     };

    //     // add some color
    //     eprintln!("\x1b[31m\x1b[1merror:\x1b[0m {}", message);
    //     std::process::exit(1);
    // }));

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
