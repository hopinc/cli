mod browser_auth;
mod flags_auth;
mod types;
pub mod util;

use clap::Parser;

use self::browser_auth::browser_login;
use self::flags_auth::flags_login;
use crate::commands::auth::login::util::TokenType;
use crate::config::EXEC_NAME;
use crate::state::State;

const WEB_AUTH_URL: &str = "https://console.hop.io/cli-auth";
const PAT_FALLBACK_URL: &str = "https://console.hop.io/settings/pats";

#[derive(Debug, Parser, PartialEq, Default)]
#[clap(about = "Login to Hop")]
pub struct LoginOptions {
    #[clap(long = "token", help = "Project Token or Personal Authorization Token")]
    pub token: Option<String>,
    #[clap(long = "email", help = "Email")]
    pub email: Option<String>,
    #[clap(long = "password", help = "Password", min_values = 0)]
    pub password: Option<String>,
}

pub async fn handle_login(options: LoginOptions, state: State) -> Result<(), std::io::Error> {
    let token = if LoginOptions::default() == options {
        browser_login().await
    } else {
        flags_login(options, state.http.clone()).await
    };

    token_login(&token, state).await
}

pub async fn token_login(token: &String, mut state: State) -> Result<(), std::io::Error> {
    // for sanity fetch the user info
    state.login(Some(token.clone())).await;

    let authorized = state.ctx.current.clone().unwrap();

    // save the state
    state
        .auth
        .authorized
        .insert(authorized.id.clone(), token.clone());
    state.auth.save().await?;

    // if the token is a ptk save default project as assigne to the token
    match state.token_type {
        Some(TokenType::Ptk) => state.ctx.default_project = Some(authorized.id.clone()),
        _ => {
            log::info!(
                "Make sure to set the default project with `{} projects switch`",
                EXEC_NAME
            );
        }
    };

    state.ctx.default_user = Some(authorized.id.clone());
    state.ctx.save().await?;

    // output the login info
    log::info!(
        "Logged in as: `{}` ({})",
        authorized.name,
        authorized.email.unwrap_or(authorized.id)
    );

    Ok(())
}
