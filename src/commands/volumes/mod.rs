mod copy;
mod delete;
mod list;
mod types;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "cp", alias = "copy")]
    Copy(copy::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Interact with Volumes")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => list::handle(options, state).await,
        Commands::Copy(options) => copy::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
    }
}
