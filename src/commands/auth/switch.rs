use clap::Parser;

use crate::state::State;

use super::utils::format_users;

#[derive(Debug, Parser)]
#[clap(about = "Switch to a different user")]
pub struct SwitchOptions {}

pub async fn handle_switch(_options: SwitchOptions, state: State) -> Result<(), std::io::Error> {
    let users = state.auth.authorized.keys().collect::<Vec<_>>();

    let users_fmt = format_users(&users, false);

    let idx = dialoguer::Select::new()
        .with_prompt("Select a user")
        .items(&users_fmt)
        .default(0)
        .interact_opt()
        .expect("Failed to select a user")
        .expect("Failed to select a user");

    let user_id = users.get(idx).unwrap().to_owned();

    super::login::token_login(state.auth.authorized.clone().get(user_id).unwrap(), state).await
}
