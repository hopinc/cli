pub mod due;
pub mod list;
pub mod types;
pub mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    Due(due::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Manage payments")]
#[group(skip)]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => list::handle(&options, &state).await,
        Commands::Due(options) => due::handle(&options, &state).await,
    }
}
