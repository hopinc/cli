use std::env::current_dir;
use std::path::PathBuf;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::ignite::util::{format_deployments, get_all_deployments, get_deployment};
use crate::commands::projects::util::format_project;
use crate::state::State;
use crate::store::hopfile::HopFile;

#[derive(Debug, Parser)]
#[clap(about = "Link an existing deployment to a hopfile")]
pub struct Options {
    #[clap(
        name = "dir",
        help = "Directory to link, defaults to current directory"
    )]
    path: Option<PathBuf>,

    #[clap(name = "deployment", help = "ID of the deployment")]
    deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let mut dir = current_dir()?;

    if let Some(path) = options.path {
        dir = dir.join(path).canonicalize()?;
    }

    ensure!(dir.is_dir(), "{dir:?} is not a directory");

    if HopFile::find(dir.clone()).await.is_some() {
        log::warn!("A hopfile was found in {dir:?}");
    }

    let project = state.ctx.current_project_error();

    log::info!("Project: {}", format_project(&project));

    let deployment = match options.deployment {
        Some(id) => get_deployment(&state.http, &id).await?,

        None => {
            let deployments = get_all_deployments(&state.http, &project.id).await?;

            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to link")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()?
                .expect("No deployment selected");

            deployments[idx].clone()
        }
    };

    HopFile::new(dir.join("hop.yml"), project.id, deployment.id.clone())
        .save()
        .await?;

    log::info!(
        "Deployment `{}` ({}) linked",
        deployment.name,
        deployment.id
    );

    Ok(())
}
