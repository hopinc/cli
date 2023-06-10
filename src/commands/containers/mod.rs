mod create;
mod delete;
mod inspect;
mod list;
mod logs;
pub mod metrics;
mod recreate;
pub mod types;
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
    #[clap(name = "recreate")]
    Recreate(recreate::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(alias = "info")]
    Inspect(inspect::Options),
    #[clap(alias = "stats")]
    Metrics(metrics::Options),

    #[clap(name = "logs", alias = "log")]
    Log(logs::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Interact with Ignite containers")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
        Commands::Log(options) => logs::handle(options, state).await,
        Commands::Recreate(options) => recreate::handle(options, state).await,
        Commands::Inspect(options) => inspect::handle(options, state).await,
        Commands::Metrics(options) => metrics::handle(options, state).await,
    }
}
