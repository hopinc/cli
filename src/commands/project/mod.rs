mod create;
mod delete;
mod ls;
mod switch;

use self::create::{handle_create, CreateOptions};
use self::delete::{handle_delete, DeleteOptions};
use self::ls::{handle_ls, LsOptions};
use self::switch::{handle_switch, SwitchOptions};
use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Commands {
    Ls(LsOptions),
    Switch(SwitchOptions),
    Create(CreateOptions),
    Delete(DeleteOptions),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project", about = "🐇 Interact with Hop via command line")]
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
            Commands::Ls(_) => handle_ls(state).await,
            Commands::Switch(_) => handle_switch(state).await,
            Commands::Delete(options) => handle_delete(options, state).await,
            Commands::Create(options) => handle_create(options, state).await,
        }
    } else {
        ProjectOptions::clap().print_long_help().unwrap();

        // newline to separate from the help output
        println!("");
        std::process::exit(1);
    }
}
