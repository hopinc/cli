use structopt::StructOpt;

use super::types::MultipleDeployments;
use crate::state::State;

#[derive(Debug, StructOpt)]
#[structopt(about = "List all deployments")]
pub struct ListOptions {}

pub async fn handle_list(_options: ListOptions, state: State) -> Result<(), std::io::Error> {
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
        panic!("No deployments found in this project");
    }

    let deployments_fmt = deployments
        .iter()
        .map(|d| {
            format!(
                " {} ({}) - {} container(s)",
                d.name, d.id, d.container_count
            )
        })
        .collect::<Vec<_>>();

    println!("Deployments:");
    println!("{}", deployments_fmt.join("\n"));

    Ok(())
}
