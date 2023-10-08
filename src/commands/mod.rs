pub mod auth;
mod channels;
mod completions;
pub mod containers;
pub mod deploy;
mod domains;
mod gateways;
pub mod ignite;
mod link;
mod oops;
mod payment;
pub mod projects;
mod secrets;
mod tunnel;
pub mod update;
mod volumes;
mod webhooks;
mod whoami;

use anyhow::Result;
use clap::Subcommand;
use ignite::from_compose;
use volumes::backup;

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth(auth::Options),
    #[clap(alias = "project")]
    Projects(projects::Options),
    #[clap(alias = "secret")]
    Secrets(secrets::Options),
    Deploy(deploy::Options),
    #[clap(alias = "info", alias = "ctx")]
    Whoami(whoami::Options),
    Ignite(ignite::Options),
    Link(link::Options),
    #[cfg(feature = "update")]
    Update(update::Options),
    #[clap(alias = "container", alias = "cts")]
    Containers(containers::Options),
    #[clap(alias = "gateway")]
    Gateways(gateways::Options),
    #[clap(alias = "domain")]
    Domains(domains::Options),
    #[clap(alias = "complete", hide = cfg!(not(feature = "update")))]
    Completions(completions::Options),
    #[clap(alias = "channel", alias = "ch")]
    Channels(channels::Options),
    Oops(oops::Options),
    #[clap(
        alias = "payments",
        alias = "finance",
        alias = "finances",
        alias = "billing"
    )]
    Payment(payment::Options),
    #[clap(alias = "fwd", alias = "forward")]
    Tunnel(tunnel::Options),
    #[clap(alias = "volume", alias = "v")]
    Volumes(volumes::Options),
    #[clap(alias = "webhook", alias = "wh")]
    Webhooks(webhooks::Options),
    #[clap(alias = "compose")]
    FromCompose(from_compose::Options),
    Backup(backup::Options),
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
                Commands::Oops(options) => oops::handle(&options, state).await,
                Commands::Tunnel(options) => tunnel::handle(&options, state).await,
                Commands::FromCompose(options) => from_compose::handle(options, state).await,
                Commands::Payment(options) => payment::handle(options, state).await,
                Commands::Volumes(options) => volumes::handle(options, state).await,
                Commands::Backup(options) => backup::handle(options, state).await,
                Commands::Webhooks(options) => webhooks::handle(options, state).await,
            }
        }
    }
}
