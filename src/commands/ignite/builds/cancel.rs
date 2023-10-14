use anyhow::{bail, ensure, Result};
use clap::Parser;

use super::utils::{cancel_build, format_builds, get_all_builds};
use crate::commands::ignite::builds::types::BuildState;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Cancel a running build")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub build: Option<String>,

    #[clap(short, long, help = "Skip confirmation")]
    pub force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let build_id = match options.build {
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

            let builds = get_all_builds(&state.http, &deployments[idx].id)
                .await?
                .into_iter()
                .filter(|b| matches!(b.state, BuildState::Pending))
                .collect::<Vec<_>>();
            ensure!(!builds.is_empty(), "No running builds found");
            let builds_fmt = format_builds(&builds, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a build")
                .items(&builds_fmt)
                .default(0)
                .interact()?;

            builds[idx].id.clone()
        }
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to cancel this build?")
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted by user");
    }

    cancel_build(&state.http, &build_id).await?;

    log::info!("Build `{build_id}` cancelled");

    Ok(())
}
