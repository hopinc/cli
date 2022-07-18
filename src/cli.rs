use crate::commands::Commands;
use clap::Parser;

#[derive(Debug, Parser)]
#[structopt(name = "hop", about = "🐇 Interact with Hop via command line")]
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
}
