pub mod builds;
pub mod create;
mod delete;
pub mod from_compose;
mod get_env;
mod health;
mod list;
mod promote;
pub mod rollout;
mod scale;
mod templates;
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
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
    #[clap(alias = "rollouts")]
    Rollout(rollout::Options),
    Update(update::Options),
    Scale(scale::Options),
    #[clap(name = "get-env")]
    GetEnv(get_env::Options),
    #[clap(name = "from-compose")]
    FromCompose(from_compose::Options),
    #[clap(alias = "check")]
    Health(health::Options),
    Builds(builds::Options),
    #[clap(alias = "rollback")]
    Promote(promote::Options),
    Templates(templates::Options),
    // alias for hop containers
    Containers(super::containers::Options),
    Gateways(super::gateways::Options),
    Tunnel(super::tunnel::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Interact with Ignite deployments")]
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
        Commands::Health(options) => health::handle(options, state).await,
        Commands::Containers(options) => super::containers::handle(options, state).await,
        Commands::Gateways(options) => super::gateways::handle(options, state).await,
        Commands::Promote(options) => promote::handle(options, state).await,
        Commands::Builds(options) => builds::handle(options, state).await,
        Commands::FromCompose(options) => from_compose::handle(options, state).await,
        Commands::Tunnel(options) => super::tunnel::handle(&options, state).await,
        Commands::Templates(options) => templates::handle(options, state).await,
    }
}
