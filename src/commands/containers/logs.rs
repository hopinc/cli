use std::{env::temp_dir, time::Duration};

use anyhow::{ensure, Result};
use clap::Parser;
use tokio::{process::Command, time::sleep};

use crate::commands::{
    containers::utils::{format_containers, format_logs, get_all_containers, get_container_logs},
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
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let deployment = deployments[idx].clone();

            let containers = get_all_containers(&state.http, &deployment.id).await?;

            let containers_fmt = format_containers(&containers, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a container")
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
        // doesnt make sense to follow reversed logs
        if options.reverse && !options.follow {
            "asc"
        } else {
            "desc"
        },
    )
    .await?;

    if !options.follow {
        let temp = temp_dir().join("hop_ignite_logs-{container}.txt");

        tokio::fs::write(&temp, format_logs(&logs, false).join("\n")).await?;

        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "vim".to_string());

        log::info!("Opening logs in {editor}");

        Command::new(editor).arg(&temp).spawn()?.wait().await?;

        tokio::fs::remove_file(&temp).await?;
    } else {
        println!("{}", format_logs(&logs, true).join("\n"));

        let mut last_log_nonce = logs.last().map(|log| log.nonce.clone());

        // TODO: replace in the future with socket
        loop {
            let logs = get_container_logs(
                &state.http,
                &container,
                50, // max out the limit
                if options.reverse { "asc" } else { "desc" },
            )
            .await?;

            if !logs.is_empty() {
                let idx = logs
                    .iter()
                    .position(|log| {
                        if let Some(last_log_nonce) = last_log_nonce.clone() {
                            log.nonce == last_log_nonce
                        } else {
                            // quick get first log
                            true
                        }
                    })
                    .map(|idx| idx + 1)
                    .unwrap_or(0);

                let logs_to_display = logs.iter().skip(idx);

                // unstable to do .is_empty() on exact
                if logs_to_display.len() > 0 {
                    last_log_nonce = logs.last().map(|log| log.nonce.clone());

                    println!("{}", format_logs(&logs, true).join("\n"));
                }
            }

            sleep(Duration::from_millis(1_500)).await;
        }
    }

    Ok(())
}
