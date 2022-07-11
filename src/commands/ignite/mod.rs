mod delete;
mod list;
/// Re export of the `deploy` command.
pub mod types;
pub mod util;

use self::delete::{handle_delete, DeleteOptions};
use self::list::{handle_list, ListOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    // Info(InfoOptions),
    #[structopt(name = "ls", alias = "list")]
    List(ListOptions),
    #[structopt(name = "rm", alias = "delete")]
    Delete(DeleteOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "ignite", about = "Interact with Ignite containers")]
pub struct IgniteOptions {
    #[structopt(subcommand)]
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
