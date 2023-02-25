mod types;
mod utils;

use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};

use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc::unbounded_channel;

use self::utils::{parse_publish, TonneruSocket};
use super::ignite::utils::get_all_deployments;
use crate::commands::ignite::types::Deployment;
use crate::commands::ignite::utils::{format_deployments, get_deployment};
use crate::commands::tunnel::types::Prefix;
use crate::commands::tunnel::utils::{
    add_entry_to_hosts, get_id_with_prefix, remove_entry_from_hosts,
};
use crate::state::State;
use crate::utils::urlify;

// TLS Socker Uri
const TONNERU_URI: &str = "tonneru.hop.io";
const TONNERU_PORT: u16 = 443;
const DOMAIN_SUFFIX: &str = "hop";

#[derive(Debug, Parser)]
#[clap(about = "Access your application via a tunnel")]
pub struct Options {
    #[clap(help = "Resource to tunnel to, can be a deployment name, ID")]
    pub deployment: Option<String>,
    #[clap(long, help = "Publish a container's port(s) to the host", value_parser = parse_publish)]
    pub publish: Option<(IpAddr, u16, u16)>,
    #[clap(long, help = "Add an entry to your hosts file with the tunnel domain")]
    pub hosts: bool,
}

pub async fn handle(options: &Options, state: State) -> Result<()> {
    let project = state.ctx.clone().current_project_error()?;

    let deployment = match get_id_with_prefix(options.deployment.as_deref()) {
        Some((Prefix::Deployment, id)) => get_deployment(&state.http, &id).await?,
        Some((Prefix::Container, id)) => Deployment {
            id: id.clone(),
            name: id,
            container_count: 1,
            ..Default::default()
        },
        unknown => {
            let deployments = get_all_deployments(&state.http, &project.id).await?;
            ensure!(!deployments.is_empty(), "No deployments found.");

            if let Some((_, name)) = unknown {
                deployments
                    .iter()
                    .find(|d| d.name.to_lowercase() == name.to_lowercase())
                    .ok_or_else(|| anyhow!("Deployment not found."))?
                    .clone()
            } else {
                let deployments_fmt = format_deployments(&deployments, false);

                let idx = dialoguer::Select::new()
                    .with_prompt("Select a deployment")
                    .items(&deployments_fmt)
                    .default(0)
                    .interact()?;

                deployments[idx].clone()
            }
        }
    };

    ensure!(
        deployment.container_count > 0,
        "Deployment has no running containers."
    );

    let (ip_address, local_port, remote_port) = if let Some(publish_values) = options.publish {
        publish_values
    } else {
        let local_address = dialoguer::Input::<IpAddr>::new()
            .with_prompt("Local IP address to bind to")
            .default(IpAddr::from([127, 0, 0, 1]))
            .interact()?;

        let mut ports = HashSet::new();

        // metadata is only available for running containers
        deployment
            .metadata
            .unwrap_or_default()
            .container_port_mappings
            .unwrap_or_default()
            .values()
            .for_each(|v| {
                v.iter().for_each(|p| {
                    let port_split = p.split(':').collect::<Vec<_>>();

                    if let Some(port) = port_split.last() {
                        ports.insert(port.to_string());
                    }
                });
            });

        let mut ports = ports.into_iter().collect::<Vec<_>>();
        ports.push("Custom".to_string());

        log::debug!("Ports set: {:?}", ports);

        let local_port = {
            let idx = if ports.len() == 1 {
                0
            } else {
                dialoguer::Select::new()
                    .with_prompt("Select a local port")
                    .items(&ports)
                    .default(0)
                    .interact()?
            };

            if idx == ports.len() - 1 {
                dialoguer::Input::<u16>::new()
                    .with_prompt("Local port number")
                    .interact()?
            } else {
                ports[idx].parse()?
            }
        };

        let remote_port = {
            let idx = if ports.len() == 1 {
                0
            } else {
                dialoguer::Select::new()
                    .with_prompt("Select the remote port")
                    .items(&ports)
                    .default(0)
                    .interact()?
            };

            if idx == ports.len() - 1 {
                dialoguer::Input::<u16>::new()
                    .with_prompt("Remote port number")
                    .interact()?
            } else {
                ports[idx].parse()?
            }
        };

        (local_address, local_port, remote_port)
    };

    let token = state
        .token()
        .ok_or_else(|| anyhow!("No auth token found."))?;

    let listiner = TcpListener::bind(SocketAddr::new(ip_address, local_port))
        .await
        .map_err(|e| anyhow!("Failed to bind to port {local_port}: {e}"))?;

    let domain = if !options.hosts {
        ip_address.to_string()
    } else {
        format!("{}.{DOMAIN_SUFFIX}", deployment.name)
    };

    if options.hosts {
        let ip_to_add = if ip_address.is_unspecified() {
            "127.0.0.1".to_string()
        } else {
            ip_address.to_string()
        };

        // edit /etc/hosts
        add_entry_to_hosts(&domain, &ip_to_add).await?;

        let rm_domain = domain.clone();

        // trap ctrl+c / SIGINT and remove entry from /etc/hosts
        let (tx, mut rx) = unbounded_channel();
        tokio::spawn(async move {
            loop {
                if let Some("CANCEL") = rx.recv().await {
                    remove_entry_from_hosts(&rm_domain).await.unwrap();

                    std::process::exit(0);
                }
            }
        });

        let ctrlc = tx.clone();

        ctrlc::set_handler(move || {
            ctrlc.send("CANCEL").ok();
        })?;
    }

    log::info!(
        "Forwarding to `{}` on {}",
        deployment.name,
        urlify(&format!("{domain}:{local_port}"))
    );

    let tonneru = TonneruSocket::new(&token, &deployment.id, remote_port)?;

    loop {
        let (mut stream, local_socket) = listiner.accept().await?;
        let tonneru = tonneru.clone();

        log::info!("New connection from {local_socket}");

        tokio::spawn(async move {
            let mut socket = match tonneru.connect().await {
                Ok(socket) => socket,
                Err(e) => {
                    log::error!("Failed to connect to tonneru: {e}");

                    // dont care if this fails
                    stream.shutdown().await.ok();

                    return;
                }
            };

            match tokio::io::copy_bidirectional(&mut stream, &mut socket).await {
                Ok(_) => log::info!("Connection closed for {local_socket}"),
                Err(e) => {
                    log::error!("Connection error: {e}, closing connection for {local_socket}")
                }
            }

            // close all sockets
            socket.shutdown().await.ok();
            stream.shutdown().await.ok();
        });
    }
}
