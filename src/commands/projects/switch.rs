use anyhow::{Context, Result};
use clap::Parser;

use crate::commands::projects::utils::{format_project, format_projects};
use crate::state::State;
use crate::store::Store;

#[derive(Debug, Parser)]
#[clap(about = "Switch to a different project")]
#[group(skip)]
pub struct Options {
    #[clap(help = "Namespace or ID of the project to use")]
    pub project: Option<String>,
}

pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let projects = state.ctx.current.clone().unwrap().projects;

    let project = match options.project.clone() {
        Some(namespace) => state
            .ctx
            .find_project_by_id_or_namespace(&namespace)
            .with_context(|| format!("Project `{namespace}` not found"))?,
        None => {
            let projects_fmt = format_projects(&projects, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project")
                .items(&projects_fmt)
                .default(if let Some(current) = state.ctx.current_project() {
                    projects
                        .iter()
                        .position(|p| p.id == current.id)
                        .unwrap_or(0)
                } else {
                    0
                })
                .interact()?;

            projects[idx].clone()
        }
    };

    state.ctx.default_project = Some(project.id.clone());
    state.ctx.save().await?;

    log::info!("Switched to project {}", format_project(&project));

    Ok(())
}
