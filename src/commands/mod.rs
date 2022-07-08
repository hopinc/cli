mod auth;
mod deploy;
mod ignite;
mod projects;
mod secrets;
mod whoami;

use self::auth::{handle_auth, AuthOptions};
use self::deploy::{handle_deploy, DeployOptions};
use self::ignite::{handle_deployments, IgniteOptions};
use self::projects::{handle_projects, ProjectsOptions};
use self::secrets::{handle_secrets, SecretsOptions};
use self::whoami::{handle_whoami, WhoamiOptions};
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
    Ignite(IgniteOptions),
}

pub async fn handle_command(command: Commands, mut state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(options) => handle_auth(options, state).await,

        authorized_command => {
            // login so these commands can run
            state.login().await;

            match authorized_command {
                Commands::Auth(_) => unreachable!(),
                Commands::Projects(options) => handle_projects(options, state).await,
                Commands::Secrets(options) => handle_secrets(options, state).await,
                Commands::Deploy(options) => handle_deploy(options, state).await,
                Commands::Whoami(options) => handle_whoami(options, state).await,
                Commands::Ignite(options) => handle_deployments(options, state).await,
            }
        }
    }
}
