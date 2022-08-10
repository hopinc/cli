mod create;
mod delete;
pub mod info;
mod list;
mod switch;
pub mod types;
pub mod util;

use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "new", alias = "create")]
    Create(create::Options),
    Switch(switch::Options),
    Info(info::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "projects", about = "Interact with projects")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> anyhow::Result<()> {
    match options.commands {
        Commands::Switch(options) => switch::handle(&options, state).await,
        Commands::Delete(options) => delete::handle(&options, state).await,
        Commands::Create(options) => create::handle(&options, state).await,

        Commands::List(options) => {
            list::handle(&options, state);
            Ok(())
        }

        Commands::Info(options) => {
            info::handle(&options, state);
            Ok(())
        }
    }
}
