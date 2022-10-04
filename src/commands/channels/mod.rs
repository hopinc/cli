mod create;
mod delete;
mod list;
mod messages;
mod tokens;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;

use crate::state::State;

#[derive(Debug, Parser)]
pub enum Commands {
    #[clap(name = "new", alias = "create")]
    Create(create::Options),
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    #[clap(name = "rm", alias = "delete")]
    Delete(delete::Options),
    Tokens(tokens::Options),
    #[clap(name = "messages", alias = "send", alias = "msg")]
    Messages(messages::Options),
}

#[derive(Debug, Parser)]
#[clap(about = "Interact with Channels")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Create(options) => create::handle(options, state).await,
        Commands::List(options) => list::handle(options, state).await,
        Commands::Delete(options) => delete::handle(options, state).await,
        Commands::Tokens(options) => tokens::handle(options, state).await,
        Commands::Messages(options) => messages::handle(options, state).await,
    }
}
