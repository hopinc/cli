use std::io::Write;

use anyhow::{ensure, Result};
use clap::Parser;
use console::Term;
use leap_client_rs::leap::types::Event;
use leap_client_rs::{LeapEdge, LeapOptions};

use super::types::ContainerEvents;
use super::utils::{format_containers, get_all_containers, get_container};
use crate::commands::containers::utils::format_single_metrics;
use crate::commands::ignite::utils::{format_deployments, get_all_deployments, get_deployment};
use crate::config::LEAP_PROJECT;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get metrics for a container")]
pub struct Options {
    #[clap(help = "ID of the container")]
    pub container: Option<String>,

    #[clap(short, long, help = "Show metrics in real time")]
    pub follow: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let (container, deployment) = if let Some(container_id) = options.container {
        let container = get_container(&state.http, &container_id).await?;
        let deployment = get_deployment(&state.http, &container.deployment_id).await?;

        (container, deployment)
    } else {
        let project_id = state.ctx.current_project_error()?.id;

        let deployments = get_all_deployments(&state.http, &project_id).await?;
        ensure!(!deployments.is_empty(), "No deployments found");
        let deployments_fmt = format_deployments(&deployments, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a deployment")
            .items(&deployments_fmt)
            .default(0)
            .interact()?;

        let deployment = deployments[idx].to_owned();

        let containers = get_all_containers(&state.http, &deployment.id).await?;
        ensure!(!containers.is_empty(), "No containers found");
        let containers_fmt = format_containers(&containers, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select container")
            .items(&containers_fmt)
            .default(0)
            .interact()?;

        (containers[idx].to_owned(), deployment)
    };

    let mut term = Term::stdout();

    writeln!(
        term,
        "{}",
        format_single_metrics(&container.metrics, &deployment)?.join("\n")
    )?;

    if !options.follow {
        return Ok(());
    }

    let mut leap = LeapEdge::new(LeapOptions {
        token: Some(&state.ctx.current.clone().unwrap().leap_token),
        project: &std::env::var("LEAP_PROJECT").unwrap_or_else(|_| LEAP_PROJECT.to_string()),
        ws_url: &std::env::var("LEAP_WS_URL")
            .unwrap_or_else(|_| LeapOptions::default().ws_url.to_string()),
    })
    .await?;

    while let Some(msg) = leap.listen().await {
        let capsuled = match msg {
            Event::Message(message) => message,

            _ => continue,
        };

        let Ok(container_events) = serde_json::from_value(serde_json::to_value(capsuled.data)?) else {
            continue;
        };

        let metrics = match container_events {
            ContainerEvents::ContainerMetricsUpdate {
                container_id,
                metrics,
            } => {
                if container_id != container.id {
                    continue;
                }

                metrics
            }
        };

        let metrics = format_single_metrics(&Some(metrics), &deployment)?;

        term.clear_last_lines(metrics.len())?;

        writeln!(term, "{}", metrics.join("\n"))?
    }

    Ok(())
}
