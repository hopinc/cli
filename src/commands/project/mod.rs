mod ls;
mod switch;

use crate::state::State;
use structopt::StructOpt;

use self::{ls::handle_ls, switch::handle_switch};

#[derive(StructOpt, Debug)]
#[structopt(name = "hop project", about = "🐇 Interact with Hop via command line")]
pub enum Commands {
    #[structopt(name = "ls", about = "List all available projects")]
    Ls,
    #[structopt(name = "switch", about = "Switch to a different project")]
    Switch,
}

#[derive(StructOpt, Debug)]
pub struct ProjectOptions {
    #[structopt(subcommand)]
    pub commands: Option<Commands>,
}

pub async fn handle_command(command: Option<Commands>, state: State) -> Result<(), std::io::Error> {
    if state.ctx.user.is_none() {
        println!("You are not logged in. Please run `hop auth login` first.");
        std::process::exit(1);
    }

    if let Some(command) = command {
        match command {
            Commands::Ls => handle_ls(state).await,
            Commands::Switch => handle_switch(state).await,
        }
    } else {
        Commands::clap().print_help().unwrap();

        // newline to separate from the help output
        println!("");
        std::process::exit(1);
    }
}
