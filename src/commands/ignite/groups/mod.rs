mod create;
mod delete;
mod list;
mod r#move;
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
    #[clap(name = "move", alias = "add-deployment", alias = "add")]
    Move(r#move::Options),
    List(list::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Manage groups")]
#[group(skip)]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::Move(options) => r#move::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
    }
}
