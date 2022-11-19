pub mod commands;
pub mod config;
pub mod state;
pub mod store;
pub mod utils;

use clap::Parser;
use commands::Commands;

#[derive(Debug, Parser)]
#[clap(
    name = "hop",
    about = "🐇 Interact with Hop via command line",
    version,
    author
)]
pub struct CLI {
    #[clap(subcommand)]
    pub commands: Commands,

    #[clap(
        short,
        long,
        help = "Namespace or ID of the project to use",
        global = true
    )]
    pub project: Option<String>,

    #[clap(short = 'D', long, help = "Enable debug mode", global = true)]
    pub debug: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli() {
        use clap::CommandFactory;

        use super::*;

        CLI::command().debug_assert();
    }
}
