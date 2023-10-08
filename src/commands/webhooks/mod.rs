mod create;
mod delete;
mod list;
mod regenerate;
mod update;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "new", alias = "create", alias = "add")]
    Create(create::Options),
    #[clap(name = "update", alias = "edit")]
    Update(update::Options),
    #[clap(name = "rm", alias = "delete", alias = "del")]
    Delete(delete::Options),
    #[clap(name = "regenerate", alias = "regen")]
    Regenerate(regenerate::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Manage webhooks")]
#[group(skip)]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => list::handle(options, state).await,
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Update(options) => update::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::Regenerate(options) => regenerate::handle(options, state).await,
    }
}
