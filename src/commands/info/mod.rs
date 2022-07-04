use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "info", about = "Get information about the current user")]
pub struct InfoOptions {}

pub async fn handle_command(_options: InfoOptions, state: State) -> Result<(), std::io::Error> {
    let me = state
        .clone()
        .ctx
        .me
        .expect("You are not logged in. Please run `hop auth login` first.");

    println!(
        "You are logged in as `{}` ({})",
        me.user.username, me.user.email
    );

    if let Some(project) = state.ctx.current_project() {
        println!(
            "Project: `{}` ({}) {}",
            project.name, project.id, project.p_type
        );
    } else {
        panic!("No project selected. Run `hop project switch` to select one or use `--project` to specify a project");
    }

    Ok(())
}
