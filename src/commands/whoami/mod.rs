use clap::Parser;

use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get information about the current user")]
pub struct WhoamiOptions {}

pub async fn handle_whoami(_options: WhoamiOptions, state: State) -> Result<(), std::io::Error> {
    let me = state
        .ctx
        .me
        .clone()
        .expect("You are not logged in. Please run `hop auth login` first.");

    println!(
        "You are logged in as `{}` ({})",
        me.user.username, me.user.email
    );

    let project = state.ctx.current_project_error();

    println!(
        "Project: `{}` ({}) {}",
        project.name, project.id, project.p_type
    );

    Ok(())
}
