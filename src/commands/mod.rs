mod auth;
mod project;
mod secrets;

use crate::state::State;
use auth::{handle_command as handle_auth, AuthOptions};
use project::{handle_command as handle_project, ProjectOptions};
use secrets::{handle_command as handle_secrets, SecretsOptions};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Auth(AuthOptions),
    Project(ProjectOptions),
    Secrets(SecretsOptions),
}

pub async fn handle_command(command: Commands, state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(option) => handle_auth(option.commands, state).await,
        Commands::Project(option) => handle_project(option.commands, state).await,
        Commands::Secrets(option) => handle_secrets(option.commands, state).await,
    }
}
