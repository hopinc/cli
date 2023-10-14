use anyhow::Result;
use clap::Parser;

use crate::{
    commands::ignite::{groups::utils::fetch_grouped_deployments, utils::get_all_deployments},
    state::State,
};

#[derive(Debug, Parser)]
#[clap(about = "List all deployments")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "Only print the IDs of the deployments")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    if options.quiet {
        let project_id = state.ctx.current_project_error()?.id;

        let deployments = get_all_deployments(&state.http, &project_id).await?;

        let ids = deployments
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let deployments_fmt = fetch_grouped_deployments(&state, false, false).await?.0;

        println!("{}", deployments_fmt.join("\n"));
    }

    Ok(())
}
