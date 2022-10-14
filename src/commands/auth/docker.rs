use std::process::Stdio;

use anyhow::{bail, Result};
use clap::Parser;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::state::State;
use crate::utils::in_path;

#[derive(Debug, Parser)]
#[clap(about = "Authenticate the current user with Docker")]
pub struct Options {}

pub async fn handle(_options: &Options, state: &mut State) -> Result<()> {
    if !in_path("docker").await {
        bail!("Docker is not installed");
    }

    state.login(None).await?;

    let current = state.ctx.current.take().unwrap();

    login(
        &current.email,
        state.auth.authorized.get(&current.id).unwrap(),
    )
    .await?;

    log::info!("Successfully logged in as `{}` with Docker", current.email);

    Ok(())
}

pub const HOP_REGISTRY_URL: &str = "registry.hop.io";

// This login is separated into two commands.
pub async fn login(username: &str, password: &str) -> Result<()> {
    // First we need to know if we are already logged in to the registry
    let status = Command::new("docker")
        .arg("login")
        .arg(HOP_REGISTRY_URL)
        // making the stdin piped disables tty
        .stdin(Stdio::piped())
        // .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status().await?;

    log::debug!("Docker login exited with {status}");

    // if the exit code is 0 we are already logged in
    if status.success() {
        log::debug!("Docker login successful");

        return Ok(());
    }

    login_new(username, password).await
}

pub async fn login_new(username: &str, password: &str) -> Result<()> {
    // if we are not logged in we need to login using the email and token (pat or bearer, ptk)
    let mut child = Command::new("docker")
        .arg("login")
        .arg("--username")
        .arg(username)
        .arg("--password-stdin")
        .arg(HOP_REGISTRY_URL)
        .stdin(Stdio::piped())
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        .spawn()?;

    log::debug!("Writing password to stdin");

    // write the password to the stdin
    child
        .stdin
        .take()
        .unwrap()
        .write_all(password.as_bytes())
        .await?;

    let status = child.wait().await?;

    log::debug!("Docker login exited with {status}");

    // if the exit code is 0 we are already logged in
    if status.success() {
        log::debug!("Docker login successful");

        return Ok(());
    }

    // if the command failed there are few possible reasons:
    // 1. docker daemon is not running
    // 2. registry authentication layer is down
    // 3. the users credentials just expired
    bail!("Docker login failed, is the docker daemon running?");
}
