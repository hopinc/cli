use clap::Parser;

use crate::config::EXEC_NAME;
use crate::state::State;
use crate::store::context::Context;

#[derive(Debug, Parser)]
#[clap(about = "Logout the current user")]
pub struct LogoutOptions {}

pub async fn hanndle_logout(
    _options: LogoutOptions,
    mut state: State,
) -> Result<(), std::io::Error> {
    let user_id = state.ctx.default_user;

    if user_id.is_none() {
        panic!(
            "You are not logged in. Please run `{} auth login` first.",
            EXEC_NAME
        );
    }

    // clear all state
    state.ctx = Context::default();
    state.ctx.save().await?;

    // remove the user from the store
    state.auth.authorized.remove(&user_id.unwrap());
    state.auth.save().await?;

    log::info!("You have been logged out");

    Ok(())
}
