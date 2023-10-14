use anyhow::{bail, ensure, Result};
use clap::Parser;

use super::utils::{delete_health_check, format_health_checks, get_all_health_checks};
use crate::{commands::ignite::groups::utils::fetch_grouped_deployments, state::State};

#[derive(Debug, Parser)]
#[clap(about = "Delete a Health Check")]
#[group(skip)]
pub struct Options {
    #[clap(name = "heath-checks", help = "IDs of the Health Check")]
    pub health_checks: Vec<String>,

    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let health_checks = if !options.health_checks.is_empty() {
        options.health_checks
    } else {
        let (deployments_fmt, deployments, validator) =
            fetch_grouped_deployments(&state, false, true).await?;

        let idx = loop {
            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            if let Ok(idx) = validator(idx) {
                break idx;
            }

            console::Term::stderr().clear_last_lines(1)?
        };

        let health_checks = get_all_health_checks(&state.http, &deployments[idx].id).await?;
        ensure!(!health_checks.is_empty(), "No health checks found");
        let health_checks_fmt = format_health_checks(&health_checks, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select a health check")
            .items(&health_checks_fmt)
            .interact()?;

        health_checks
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} Health Checks?",
                health_checks.len()
            ))
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    let mut delete_count = 0;

    for health_check in &health_checks {
        log::info!("Deleting Health Check `{}`", health_check);

        if let Err(err) = delete_health_check(&state.http, health_check).await {
            log::error!("Failed to delete Health Check `{}`: {}", health_check, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!(
        "Deleted {delete_count}/{} Health Check",
        health_checks.len()
    );

    Ok(())
}
