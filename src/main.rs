mod commands;
mod config;
mod macros;
mod state;
mod store;
mod types;

use clap::Parser;
use commands::{handle_command, Commands};
use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, LevelFilter};
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

    let colors = ColoredLevelConfig::new()
        .info(Color::BrightCyan)
        .error(Color::BrightRed)
        .warn(Color::BrightYellow)
        .debug(Color::BrightWhite);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let level = record.level();

            match level {
                Level::Debug => out.finish(format_args!(
                    "{} [{}]: {}",
                    colors.color(Level::Debug).to_string().to_lowercase(),
                    record.target(),
                    message
                )),

                level => out.finish(format_args!(
                    "{}: {}",
                    colors.color(level).to_string().to_lowercase(),
                    message
                )),
            }
        })
        .level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let state = State::new(StateOptions {
        override_project_id: cli.project,
        override_token: option_env!("HOP_TOKEN").map(|s| s.to_string()),
    })
    .await
    .unwrap();

    handle_command(cli.commands, state).await
}
