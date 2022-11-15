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
use crate::commands::ignite::utils::format_deployments;
use crate::commands::tunnel::utils::{add_entry_to_hosts, remove_entry_from_hosts};
use crate::state::State;

// TLS Socker Uri
const TONNERU_URI: &str = "tonneru.hop.io";
const TONNERU_PORT: u16 = 443;
const DOMAIN_SUFFIX: &str = "hop";

#[derive(Debug, Parser)]
#[clap(about = "Access your application via a tunnel")]
pub struct Options {
    #[clap(help = "ID or name of the deployment")]
    pub deployment: Option<String>,
    #[clap(long, help = "Publish a container's port(s) to the host", value_parser = parse_publish)]
    pub publish: Option<(String, u16, u16)>,
    #[clap(long, help = "Do not add an entry to your hosts file")]
    pub no_hosts: bool,
}

pub async fn handle(options: &Options, state: State) -> Result<()> {
    let project = state.ctx.clone().current_project_error();

    let deployments = get_all_deployments(&state.http, &project.id).await?;
    ensure!(!deployments.is_empty(), "No deployments found.");

    let deployment = if let Some(ref id) = options.deployment {
        deployments
            .into_iter()
            .find(|d| d.id == *id || d.name == *id)
            .ok_or_else(|| anyhow!("Deployment not found."))?
    } else {
        let deployments_fmt = format_deployments(&deployments, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a deployment")
            .items(&deployments_fmt)
            .default(0)
            .interact_opt()?
            .ok_or_else(|| anyhow!("No deployment selected."))?;

        deployments[idx].clone()
    };

    ensure!(
        deployment.container_count > 0,
        "Deployment has no running containers."
    );

    let (ip_address, local_port, remote_port) = if let Some(ref publish_values) = options.publish {
        publish_values.clone()
    } else {
        let local_address = dialoguer::Input::<String>::new()
            .with_prompt("Local IP address to bind to")
            .default("127.0.0.1".to_string())
            .interact()?;

        let mut ports = HashSet::new();

        // metadata is only available for running containers
        deployment
            .metadata
            .unwrap()
            .container_port_mappings
            .values()
            .for_each(|v| {
                v.iter().for_each(|p| {
                    let port_split = p.split(':').collect::<Vec<_>>();
                    ports.insert(port_split.last().unwrap().to_string());
                });
            });

        let mut ports = ports.into_iter().collect::<Vec<_>>();
        ports.push("Custom".to_string());

        log::debug!("Ports set: {:?}", ports);

        let local_port = {
            let idx = dialoguer::Select::new()
                .with_prompt("Select a local port")
                .items(&ports)
                .default(0)
                .interact_opt()?
                .ok_or_else(|| anyhow!("No port selected."))?;

            if idx == ports.len() - 1 {
                dialoguer::Input::<u16>::new()
                    .with_prompt("Local port number")
                    .interact()?
            } else {
                ports[idx].parse()?
            }
        };

        let remote_port = {
            let idx = dialoguer::Select::new()
                .with_prompt("Select the remote port")
                .items(&ports)
                .default(0)
                .interact_opt()?
                .ok_or_else(|| anyhow!("No port selected."))?;

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

    let listiner = TcpListener::bind(SocketAddr::new(IpAddr::V4(ip_address.parse()?), local_port))
        .await
        .map_err(|e| anyhow!("Failed to bind to port {local_port}: {e}"))?;

    let domain = if options.no_hosts {
        ip_address.clone()
    } else {
        format!("{}.{DOMAIN_SUFFIX}", deployment.name)
    };

    if !options.no_hosts {
        let ip_to_add = match ip_address.as_str() {
            // if users wants to listen on all interfaces lets use loopback for the domain
            "0.0.0.0" => "127.0.0.1",
            ip => ip,
        };

        // edit /etc/hosts
        add_entry_to_hosts(&domain, ip_to_add).await?;

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

    log::info!("Tonneru listening on port tcp://{domain}:{local_port}");

    let tonneru = TonneruSocket::new(&token, &deployment.id, remote_port)?;

    loop {
        let (mut stream, local_socket) = listiner.accept().await?;

        log::info!("New connection from {local_socket}");

        let tonneru = tonneru.clone();

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
