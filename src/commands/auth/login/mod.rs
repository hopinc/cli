mod browser_auth;
mod flags_auth;
mod types;
pub mod util;

use anyhow::Result;
use clap::Parser;

use self::browser_auth::browser_login;
use self::flags_auth::flags_login;
use crate::state::State;

const WEB_AUTH_URL: &str = "https://console.hop.io/cli-auth";
const PAT_FALLBACK_URL: &str = "https://console.hop.io/settings/pats";

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "Login to Hop")]
pub struct Options {
    #[clap(
        long = "token",
        help = "Project Token or Personal Authorization Token",
        long_help = "Project Token or Personal Authorization Token, you can use `--token=` to take the token from stdin"
    )]
    pub token: Option<String>,
    #[clap(long = "email", help = "Email")]
    pub email: Option<String>,
    #[clap(
        long = "password",
        help = "Password",
        long_help = "Password, you can use `--password=` to take the token from stdin"
    )]
    pub password: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    println!("{:?}", options.token);

    let init_token = if Options::default() != options {
        flags_login(options, state.http.clone()).await
    } else if let Ok(env_token) = std::env::var("HOP_TOKEN") {
        env_token
    } else {
        browser_login().await
    };

    token(&init_token, state).await
}

pub async fn token(token: &str, mut state: State) -> Result<()> {
    state.login(Some(token.to_string())).await?;

    // safe to unwrap here
    let authorized = state.ctx.current.clone().unwrap();

    if Some(authorized.id.clone()) == state.ctx.default_user {
        log::info!(
            "Nothing was changed. You are already logged in as: `{}` ({})",
            authorized.name,
            authorized.email.unwrap_or(authorized.id)
        );
        return Ok(());
    }

    // save the state
    state
        .auth
        .authorized
        .insert(authorized.id.clone(), token.to_string());
    state.auth.save().await?;

    state.ctx.default_project = None;
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
