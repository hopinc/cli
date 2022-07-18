mod delete;
mod list;
pub mod types;
pub mod util;

use clap::{Parser, Subcommand};

use self::delete::{handle_delete, DeleteOptions};
use self::list::{handle_list, ListOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    // Info(InfoOptions),
    #[clap(name = "ls", alias = "list")]
    List(ListOptions),
    #[clap(name = "rm", alias = "delete")]
    Delete(DeleteOptions),
}

#[derive(Debug, Parser)]
#[clap(name = "ignite", about = "Interact with Ignite containers")]
pub struct IgniteOptions {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle_deployments(
    options: IgniteOptions,
    _state: State,
) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::List(options) => handle_list(options, _state).await,
        Commands::Delete(options) => handle_delete(options, _state).await,
    }
}
