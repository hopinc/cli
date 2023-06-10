use anyhow::{bail, ensure, Result};
use clap::Parser;

use super::utils::{cancel_build, format_builds, get_all_builds};
use crate::commands::ignite::builds::types::BuildState;
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
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
            let project_id = state.ctx.current_project_error()?.id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

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
