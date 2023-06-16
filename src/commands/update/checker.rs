use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, ensure, Result};

use super::types::{GithubRelease, Version};
use crate::config::VERSION;
use crate::state::http::HttpClient;
use crate::store::context::Context;
use crate::store::Store;

pub const RELEASE_HOP_CLI_URL: &str = "https://api.github.com/repos/hopinc/hop_cli/releases";

pub async fn check_version(current: &Version, beta: bool) -> Result<(bool, Version)> {
    let http = HttpClient::new(None, None);

    let response = http
        .client
        .get(RELEASE_HOP_CLI_URL)
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

    let latest = if beta {
        // the latest release that can be prereleased
        data
            .iter()
            // skip drafts
            .find(|r| !r.draft)
            .map(|r| r.tag_name.clone())
            .ok_or_else(|| anyhow!("No prerelease found"))?
    } else {
        // the latest release that is not prereleased
        data
            .iter()
            // skip drafts and prereleases
            .find(|r| !r.prerelease && !r.draft)
            .map(|r| r.tag_name.clone())
            .ok_or_else(|| anyhow!("No release found"))?
    };

    let latest = Version::from_string(&latest)?;

    if latest.is_newer_than(current) {
        Ok((true, latest))
    } else {
        Ok((false, current.clone()))
    }
}

// static time to check for updates
const HOUR_IN_SECONDS: u64 = 60 * 60;

pub async fn version_notice(mut ctx: Context) -> Result<()> {
    let now = now_secs()?;

    let last_check = ctx
        .last_version_check
        .clone()
        .map(|(time, version)| (time.parse::<u64>().unwrap_or(now), version));

    let (last_checked, last_newest) = match last_check {
        Some(data) => data,
        // more than an hour to force check
        None => (now - HOUR_IN_SECONDS - 1, VERSION.to_string()),
    };

    let last_newest = Version::from_string(&last_newest)?;
    let current = Version::from_string(VERSION)?;

    let new_version = if now - last_checked > HOUR_IN_SECONDS {
        let (update, latest) = check_version(&current, false)
            .await
            .unwrap_or((false, current));

        ctx.last_version_check = Some((now.to_string(), latest.to_string()));
        ctx.save().await?;

        if !update {
            return Ok(());
        }

        latest
    } else if last_newest.is_newer_than(&current) {
        last_newest
    } else {
        // skip fs action
        return Ok(());
    };

    log::warn!("A new version is available: {new_version}");

    #[cfg(feature = "update")]
    log::warn!("Use `{}` to update", ctx.update_command());

    Ok(())
}

pub fn now_secs() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
