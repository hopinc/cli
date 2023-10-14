use anyhow::Result;
use clap::Parser;

use crate::commands::ignite::groups::utils::format_groups;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all Ignite groups")]
#[group(skip)]
pub struct Options {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the deployments"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let groups = state.hop.ignite.groups.get_all(&project.id).await?;

    if options.quiet {
        let ids = groups
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let formated = format_groups(&groups)?;

        println!("{}", formated.join("\n"))
    }

    Ok(())
}
