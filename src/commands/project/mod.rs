mod create;
mod delete;
mod list;
mod switch;

use self::create::{handle_create, CreateOptions};
use self::delete::{handle_delete, DeleteOptions};
use self::list::{handle_list, ListOptions};
use self::switch::{handle_switch, SwitchOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    List(ListOptions),
    Switch(SwitchOptions),
    Create(CreateOptions),
    Delete(DeleteOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project", about = "🗺️ Interact with projects")]
pub struct ProjectOptions {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle_command(command: Commands, state: State) -> Result<(), std::io::Error> {
    state
        .clone()
        .ctx
        .user
        .expect("You are not logged in. Please run `hop auth login` first.");

    match command {
        Commands::List(_) => handle_list(state).await,
        Commands::Switch(_) => handle_switch(state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
        Commands::Create(options) => handle_create(options, state).await,
    }
}
