use anyhow::Result;
use clap::Parser;

use super::utils::{format_tokens, get_all_tokens};
use crate::state::State;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "List all Leap Tokens")]
pub struct Options {
    #[clap(short, long, help = "Only print the IDs of the Tokens")]
    quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error()?.id;
    let tokens = get_all_tokens(&state.http, &project_id).await?;

    if options.quiet {
        let ids = tokens
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let channels_fmt = format_tokens(&tokens, true);

        println!("{}", channels_fmt.join("\n"));
    }

    Ok(())
}
