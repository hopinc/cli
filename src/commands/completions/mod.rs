use std::io;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell as CompletionShell};

use crate::config::EXEC_NAME;
use crate::state::State;
use crate::CLI;

#[derive(Debug, Parser)]
#[clap(about = "Generate completion scripts for the specified shell")]
pub struct Options {
    #[clap(name = "shell", help = "The shell to print the completion script for")]
    shell: CompletionShell,
}

pub fn handle(options: Options, _state: State) {
    generate(
        options.shell,
        &mut CLI::command(),
        EXEC_NAME,
        &mut io::stdout().lock(),
    )
}
