use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "whoami",
    about = "Get information about the current user",
    alias = "info",
    alias = "ctx"
)]
pub struct WhoamiOptions {}

pub async fn handle_whoami(_options: WhoamiOptions, state: State) -> Result<(), std::io::Error> {
    let me = state
        .clone()
        .ctx
        .me
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
