pub mod create;
mod delete;
mod get_env;
mod list;
pub mod rollout;
mod scale;
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
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
    #[clap(name = "rollout", alias = "rollouts")]
    Rollout(rollout::Options),
    #[clap(name = "update")]
    Update(update::Options),
    #[clap(name = "scale")]
    Scale(scale::Options),
    #[clap(name = "get-env")]
    GetEnv(get_env::Options),
    // alias for hop containers
    #[clap(name = "containers")]
    Containers(super::containers::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "ignite", about = "Interact with Ignite deployments")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => list::handle(options, state).await,
        Commands::Create(options) => create::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::Update(options) => update::handle(options, state).await,
        Commands::Rollout(options) => rollout::handle(options, state).await,
        Commands::Scale(options) => scale::handle(options, state).await,
        Commands::GetEnv(options) => get_env::handle(options, state).await,
        Commands::Containers(options) => super::containers::handle(options, state).await,
    }
}
