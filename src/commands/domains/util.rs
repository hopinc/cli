use std::io::Write;

use anyhow::Result;
use serde_json::Value;
use tabwriter::TabWriter;

use crate::{commands::gateways::types::SingleGateway, state::http::HttpClient};

use super::types::{AttachDomain, Domain};

pub async fn attach_domain(http: &HttpClient, gateway_id: &str, domain: &str) -> Result<()> {
    http.request::<Value>(
        "POST",
        &format!("/ignite/gateways/{gateway_id}/domains"),
        Some((
            serde_json::to_vec(&AttachDomain { domain }).unwrap().into(),
            "application/json",
        )),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?;

    Ok(())
}

pub async fn get_all_domains(http: &HttpClient, gateway_id: &str) -> Result<Vec<Domain>> {
    let response = http
        .request::<SingleGateway>("GET", &format!("/ignite/gateways/{gateway_id}"), None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?;

    Ok(response.gateway.domains)
}

pub async fn delete_domain(http: &HttpClient, domain_id: &str) -> Result<()> {
    http.request::<Value>("DELETE", &format!("/ignite/domains/{domain_id}"), None)
        .await?;

    Ok(())
}

pub fn format_domains(domains: &[Domain], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(tw, "ID\tDOMAIN\tSTATE\tCREATED AT").unwrap();
    }

    for domain in domains {
        writeln!(
            tw,
            "{}\t{}\t{}\t{}",
            domain.id, domain.domain, domain.state, domain.created_at
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
