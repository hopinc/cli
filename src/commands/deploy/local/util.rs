use std::path::PathBuf;
use std::vec;

use anyhow::{anyhow, bail, ensure, Result};
use tokio::fs;

use crate::commands::update::types::GithubRelease;
use crate::commands::update::util::{download, execute_commands, swap_exe_command, unpack};
use crate::config::ARCH;
use crate::state::http::HttpClient;

const RELEASE_NIXPACKS_URL: &str = "https://api.github.com/repos/hopinc/nixpacks/releases";
const BASE_NIXPACKS_URL: &str = "https://github.com/hopinc/nixpacks/releases/download";

pub async fn install_nixpacks(path: &PathBuf) -> Result<()> {
    log::debug!("Install nixpacks to {path:?}");

    let http = HttpClient::new(None, None);

    let response = http
        .client
        .get(RELEASE_NIXPACKS_URL)
        .send()
        .await
        .map_err(|_| anyhow!("Failed to get latest release"))?;

    ensure!(
        response.status().is_success(),
        "Failed to get latest release from Github: {}",
        response.status()
    );

    let data = response
        .json::<Vec<GithubRelease>>()
        .await
        .map_err(|_| anyhow!("Failed to parse Github release"))?;

    let version = &data.first().unwrap().tag_name;

    let platform = get_nixpacks_platform()?;

    let packed = download(
        &http,
        BASE_NIXPACKS_URL,
        version,
        &format!("nixpacks-{version}-{ARCH}-{platform}"),
    )
    .await?;

    let unpacked = unpack(&packed, "nixpacks").await?;

    fs::remove_file(&packed).await.ok();

    let mut elevated = vec![];
    let mut non_elevated = vec![];

    let parent = path.parent().unwrap().to_path_buf();

    if fs::create_dir_all(&parent).await.is_err() {
        elevated.push(format!("mkdir -p {}", parent.display()).into());
    }

    swap_exe_command(&mut non_elevated, &mut elevated, path.clone(), unpacked).await;
    execute_commands(&non_elevated, &elevated).await?;

    Ok(())
}

fn get_nixpacks_platform() -> Result<&'static str> {
    match sys_info::os_type()?.to_lowercase().as_str() {
        "linux" => Ok("unknown-linux-musl"),
        "darwin" => Ok("apple-darwin"),
        "windows" => Ok("pc-windows-msvc"),
        _ => bail!("Unsupported platform"),
    }
}
