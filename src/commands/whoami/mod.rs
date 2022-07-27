use clap::Parser;

use crate::{config::EXEC_NAME, state::State};

#[derive(Debug, Parser)]
#[clap(about = "Get information about the current user")]
pub struct WhoamiOptions {}

pub async fn handle_whoami(_options: WhoamiOptions, state: State) -> Result<(), std::io::Error> {
    let me = state.ctx.me.clone().unwrap();

    log::info!(
        "You are logged in as `{}` ({})",
        me.user.username,
        me.user.email
    );

    let project = state.ctx.current_project();

    match project {
        Some(project) => {
            log::info!(
                "Project: `{}` ({}) {}",
                project.name,
                project.id,
                project.p_type
            );
        }
        None => {
            log::warn!(
                "No project is currently selected. Please run `{} project switch` first.",
                EXEC_NAME
            );
        }
    }

    Ok(())
}
