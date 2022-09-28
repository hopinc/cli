mod types;
mod util;

use std::path::PathBuf;

use anyhow::{bail, Result};
use tokio::fs;
use tokio::process::Command;

use crate::commands::auth::docker;
use crate::commands::deploy::local::util::install_nixpacks;
use crate::state::State;
use crate::store::utils::home_path;
use crate::util::in_path;

#[cfg(not(windows))]
const NIXPACKS_VENDORED_PATH: &str = ".hop/bin/nixpacks";

#[cfg(windows)]
const NIXPACKS_VENDORED_PATH: &str = ".hop/bin/nixpacks.exe";

pub async fn build(state: &State, image: &str, dir: PathBuf) -> Result<()> {
    if !in_path("docker").await {
        bail!("Docker is not installed, it is required to use nixpacks");
    }

    let current_user = state.ctx.current.clone().unwrap();

    docker::login(
        &current_user.email,
        state.auth.authorized.get(&current_user.id).unwrap(),
    )
    .await?;

    // if the dir has a dockerfile act like a normal docker build
    if fs::metadata(dir.join("Dockerfile")).await.is_ok() {
        let command = Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(image)
            .arg(dir)
            .status()
            .await?;

        if !command.success() {
            bail!(
                "Failed to build docker image: exit code {}",
                command.code().unwrap_or(1)
            );
        }
    } else {
        // if we do not have a dockerfile we need to build the image
        // ourselves using nixpacks that are vendored for hop

        let vendored_nixpacks_path = home_path(NIXPACKS_VENDORED_PATH);

        if fs::metadata(vendored_nixpacks_path.clone()).await.is_err() {
            log::warn!("Nixpacks binary not found, installing...");

            install_nixpacks(&vendored_nixpacks_path).await?;
        }

        let command = Command::new(vendored_nixpacks_path)
            .arg("build")
            .arg("-n")
            .arg(image)
            .arg(dir)
            .status()
            .await?;

        if !command.success() {
            bail!(
                "Failed to build docker image: exit code {}",
                command.code().unwrap_or(1)
            );
        }
    }

    println!();

    let command = Command::new("docker")
        .arg("push")
        .arg(image)
        .status()
        .await?;

    if !command.success() {
        bail!(
            "Failed to push image: exit code {}",
            command.code().unwrap_or(1)
        );
    }

    println!();
    log::info!("Pushed image `{image}`");

    Ok(())
}
