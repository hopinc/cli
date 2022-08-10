use std::time::Duration;

use anyhow::{ensure, Result};
use clap::Parser;
use tokio::time::sleep;

use crate::commands::{
    containers::utils::{format_containers, format_log, get_all_containers, get_container_logs},
    ignite::util::{format_deployments, get_all_deployments},
};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get logs of a container")]
pub struct Options {
    #[clap(name = "containers", help = "ID of the container")]
    container: Option<String>,

    #[clap(short = 'f', long = "follow", help = "Follow the logs")]
    follow: bool,

    #[clap(
        short = 'n',
        long = "lines",
        help = "Number of lines to show",
        default_value = "10"
    )]
    lines: u64,

    #[clap(short = 'r', long = "reverse", help = "Show the newest entries first")]
    reverse: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let container = match options.container {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;

            ensure!(!deployments.is_empty(), "No deployments found");

            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment to list containers of")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let deployment = deployments[idx].clone();

            let containers = get_all_containers(&state.http, &deployment.id).await?;

            let containers_fmt = format_containers(&containers, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a container to get logs of")
                .default(0)
                .items(&containers_fmt)
                .interact_opt()?
                .expect("No containers selected");

            containers[idx].id.clone()
        }
    };

    // initial logs
    let logs = get_container_logs(
        &state.http,
        &container,
        options.lines,
        0,
        if options.reverse { "asc" } else { "desc" },
    )
    .await?
    .iter()
    .map(format_log)
    .collect::<Vec<_>>();

    println!("{}", logs.join("\n"));

    if options.follow {
        let mut log_count = logs.len() as u64;

        loop {
            let logs = get_container_logs(
                &state.http,
                &container,
                50, // max out the limit
                log_count,
                if options.reverse { "asc" } else { "desc" },
            )
            .await?;

            if !logs.is_empty() {
                log_count += logs.len() as u64;

                println!(
                    "{}",
                    logs.iter().map(format_log).collect::<Vec<_>>().join("\n")
                );
            }

            sleep(Duration::from_millis(1_500)).await;
        }
    }

    Ok(())
}
