use clap::Parser;

use super::types::MultipleDeployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a deployment")]
pub struct DeleteOptions {
    #[clap(name = "name", help = "Name of the deployment")]
    name: Option<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle_delete(options: DeleteOptions, state: State) -> Result<(), std::io::Error> {
    let project_id = state.ctx.current_project_error().id;

    let deployments = state
        .http
        .request::<MultipleDeployments>(
            "GET",
            &format!("/ignite/deployments?project={}", project_id),
            None,
        )
        .await
        .expect("Error while getting deployments")
        .unwrap()
        .deployments;

    if deployments.is_empty() {
        panic!("No deployments found");
    }

    let deployment = match options.name {
        Some(name) => {
            let deployment = deployments
                .iter()
                .find(|p| p.name == name)
                .expect("Deployment not found");
            deployment.to_owned()
        }
        None => {
            let deployments_fmt = deployments
                .iter()
                .map(|d| format!("{} ({})", d.name, d.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to delete")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            deployments[idx].to_owned()
        }
    };

    if !options.force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to delete deployment {}?",
                deployment.name
            ))
            .interact_opt()
            .expect("Failed to confirm");

        if confirm.is_none() || !confirm.unwrap() {
            panic!("Aborted deletion of `{}`", deployment.name);
        }
    }

    state
        .http
        .request::<()>(
            "DELETE",
            format!(
                "/ignite/deployments/{}?project={}",
                deployment.id, project_id
            )
            .as_str(),
            None,
        )
        .await
        .expect("Error while deleting deployment");

    log::info!(
        "Deployment `{}` ({}) deleted",
        deployment.name,
        deployment.id
    );

    Ok(())
}
