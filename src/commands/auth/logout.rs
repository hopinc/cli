use structopt::StructOpt;

use crate::state::State;
use crate::store::context::Context;

#[derive(Debug, StructOpt)]
#[structopt(name = "logout", about = "🔑 Logout the current user")]
pub struct LogoutOptions {}

pub async fn hanndle_logout(mut state: State) -> Result<(), std::io::Error> {
    let user_id = state.ctx.user;

    if user_id.is_none() {
        println!("You are not logged in");
        return Ok(());
    }

    // clear all state
    state.ctx = Context::default();
    state.ctx.save().await?;

    // remove the user from the store
    state.auth.authorized.remove(&user_id.unwrap());
    state.auth.save().await?;

    println!("You have been logged out");

    Ok(())
}
