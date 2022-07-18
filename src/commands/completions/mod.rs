use std::fs::File;

use structopt::{clap::Shell, StructOpt};

use crate::state::State;
use crate::CLI;

#[derive(Debug, StructOpt)]
#[structopt(about = "Generate shell completions")]
pub struct CompletionsOptions {
    #[structopt(short = "s", long = "shell", help = "Your shell")]
    pub shell: Shell,
}

pub async fn handle_completions(
    options: CompletionsOptions,
    _state: State,
) -> Result<(), std::io::Error> {
    let mut cli = CLI::clap();

    match options.shell {
        Shell::Bash => {
            let mut file =
                File::create("/etc/bash_completion.d/hop").expect("Failed to create file");
            cli.gen_completions_to("hop", options.shell, &mut file);
        }
        Shell::Zsh => {
            let mut file =
                File::create("/usr/share/zsh/site-functions/_hop").expect("Failed to create file");
            cli.gen_completions_to("hop", Shell::Zsh, &mut file)
        }
        Shell::Fish => {
            let mut file = File::create("/usr/share/fish/completions/hop.fish")
                .expect("Failed to create file");

            cli.gen_completions_to("hop", Shell::Fish, &mut file);
        }

        _ => panic!("Unsupported shell"),
    }

    Ok(())
}
