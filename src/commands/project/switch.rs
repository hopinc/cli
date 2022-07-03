use crate::state::State;
use crate::types::{Base, Projects};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hop project switch",
    about = "🚦 Switch to a different project"
)]
pub struct SwitchOptions {}

pub async fn handle_switch(mut state: State) -> Result<(), std::io::Error> {
    let projects = state
        .http
        .request::<Base<Projects>>("GET", "/users/@me", None)
        .await
        .expect("Error while getting project info")
        .unwrap()
        .data
        .projects;

    if projects.len() == 0 {
        panic!("No projects found");
    }

    let idx = match state.ctx.project.clone() {
        Some(project) => projects
            .iter()
            .position(|p| p.id == project)
            .expect("Project not found"),
        None => {
            let projects_fmt = projects
                .iter()
                .map(|p| format!("{} @{} ({})", p.name, p.namespace, p.id))
                .collect::<Vec<_>>();

            dialoguer::Select::new()
                .with_prompt(
                    "Select a project to set as default (use arrow keys and enter to select)",
                )
                .items(&projects_fmt)
                .default(if let Some(id) = state.ctx.default_project {
                    projects.iter().position(|p| p.id == id).unwrap_or(0)
                } else {
                    0
                })
                .interact_opt()
                .expect("Failed to select project")
                .expect("No project selected")
        }
    };

    let project = &projects[idx];

    state.ctx.default_project = Some(project.id.clone());
    state.ctx.save().await?;

    Ok(())
}
