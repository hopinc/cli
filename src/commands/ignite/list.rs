use anyhow::Result;
use clap::Parser;

use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all deployments")]
pub struct Options {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the deployments"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let deployments = get_all_deployments(&state.http, &project_id).await?;

    if options.quiet {
        let ids = deployments
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let deployments_fmt = format_deployments(&deployments, true);

        println!("{}", deployments_fmt.join("\n"));
    }

    Ok(())
}
