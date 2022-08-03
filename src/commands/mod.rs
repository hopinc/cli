pub mod auth;
pub mod containers;
pub mod deploy;
pub mod ignite;
mod link;
pub mod projects;
mod secrets;
pub mod update;
mod whoami;

use clap::Subcommand;

use self::auth::{handle as handle_auth, Options as AuthOptions};
use self::deploy::{handle as handle_deploy, Options as DeployOptions};
use self::ignite::{handle as handle_ignite, Options as IgniteOptions};
use self::link::{handle as handle_link, Options as LinkOptions};
use self::projects::{handle as handle_projects, Options as ProjectsOptions};
use self::secrets::{handle as handle_secrets, Options as SecretsOptions};
use self::update::{handle as handle_update, Options as UpdateOptions};
use self::whoami::{handle as handle_whoami, Options as WhoamiOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth(AuthOptions),
    Projects(ProjectsOptions),
    Secrets(SecretsOptions),
    Deploy(DeployOptions),
    #[clap(name = "whoami", alias = "info", alias = "ctx")]
    Whoami(WhoamiOptions),
    Ignite(IgniteOptions),
    Link(LinkOptions),
    Update(UpdateOptions),
}

pub async fn handle_command(command: Commands, mut state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(options) => handle_auth(options, state).await,
        Commands::Update(options) => handle_update(options, state).await,

        authorized_command => {
            // login so these commands can run
            state.login(None).await;

            match authorized_command {
                Commands::Auth(_) | Commands::Update(_) => unreachable!(),
                Commands::Projects(options) => handle_projects(options, state).await,
                Commands::Secrets(options) => handle_secrets(options, state).await,
                Commands::Deploy(options) => handle_deploy(options, state).await,
                Commands::Whoami(options) => handle_whoami(&options, state),
                Commands::Ignite(options) => handle_ignite(options, state).await,
                Commands::Link(options) => handle_link(options, state).await,
            }
        }
    }
}
