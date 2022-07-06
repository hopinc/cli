mod auth;
mod deploy;
mod projects;
mod secrets;
mod whoami;

use self::auth::{handle_auth, AuthOptions};
use self::deploy::{handle_deploy, DeployOptions};
use self::projects::{handle_projects, ProjectsOptions};
use self::secrets::{handle_secrets, SecretsOptions};
use self::whoami::{handle_whoami as handle_info, WhoamiOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Auth(AuthOptions),
    Projects(ProjectsOptions),
    Secrets(SecretsOptions),
    Deploy(DeployOptions),
    #[structopt(name = "whoami", alias = "info", alias = "ctx")]
    Whoami(WhoamiOptions),
}

pub async fn handle_command(command: Commands, mut state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(options) => handle_auth(options, state).await,

        authorized_command => {
            // login so these commands can run
            state.login().await;

            match authorized_command {
                Commands::Auth(_) => unreachable!(),
                Commands::Projects(option) => handle_projects(option, state).await,
                Commands::Secrets(option) => handle_secrets(option, state).await,
                Commands::Deploy(option) => handle_deploy(option, state).await,
                Commands::Whoami(options) => handle_info(options, state).await,
            }
        }
    }
}
