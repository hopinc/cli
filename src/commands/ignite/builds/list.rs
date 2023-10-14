use anyhow::Result;
use clap::Parser;

use super::utils::{format_builds, get_all_builds};
use crate::{commands::ignite::groups::utils::fetch_grouped_deployments, state::State};

#[derive(Debug, Parser)]
#[clap(about = "List all builds in a deployment")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(short, long, help = "Only print the IDs of the builds")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
            let (deployments_fmt, deployments, validator) =
                fetch_grouped_deployments(&state, false, true).await?;

            let idx = loop {
                let idx = dialoguer::Select::new()
                    .with_prompt("Select a deployment")
                    .items(&deployments_fmt)
                    .default(0)
                    .interact()?;

                if let Ok(idx) = validator(idx) {
                    break idx;
                }

                console::Term::stderr().clear_last_lines(1)?
            };

            deployments[idx].id.clone()
        }
    };

    let builds = get_all_builds(&state.http, &deployment_id).await?;

    if options.quiet {
        let ids = builds
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let builds_fmt = format_builds(&builds, true);

        println!("{}", builds_fmt.join("\n"));
    }

    Ok(())
}
