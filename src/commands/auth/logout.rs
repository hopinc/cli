use anyhow::Result;
use clap::Parser;

use crate::state::State;
use crate::store::context::Context;

#[derive(Debug, Parser)]
#[clap(about = "Logout the current user")]
pub struct Options {}

pub async fn handle(_options: Options, mut state: State) -> Result<()> {
    let user_id = state.ctx.default_user;

    assert!(user_id.is_some(), "You are not logged in.");

    // clear all state
    state.ctx = Context::default();
    state.ctx.save().await?;

    // remove the user from the store
    state.auth.authorized.remove(&user_id.unwrap());
    state.auth.save().await?;

    log::info!("You have been logged out");

    Ok(())
}
