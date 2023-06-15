pub(crate) mod commands;
pub(crate) mod config;
pub(crate) mod state;
pub(crate) mod store;
pub(crate) mod utils;

use anyhow::Result;
use clap::Parser;
use commands::update::version_notice;
use commands::{handle_command, Commands};
use config::{ARCH, PLATFORM, VERSION};
use state::{State, StateOptions};

#[derive(Debug, Parser)]
#[clap(
    name = "hop",
    about = "🐇 Interact with Hop via command line",
    version,
    author
)]
pub struct CLI {
    #[clap(subcommand)]
    pub commands: Commands,

    #[clap(
        short,
        long,
        help = "Namespace or ID of the project to use",
        global = true
    )]
    pub project: Option<String>,

    #[clap(short = 'D', long, help = "Enable debug mode", global = true)]
    pub debug: bool,
}

pub async fn run() -> Result<()> {
    // create a new CLI instance
    let cli = CLI::parse();

    // setup panic hook
    utils::set_hook();

    utils::logs(cli.debug);

    // in the debug mode, print the version and arch for easier debugging
    log::debug!("Hop-CLI v{VERSION} build for {ARCH}-{PLATFORM}");

    utils::sudo::fix().await?;

    let mut state = State::new(StateOptions {
        override_project: std::env::var("PROJECT_ID").ok().or(cli.project),
        override_token: std::env::var("TOKEN").ok(),
    })
    .await?;

    match cli.commands {
        #[cfg(feature = "update")]
        Commands::Update(_) => None,

        // do not show the notice if we are in completions mode
        // since it could break the shell
        Commands::Completions(_) => None,

        // only show the notice if we are not in debug mode or in CI
        _ if cfg!(debug_assertions) || state.is_ci => None,

        // its okay for the notice to fail
        _ => version_notice(&mut state.ctx).await.ok(),
    };

    if let Err(error) = handle_command(cli.commands, state).await {
        log::error!("{error}");
        log::debug!("{error:#?}");
        std::process::exit(1);
    }

    utils::clean_term();

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_cli() {
        use clap::CommandFactory;

        use super::*;

        CLI::command().debug_assert();
    }
}
