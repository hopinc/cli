mod attach;
mod delete;
mod list;
pub mod types;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "attach", alias = "new", alias = "create")]
    Attach(attach::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "del", alias = "delete", alias = "remove")]
    Delete(delete::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Interact with domains")]
#[group(skip)]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Attach(options) => attach::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
    }
}
