mod delete;
mod list;
mod set;
mod util;

use self::delete::{handle_delete, DeleteOptions};
use self::list::{handle_list, ListOptions};
use self::set::{handle_set, SetOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Set(SetOptions),
    #[structopt(name = "ls", alias = "list")]
    List(ListOptions),
    #[structopt(name = "rm", alias = "del", alias = "delete", alias = "remove")]
    Delete(DeleteOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "secrets", about = "Interact with secrets")]
pub struct SecretsOptions {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle_secrets(options: SecretsOptions, state: State) -> Result<(), std::io::Error> {
    state
        .clone()
        .ctx
        .current_project()
        .expect("No project selected run `hop project switch` to select one or use `--project` to specify a project");

    match options.commands {
        Commands::List(options) => handle_list(options, state).await,
        Commands::Set(options) => handle_set(options, state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
    }
}
