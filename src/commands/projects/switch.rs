use anyhow::Result;
use clap::Parser;

use crate::commands::projects::util::{format_project, format_projects};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(name = "switch", about = "Switch to a different project")]
pub struct Options {
    #[clap(name = "project", help = "Namespace or ID of the project to use")]
    pub project: Option<String>,
}

pub async fn handle(options: &Options, mut state: State) -> Result<()> {
    let projects = state.ctx.current.clone().unwrap().projects;

    let project = match options.project.clone() {
        Some(namespace) => projects
            .iter()
            .find(|p| p.namespace == namespace || p.id == namespace)
            .expect("Project not found"),
        None => {
            let projects_fmt = format_projects(&projects, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project")
                .items(&projects_fmt)
                .default(if let Some(current) = state.ctx.clone().current_project() {
                    projects
                        .iter()
                        .position(|p| p.id == current.id)
                        .unwrap_or(0)
                } else {
                    0
                })
                .interact_opt()
                .expect("Failed to select project")
                .expect("No project selected");

            &projects[idx]
        }
    };

    state.ctx.default_project = Some(project.id.clone());
    state.ctx.save().await?;

    log::info!("Switched to project {}", format_project(project));

    Ok(())
}
