use clap::Parser;

use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get information about a project")]
pub struct InfoOptions {}

pub async fn handle_info(_options: InfoOptions, state: State) -> Result<(), std::io::Error> {
    let project = state.ctx.current_project_error();

    println!(
        "Project: `{}` /{} ({}) {}",
        project.name, project.namespace, project.id, project.p_type
    );

    Ok(())
}
