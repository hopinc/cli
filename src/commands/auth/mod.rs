mod login;
mod logout;

use self::login::{handle_login, LoginOptions};
use self::logout::hanndle_logout;
use crate::state::State;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hop auth", about = "🐇 Interact with Hop via command line")]
pub enum Commands {
    #[structopt(name = "login", about = "Login to Hop")]
    Login(LoginOptions),
    #[structopt(name = "logout", about = "Logout the current user")]
    Logout,
}

#[derive(StructOpt, Debug)]
pub struct AuthOptions {
    #[structopt(subcommand)]
    pub commands: Option<Commands>,
}

pub async fn handle_command(command: Option<Commands>, state: State) -> Result<(), std::io::Error> {
    if let Some(command) = command {
        match command {
            Commands::Login(options) => handle_login(options, state).await,
            Commands::Logout => hanndle_logout(state).await,
        }
    } else {
        Commands::clap().print_help().unwrap();

        // newline to separate from the help output
        println!("");
        std::process::exit(1);
    }
}
