mod auth;
mod info;
mod projects;
mod secrets;

use self::auth::{handle_command as handle_auth, AuthOptions};
use self::info::{handle_command as handle_info, InfoOptions};
use self::projects::{handle_command as handle_project, ProjectsOptions};
use self::secrets::{handle_command as handle_secrets, SecretsOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Auth(AuthOptions),
    Projects(ProjectsOptions),
    Secrets(SecretsOptions),
    #[structopt(name = "info", alias = "ctx")]
    Info(InfoOptions),
}

pub async fn handle_command(command: Commands, mut state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(options) => handle_auth(options.commands, state).await,

        authorized_command => {
            // login so these commands can run
            state.login().await;

            match authorized_command {
                Commands::Projects(option) => handle_project(option.commands, state).await,
                Commands::Secrets(option) => handle_secrets(option.commands, state).await,
                Commands::Info(option) => handle_info(option, state).await,
                _ => unreachable!(),
            }
        }
    }
}
