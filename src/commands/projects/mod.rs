mod create;
mod delete;
mod info;
mod list;
mod switch;
pub mod types;
pub mod util;

use clap::{Parser, Subcommand};

use self::create::{handle as handle_create, Options as CreateOptions};
use self::delete::{handle as handle_delete, Options as DeleteOptions};
use self::info::{handle as handle_info, Options as InfoOptions};
use self::list::{handle as handle_list, Options as ListOptions};
use self::switch::{handle as handle_switch, Options as SwitchOptions};
use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "new", alias = "create")]
    Create(CreateOptions),
    Switch(SwitchOptions),
    Info(InfoOptions),
    #[clap(name = "ls", alias = "list")]
    List(ListOptions),
    #[clap(name = "rm", alias = "delete")]
    Delete(DeleteOptions),
}

#[derive(Debug, Parser)]
#[clap(name = "projects", about = "Interact with projects")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> anyhow::Result<()> {
    match options.commands {
        Commands::Switch(options) => handle_switch(&options, state).await,
        Commands::Delete(options) => handle_delete(&options, state).await,
        Commands::Create(options) => handle_create(&options, state).await,
        Commands::List(options) => {
            handle_list(&options, state);
            Ok(())
        }

        Commands::Info(options) => {
            handle_info(&options, state);
            Ok(())
        }
    }
}
