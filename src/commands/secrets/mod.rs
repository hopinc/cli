mod delete;
mod list;
mod set;
mod types;
mod util;

use clap::{Parser, Subcommand};

use self::delete::{handle_delete, DeleteOptions};
use self::list::{handle_list, ListOptions};
use self::set::{handle_set, SetOptions};
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
pub struct SecretsOptions {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle_secrets(options: SecretsOptions, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::List(options) => handle_list(options, state).await,
        Commands::Set(options) => handle_set(options, state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
    }
}
