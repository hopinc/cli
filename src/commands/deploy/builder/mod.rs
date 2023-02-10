mod types;
mod util;

use std::path::PathBuf;

use anyhow::{bail, Result};
use leap_client_rs::leap::types::Event;
use leap_client_rs::LeapEdge;
use tokio::sync::mpsc::unbounded_channel;
use tokio::{fs, spawn};

use self::types::BuildEvents;
use self::util::{builder_post, compress};
use crate::commands::deploy::builder::types::BuildStatus;
use crate::commands::ignite::builds::utils::cancel_build;
use crate::state::State;
use crate::utils::urlify;

pub async fn build(
    state: &State,
    project_id: &str,
    deployment_id: &str,
    dir: PathBuf,
    leap: &mut LeapEdge,
) -> Result<()> {
    // deployment id is used not to colide if the user is deploying multiple items
    let packed = compress(deployment_id, dir).await?;

    log::info!("Packed to: {packed}");

    let bytes = fs::read(packed.clone()).await?;

    log::info!("Uploading...");

    let build = builder_post(&state.http, deployment_id, bytes).await?;

    let (tx, mut rx) = unbounded_channel();

    let http = state.http.clone();
    let build_id = build.id.clone();

    spawn(async move {
        loop {
            match rx.recv().await {
                Some("CANCEL") => {
                    log::info!("Cancelling build...");

                    if cancel_build(&http, &build_id).await.is_ok() {
                        log::info!("Build cancelled by user");
                    } else {
                        log::error!("Failed to cancel build");
                    }

                    std::process::exit(1);
                }

                Some("OK") => break,

                _ => {}
            }
        }
    });

    let ctrlc = tx.clone();

    ctrlc::set_handler(move || {
        ctrlc.send("CANCEL").ok();
    })?;

    log::info!("Deleting archive...");
    fs::remove_file(packed).await?;

    log::info!("From Hop builder:");

    while let Some(event) = leap.listen().await {
        if let Event::Message(capsuled) = event {
            if Some(project_id) != capsuled.channel.as_deref() {
                continue;
            }

            let Ok(build_data) = serde_json::from_value(serde_json::to_value(capsuled.data)?) else {
                continue;
            };

            match build_data {
                BuildEvents::BuildCreate(build_create) => {
                    if build_create.build.id == build.id {
                        println!("Validating build...");
                    }
                }

                BuildEvents::BuildUpdate(build_update) => {
                    if build_update.build.id == build.id {
                        match build_update.build.state {
                            // initial state from create
                            BuildStatus::Validating => {}

                            BuildStatus::Pending => {
                                println!("Build has been successfully validated, building...");
                            }

                            BuildStatus::ValidationFailed => {
                                tx.send("OK").ok();
                                leap.close().await;

                                // this **should** be present if the status is validation failed
                                let error = build_update.build.validation_failure.unwrap();

                                bail!(
                                    "Build validation failed: {} Visit {} for more information",
                                    error.reason,
                                    urlify(&error.help_link)
                                );
                            }
                        }
                    }
                }

                BuildEvents::BuildProgress(build_progress) => {
                    if build_progress.build_id == build.id {
                        print!("{}", build_progress.log);
                    }
                }

                BuildEvents::BuildCancelled(build_cancelled) => {
                    if build_cancelled.build_id == build.id {
                        tx.send("OK").ok();
                        leap.close().await;

                        bail!("Build cancelled");
                    }
                }

                BuildEvents::PushSuccess(build_complete) => {
                    if build_complete.build_id == build.id {
                        tx.send("OK").ok();

                        println!();

                        log::info!("Build complete");

                        return Ok(());
                    }
                }

                BuildEvents::PushFailure(build_failure) => {
                    if build_failure.build_id == build.id {
                        leap.close().await;

                        println!();

                        bail!(
                                "Push failed, for help contact us on {} and mention the deployment id: {} and build id: {}",
                                urlify("https://discord.gg/hop"),
                                deployment_id,
                                build.id
                            );
                    }
                }
            }
        }
    }

    tx.send("OK").ok();

    Ok(())
}
