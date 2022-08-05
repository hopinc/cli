mod delete;
mod list;
mod set;
mod types;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

use self::delete::{handle as handle_delete, Options as DeleteOptions};
use self::list::{handle as handle_list, Options as ListOptions};
use self::set::{handle as handle_set, Options as SetOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "set", alias = "create", alias = "update", alias = "new")]
    Set(SetOptions),
    #[clap(name = "ls", alias = "list")]
    List(ListOptions),
    #[clap(name = "rm", alias = "del", alias = "delete", alias = "remove")]
    Delete(DeleteOptions),
}

#[derive(Debug, Parser)]
#[structopt(name = "secrets", about = "Interact with secrets")]
pub struct Options {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::List(options) => handle_list(options, state).await,
        Commands::Set(options) => handle_set(options, state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
    }
}
