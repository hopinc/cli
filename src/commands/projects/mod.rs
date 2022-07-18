mod create;
mod delete;
mod info;
mod list;
mod switch;
pub mod types;
pub mod util;

use clap::{Parser, Subcommand};

use self::create::{handle_create, CreateOptions};
use self::delete::{handle_delete, DeleteOptions};
use self::info::{handle_info, InfoOptions};
use self::list::{handle_list, ListOptions};
use self::switch::{handle_switch, SwitchOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "new", alias = "create")]
    Create(CreateOptions),
    Switch(SwitchOptions),
    Info(InfoOptions),
    #[clap(name = "ls", alias = "list")]
    List(ListOptions),
    #[clap(name = "rm", alias = "delete")]
    Delete(DeleteOptions),
}

#[derive(Debug, Parser)]
#[clap(name = "projects", about = "Interact with projects")]
pub struct ProjectsOptions {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle_projects(options: ProjectsOptions, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::List(options) => handle_list(options, state).await,
        Commands::Switch(options) => handle_switch(options, state).await,
        Commands::Delete(options) => handle_delete(options, state).await,
        Commands::Create(options) => handle_create(options, state).await,
        Commands::Info(options) => handle_info(options, state).await,
    }
}
