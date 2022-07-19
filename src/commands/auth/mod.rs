mod login;
mod logout;
pub mod types;

use clap::{Parser, Subcommand};

use self::login::{handle_login, LoginOptions};
use self::logout::{hanndle_logout, LogoutOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Login(LoginOptions),
    Logout(LogoutOptions),
}

#[derive(Debug, Parser)]
#[clap(name = "auth", about = "Authenticate with Hop")]
pub struct AuthOptions {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle_auth(options: AuthOptions, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::Login(options) => handle_login(options, state).await,
        Commands::Logout(options) => hanndle_logout(options, state).await,
    }
}
