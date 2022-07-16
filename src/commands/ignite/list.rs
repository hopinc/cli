use structopt::StructOpt;

use crate::commands::ignite::util::{format_deployments, get_deployments};
use crate::state::State;

#[derive(Debug, StructOpt)]
#[structopt(about = "List all deployments")]
pub struct ListOptions {}

pub async fn handle_list(_options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let project_id = state.ctx.current_project_error().id;

    let deployments = get_deployments(state.http.clone(), project_id).await;

    if deployments.is_empty() {
        panic!("No deployments found in this project");
    }

    let deployments_fmt = format_deployments(&deployments);

    println!("Deployments:");
    println!("{}", deployments_fmt.join("\n"));

    Ok(())
}
