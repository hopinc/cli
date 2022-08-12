mod create;
mod delete;
mod list;
pub mod types;
mod update;
pub mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "new", alias = "create")]
    Create(create::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    Update(update::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "gateways", about = "Interact with Ignite gateways")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
        Commands::Update(options) => update::handle(options, state).await,
    }
}
