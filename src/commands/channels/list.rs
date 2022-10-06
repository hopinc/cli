use anyhow::Result;
use clap::Parser;

use super::utils::{format_channels, get_all_channels};
use crate::state::State;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "List all Channel")]
pub struct Options {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the Channels"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;
    let channels = get_all_channels(&state.http, &project_id).await?;

    if options.quiet {
        let ids = channels
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let channels_fmt = format_channels(&channels, true);

        println!("{}", channels_fmt.join("\n"));
    }

    Ok(())
}
