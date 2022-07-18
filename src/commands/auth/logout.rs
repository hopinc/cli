use clap::Parser;

use crate::done;
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
        panic!("You are not logged in. Please run `hop auth login` first.");
    }

    // clear all state
    state.ctx = Context::default();
    state.ctx.save().await?;

    // remove the user from the store
    state.auth.authorized.remove(&user_id.unwrap());
    state.auth.save().await?;

    done!("You have been logged out");

    Ok(())
}
