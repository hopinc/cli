use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{create_health_check, create_health_check_config};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create Health Checks for a deployment")]
pub struct Options {
    #[clap(name = "deployment", help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(flatten)]
    pub health_check: self::HealthCheckCreate,
}

#[derive(Debug, Parser, PartialEq, Eq, Default)]
pub struct HealthCheckCreate {
    #[clap(long, help = "Port to check")]
    pub port: Option<u16>,

    #[clap(long, help = "Path to check")]
    pub path: Option<String>,

    #[clap(long, help = "Interval to check")]
    pub interval: Option<u64>,

    #[clap(long, help = "Timeout to check")]
    pub timeout: Option<u64>,

    #[clap(long = "max-retries", help = "Max retries of the check")]
    pub max_retries: Option<u64>,

    #[clap(long = "initial-delay", help = "Initial delay of the check")]
    pub initial_delay: Option<u64>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()?
                .ok_or_else(|| anyhow::anyhow!("No deployment selected"))?;

            deployments[idx].id.clone()
        }
    };

    let health_config = create_health_check_config(options.health_check)?;

    let health_check = create_health_check(&state.http, &deployment_id, health_config).await?;

    log::info!("Created Health Check `{}`", health_check.id);

    Ok(())
}
