use clap::Parser;

use crate::commands::projects::util::format_projects;
use crate::done;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(name = "switch", about = "Switch to a different project")]
pub struct SwitchOptions {}

pub async fn handle_switch(
    _options: SwitchOptions,
    mut state: State,
) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .clone()
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if projects.is_empty() {
        panic!("No projects found");
    }

    let idx = match state.ctx.clone().project_override {
        Some(project) => projects
            .iter()
            .position(|p| p.id == project)
            .expect("Project not found"),
        None => {
            let projects_fmt = format_projects(&projects, &state.ctx.default_project);

            dialoguer::Select::new()
                .with_prompt("Select a project to set as default")
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

    done!(
        "Switched to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    Ok(())
}
