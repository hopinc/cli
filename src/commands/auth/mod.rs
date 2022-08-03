mod list;
pub mod login;
mod logout;
mod switch;
pub mod types;
mod utils;

use clap::{Parser, Subcommand};

use self::list::{handle as handle_list, Options as ListOptions};
use self::login::{handle as handle_login, Options as LoginOptions};
use self::logout::{handle as handle_logout, Options as LogoutOptions};
use self::switch::{handle as handle_switch, Options as SwitchOptions};
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
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<(), std::io::Error> {
    match options.commands {
        Commands::Login(options) => handle_login(options, state).await,
        Commands::Logout(options) => handle_logout(options, state).await,
        Commands::Switch(options) => handle_switch(options, state).await,
        Commands::List(options) => {
            handle_list(&options, &state);
            Ok(())
        }
    }
}
