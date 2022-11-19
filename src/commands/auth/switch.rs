use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::format_users;
use crate::config::EXEC_NAME;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Switch to a different user")]
pub struct Options {}

pub async fn handle(_options: Options, state: State) -> Result<()> {
    let users = state.auth.authorized.keys().collect::<Vec<_>>();

    ensure!(
        !users.is_empty(),
        "You are not logged in into any accounts, run `{} auth login` to login",
        EXEC_NAME
    );

    let users_fmt = format_users(&users, false);

    let idx = dialoguer::Select::new()
        .with_prompt("Select a user")
        .items(&users_fmt)
        .default(0)
        .interact()?;

    let user_id = users.get(idx).unwrap().to_owned();

    super::login::token(state.auth.authorized.clone().get(user_id).unwrap(), state).await
}
