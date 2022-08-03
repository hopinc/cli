use clap::Parser;

use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get information about a project")]
pub struct Options {}

pub fn handle(_options: &Options, state: State) {
    let project = state.ctx.current_project_error();

    println!(
        "Project: `{}` /{} ({}) {}",
        project.name, project.namespace, project.id, project.p_type
    );
}
