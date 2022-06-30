mod login;

use self::login::handle_login;
use crate::state::State;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hop auth", about = "Interact with Hop in a simple way")]
pub enum Commands {
    #[structopt(name = "login", about = "Login to Hop")]
    Login,
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
            Commands::Login => handle_login(state).await,
            Commands::Logout => {
                // TODO: remove panic test
                panic!("test")
            }
        }
    } else {
        Commands::clap().print_help().unwrap();

        // newline to separate from the help output
        println!("");
        std::process::exit(1);
    }
}
