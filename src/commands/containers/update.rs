use anyhow::{ensure, Result};
use clap::Parser;

use super::types::ChangeableContainerState;
use super::utils::update_container_state;
use crate::commands::containers::utils::{format_containers, get_all_containers};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a container")]
pub struct Options {
    #[clap(help = "ID of the container")]
    container: Option<String>,

    #[clap(help = "State to set the container to")]
    state: Option<ChangeableContainerState>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let container = match options.container {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let containers = get_all_containers(&state.http, &deployments[idx].id).await?;
            let containers_fmt = format_containers(&containers, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a container")
                .default(0)
                .items(&containers_fmt)
                .interact_opt()?
                .expect("No containers selected");

            containers[idx].id.clone()
        }
    };

    let container_state = match options.state {
        Some(new_state) => new_state,

        None => {
            let items = ChangeableContainerState::values();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a state to set the container to")
                .default(0)
                .items(&items)
                .interact_opt()?
                .expect("No state selected");

            items[idx].clone()
        }
    };

    update_container_state(&state.http, &container, &container_state).await?;

    log::info!("Updated container `{}` to {}", container, container_state);

    Ok(())
}
