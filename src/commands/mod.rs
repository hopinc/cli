mod auth;

use crate::state::State;
use auth::{handle_command as handle_auth, AuthOptions};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Commands {
    #[structopt(name = "auth", about = "Authenticate with Hop")]
    Auth(AuthOptions),
}

pub async fn handle_command(command: Commands, state: State) -> Result<(), std::io::Error> {
    match command {
        Commands::Auth(option) => handle_auth(option.commands, state).await,
    }
}
