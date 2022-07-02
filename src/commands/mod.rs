mod auth;
mod project;

use crate::state::State;
use auth::{handle_command as handle_auth, AuthOptions};
use project::{handle_command as handle_project, ProjectOptions};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Auth(AuthOptions),
    Project(ProjectOptions),
}

pub async fn handle_command(command: Commands, state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(option) => handle_auth(option.commands, state).await,
        Commands::Project(option) => handle_project(option.commands, state).await,
    }
}
