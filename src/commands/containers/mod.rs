mod create;
mod delete;
mod list;
mod logs;
pub mod types;
mod update;
pub mod utils;

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
    #[clap(name = "logs", alias = "log")]
    Log(logs::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "ignite", about = "Interact with Ignite containers")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::Update(options) => update::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
        Commands::Log(options) => logs::handle(options, state).await,
    }
}
