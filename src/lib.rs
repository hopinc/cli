pub mod commands;
pub mod config;
pub mod state;
pub mod store;
pub mod util;

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

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli() {
        use super::*;
        use clap::CommandFactory;

        CLI::command().debug_assert();
    }
}
