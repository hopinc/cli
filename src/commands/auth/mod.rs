mod list;
pub mod login;
mod logout;
mod switch;
pub mod types;
mod utils;

use clap::{Parser, Subcommand};

use self::list::{handle_list, ListOptions};
use self::login::{handle_login, LoginOptions};
use self::logout::{hanndle_logout, LogoutOptions};
use self::switch::{handle_switch, SwitchOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "ls", alias = "list")]
    List(ListOptions),
    Login(LoginOptions),
    Logout(LogoutOptions),
    Switch(SwitchOptions),
}

#[derive(Debug, Parser)]
#[clap(name = "auth", about = "Authenticate with Hop")]
pub struct AuthOptions {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle_auth(options: AuthOptions, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::List(options) => handle_list(options, state).await,
        Commands::Login(options) => handle_login(options, state).await,
        Commands::Logout(options) => hanndle_logout(options, state).await,
        Commands::Switch(options) => handle_switch(options, state).await,
    }
}
