mod delete;
mod list;
mod set;
mod types;
pub mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "set", alias = "create", alias = "update", alias = "new")]
    Set(set::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "del", alias = "delete", alias = "remove")]
    Delete(delete::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "secrets", about = "Interact with secrets")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => list::handle(options, state).await,
        Commands::Set(options) => set::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
    }
}
