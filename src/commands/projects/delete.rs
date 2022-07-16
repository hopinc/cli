use structopt::StructOpt;

use crate::done;
use crate::state::State;

static CONFIRM_DELETE_PROJECT_MESSAGE: &str = "I am sure I want to delete the project named ";

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a project")]
pub struct DeleteOptions {
    #[structopt(name = "namespace", help = "Namespace of the project")]
    namespace: Option<String>,
    #[structopt(long = "no-confirm", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle_delete(options: DeleteOptions, mut state: State) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .clone()
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if projects.is_empty() {
        panic!("No projects found");
    }

    let project = match options.namespace {
        Some(namespace) => {
            let project = projects
                .iter()
                .find(|p| p.namespace == namespace)
                .expect("Project not found");
            project.to_owned()
        }
        None => {
            let projects_fmt = projects
                .iter()
                .map(|p| format!("{} /{} ({})", p.name, p.namespace, p.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a project to delete")
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

            projects[idx].to_owned()
        }
    };

    if !options.force {
        println!(
            "To confirm, input the following message `{}{}`",
            CONFIRM_DELETE_PROJECT_MESSAGE, project.name
        );
        let output = dialoguer::Input::<String>::new()
            .with_prompt("Message")
            .interact()
            .expect("Failed to confirm deletion");

        if output != format!("{}{}", CONFIRM_DELETE_PROJECT_MESSAGE, project.name) {
            panic!("Aborted deletion of `{}`", project.name);
        }
    }

    state
        .http
        .request::<()>("DELETE", format!("/projects/{}", project.id).as_str(), None)
        .await
        .expect("Error while deleting project");

    if state.ctx.default_project == Some(project.id.to_string()) {
        state.ctx.default_project = None;
        state.ctx.save().await?;
    }

    done!("Project `{}` ({}) deleted", project.name, project.namespace);

    Ok(())
}
