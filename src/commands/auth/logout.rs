use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use serde_json::Value;

use crate::state::http::HttpClient;
use crate::state::State;
use crate::store::context::Context;

#[derive(Debug, Parser)]
#[clap(about = "Logout the current user")]
pub struct Options {}

pub async fn handle(_options: Options, mut state: State) -> Result<()> {
    let user_id = state.ctx.default_user;

    ensure!(user_id.is_some(), "You are not logged in.");

    invalidate_token(
        &state.http,
        state
            .auth
            .authorized
            .get(user_id.as_ref().unwrap())
            .unwrap(),
    )
    .await?;

    // clear all state
    state.ctx = Context::default();
    state.ctx.save().await?;

    // remove the user from the store
    state.auth.authorized.remove(user_id.as_ref().unwrap());
    state.auth.save().await?;

    log::info!("You have been logged out");

    Ok(())
}

async fn invalidate_token(http: &HttpClient, token: &str) -> Result<()> {
    match token.split('_').next() {
        Some("bearer") => {
            http.request::<Value>("POST", "/auth/logout", None).await?;
        }

        Some("pat") => {
            http.request::<Value>("DELETE", &format!("/users/@me/pats/{token}"), None)
                .await?;
        }

        Some("ptk") => {
            log::warn!("Project tokens are not invalidated on logout, please revoke them manually.");
        }

        _ => {
            return Err(anyhow!("Unknown token type"));
        }
    }

    Ok(())
}
