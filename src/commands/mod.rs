pub mod auth;
mod channels;
mod completions;
pub mod containers;
pub mod deploy;
mod domains;
mod gateways;
pub mod ignite;
mod link;
pub mod projects;
mod secrets;
pub mod update;
mod whoami;

use anyhow::Result;
use clap::Subcommand;

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth(auth::Options),
    Projects(projects::Options),
    Secrets(secrets::Options),
    Deploy(deploy::Options),
    #[clap(name = "whoami", alias = "info", alias = "ctx")]
    Whoami(whoami::Options),
    Ignite(ignite::Options),
    Link(link::Options),
    #[cfg(feature = "update")]
    Update(update::Options),
    Containers(containers::Options),
    Gateways(gateways::Options),
    Domains(domains::Options),
    #[clap(name = "completions", alias = "complete", hide = cfg!(not(feature = "update")))]
    Completions(completions::Options),
    Channels(channels::Options),
}

pub async fn handle_command(command: Commands, mut state: State) -> Result<()> {
    match command {
        Commands::Auth(options) => auth::handle(options, state).await,
        #[cfg(feature = "update")]
        Commands::Update(options) => update::handle(options, state).await,
        Commands::Completions(options) => {
            completions::handle(options, state);
            Ok(())
        }

        authorized_command => {
            // login so these commands can run
            state.login(None).await?;

            match authorized_command {
                Commands::Auth(_) | Commands::Completions(_) => {
                    unreachable!()
                }

                #[cfg(feature = "update")]
                Commands::Update(_) => unreachable!(),

                Commands::Channels(options) => channels::handle(options, state).await,
                Commands::Projects(options) => projects::handle(options, state).await,
                Commands::Secrets(options) => secrets::handle(options, state).await,
                Commands::Deploy(options) => deploy::handle(options, state).await,
                Commands::Whoami(options) => whoami::handle(&options, state),
                Commands::Ignite(options) => ignite::handle(options, state).await,
                Commands::Link(options) => link::handle(options, state).await,
                Commands::Containers(options) => containers::handle(options, state).await,
                Commands::Gateways(options) => gateways::handle(options, state).await,
                Commands::Domains(options) => domains::handle(options, state).await,
            }
        }
    }
}
