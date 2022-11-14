use std::path::PathBuf;
use std::process::Command as Cmd;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use runas::Command as Sudo;
use tokio::fs;
use tokio::{io::AsyncWriteExt, net::TcpStream};
#[cfg(not(windows))]
use tokio_rustls::client::TlsStream;

use crate::utils::is_writable;

use super::types::TonneruPacket;
use super::{TONNERU_PORT, TONNERU_URI};

#[derive(Debug, Clone)]
pub struct TonneruSocket {
    token: String,
    deployment_id: String,
    port: u16,
}

impl TonneruSocket {
    pub fn new(token: &str, deployment_id: &str, port: u16) -> Self {
        Self {
            token: token.to_string(),
            deployment_id: deployment_id.to_string(),
            port,
        }
    }

    #[cfg(not(windows))]
    pub async fn connect(&self) -> Result<TlsStream<TcpStream>> {
        use tokio::io::AsyncReadExt;

        use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
        use tokio_rustls::TlsConnector;

        let remote = TcpStream::connect(format!("{TONNERU_URI}:{TONNERU_PORT}")).await?;

        log::debug!("Connected to {TONNERU_URI}:{TONNERU_PORT}");

        // ref: https://github.com/rustls/hyper-rustls/blob/main/src/config.rs#L55
        let mut roots = RootCertStore::empty();
        roots.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let arc = Arc::new(config);
        let dns_name = ServerName::try_from(TONNERU_URI)?;

        log::debug!("Connecting to {TONNERU_URI} with TLS");

        let mut socket = TlsConnector::from(arc)
            .connect(dns_name, remote)
            .await
            .map_err(|e| anyhow!("Failed to connect to {}: {}", TONNERU_URI, e))?;

        let packet = serde_json::to_vec(&TonneruPacket::Auth {
            token: self.token.clone(),
            resource_id: self.deployment_id.clone(),
            port: self.port,
        })?;

        log::debug!("Sending auth packet: {}", String::from_utf8_lossy(&packet));

        socket.write_all(&packet).await?;

        let mut buf = [0; 1024];

        match socket.read(&mut buf).await {
            Ok(n) => match serde_json::from_slice::<TonneruPacket>(&buf[..n]) {
                Ok(TonneruPacket::Connect) => Ok(socket),
                _ => Err(anyhow!(
                    "Unexpected packet. Received: {}",
                    String::from_utf8_lossy(&buf[..n])
                )),
            },
            Err(e) => Err(anyhow!("Failed to read from socket: {}", e)),
        }
    }
}

pub fn parse_ports(ports: &str) -> Result<(u16, u16)> {
    let mut ports = ports.split(':').map(|p| p.parse::<u16>());

    if ports.clone().count() > 2 {
        return Err(anyhow!("Invalid port format."));
    }

    let local = ports
        .next()
        .ok_or_else(|| anyhow!("Invalid port format."))??;
    let remote = ports.next().unwrap_or(Ok(local))?;

    Ok((local, remote))
}

pub async fn add_entry_to_hosts(domain: &str, address: &str) -> Result<()> {
    log::debug!("Adding entry to hosts: {domain} -> {address}");

    let path = PathBuf::from("/etc/hosts");

    let mut hosts = fs::read_to_string(&path)
        .await?
        .trim_matches('\n')
        .to_string();

    hosts.push_str(&format!("\n{address}\t{domain}\t# Added by Hop CLI"));

    let ok = if is_writable(&path).await {
        Cmd::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | tee {} > /dev/null",
                hosts,
                path.to_str().unwrap()
            ))
            .status()?
            .success()
    } else {
        log::warn!("Adding entry to hosts requires sudo permissions.");
        Sudo::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | tee {} > /dev/null",
                hosts,
                path.to_str().unwrap()
            ))
            .status()?
            .success()
    };

    if !ok {
        return Err(anyhow!("Failed to add entry to hosts file."));
    }

    Ok(())
}

pub async fn remove_entry_from_hosts(domain: &str) -> Result<()> {
    let path = PathBuf::from("/etc/hosts");

    let hosts = fs::read_to_string(&path).await?;

    let hosts = hosts
        .lines()
        .filter(|l| !l.contains(domain))
        .collect::<Vec<_>>()
        .join("\n");

    let ok = if is_writable(&path).await {
        Cmd::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | tee {} > /dev/null",
                hosts,
                path.to_str().unwrap()
            ))
            .status()?
            .success()
    } else {
        log::warn!("Removing entry from hosts requires sudo permissions.");
        Sudo::new("sh")
            .arg("-c")
            .arg(format!(
                "echo '{}' | tee {} > /dev/null",
                hosts,
                path.to_str().unwrap()
            ))
            .status()?
            .success()
    };

    if !ok {
        return Err(anyhow!("Failed to remove entry from hosts file."));
    }

    Ok(())
}
