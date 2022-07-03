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
    List(ListOptions),
    Update(SetOptions),
    Set(SetOptions),
    Delete(DeleteOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "hop secrets", about = "🔏 Interact with secrets")]
pub struct SecretsOptions {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle_command(command: Commands, state: State) -> Result<(), std::io::Error> {
    state
        .clone()
        .ctx
        .user
        .expect("You are not logged in. Please run `hop auth login` first.");

    state
        .clone()
        .ctx
        .current_project()
        .expect("No project selected run `hop project switch` to select one or use `--project` to specify a project");

    match command {
        Commands::List(_) => handle_list(state).await,
        Commands::Update(options) => handle_set(options, state).await,
        Commands::Set(options) => handle_set(options, state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
    }
}
