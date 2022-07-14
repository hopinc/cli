mod login;
mod logout;
pub mod types;

use structopt::StructOpt;

use self::login::{handle_login, LoginOptions};
use self::logout::{hanndle_logout, LogoutOptions};
use crate::state::State;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Login(LoginOptions),
    Logout(LogoutOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "auth", about = "Authenticate with Hop")]
pub struct AuthOptions {
    #[structopt(subcommand)]
    pub commands: Commands,
}

pub async fn handle_auth(options: AuthOptions, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::Login(options) => handle_login(options, state).await,
        Commands::Logout(options) => hanndle_logout(options, state).await,
    }
}
