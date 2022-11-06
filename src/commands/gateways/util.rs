use std::io::Write;

use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;

use super::create::GatewayOptions;
use super::types::{
    Gateway, GatewayConfig, GatewayProtocol, GatewayType, MultipleGateways, SingleGateway,
};
use crate::state::http::HttpClient;
use crate::utils::ask_question_iter;

pub async fn create_gateway(
    http: &HttpClient,
    deployment_id: &str,
    gateway_config: &GatewayConfig,
) -> Result<Gateway> {
    let response = http
        .request::<SingleGateway>(
            "POST",
            &format!("/ignite/deployments/{deployment_id}/gateways"),
            Some((
                serde_json::to_vec(&gateway_config).unwrap().into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.gateway)
}

pub async fn get_all_gateways(http: &HttpClient, deployment_id: &str) -> Result<Vec<Gateway>> {
    let response = http
        .request::<MultipleGateways>(
            "GET",
            &format!(
                "/ignite/deployments/{deployment_id}/gateways",
                deployment_id = deployment_id
            ),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.gateways)
}

pub async fn get_gateway(http: &HttpClient, gateway_id: &str) -> Result<Gateway> {
    let response = http
        .request::<SingleGateway>("GET", &format!("/ignite/gateways/{gateway_id}"), None)
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.gateway)
}

pub async fn update_gateway(
    http: &HttpClient,
    gateway_id: &str,
    gateway_config: &GatewayConfig,
) -> Result<Gateway> {
    let response = http
        .request::<SingleGateway>(
            "PATCH",
            &format!("/ignite/gateways/{gateway_id}", gateway_id = gateway_id),
            Some((
                serde_json::to_vec(&gateway_config).unwrap().into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.gateway)
}

pub async fn delete_gateway(http: &HttpClient, gateway_id: &str) -> Result<()> {
    http.request::<Value>("DELETE", &format!("/ignite/gateways/{gateway_id}"), None)
        .await?;

    Ok(())
}

pub fn update_gateway_config(
    options: &GatewayOptions,
    is_not_guided: bool,
    gateway_config: &GatewayConfig,
) -> Result<GatewayConfig> {
    let mut gateway_config = gateway_config.clone();

    if is_not_guided {
        update_config_from_args(options, &mut gateway_config)?;
    } else {
        update_config_from_guided(&mut gateway_config)?;
    }

    Ok(gateway_config)
}

fn update_config_from_args(
    options: &GatewayOptions,
    gateway_config: &mut GatewayConfig,
) -> Result<()> {
    let is_update = gateway_config != &GatewayConfig::default();

    let gateway_type = if !is_update {
        let value = options.type_.clone().ok_or_else(|| {
            anyhow!("The argument '--type <TYPE>' requires a value but none was supplied")
        })?;
        gateway_config.type_ = Some(value.clone());
        value
    } else {
        let value = gateway_config.type_.clone().unwrap();
        gateway_config.type_ = None;
        value
    };

    gateway_config.name = options.name.clone();

    match gateway_type {
        GatewayType::Internal => {
            gateway_config.protocol = None;
            gateway_config.target_port = None;

            gateway_config.internal_domain = Some(
                options
                    .internal_domain.clone()
                    .or_else(|| {
                        if is_update {
                            gateway_config.internal_domain.clone()
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("The argument '--internal-domain <INTERNAL_DOMAIN>' requires a value but none was supplied"))?,
            );
        }

        GatewayType::External => {
            gateway_config.internal_domain = None;

            gateway_config.protocol = Some(
                options
                    .protocol
                    .clone()
                    .or_else(|| {
                        if is_update {
                            gateway_config.protocol.clone()
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        anyhow!(
                    "The argument '--protocol <PROTOCOL>' requires a value but none was supplied"
                )
                    })?,
            );

            gateway_config.target_port = Some(options.target_port.or({
                    if is_update {
                        gateway_config.target_port
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("The argument '--target-port <TARGET_PORT>' requires a value but none was supplied"))?,
            );
        }
    };

    Ok(())
}

fn update_config_from_guided(gateway_config: &mut GatewayConfig) -> Result<()> {
    let is_update = gateway_config != &GatewayConfig::default();

    log::debug!("is_update: {is_update}");

    let name = gateway_config.name.clone().unwrap_or_default();

    gateway_config.name = Some(
        dialoguer::Input::<String>::new()
            .with_prompt("Gateway name")
            .show_default(is_update && !name.is_empty())
            .default(name)
            .interact()?,
    );

    if gateway_config.name == Some(String::new()) {
        gateway_config.name = None;
    }

    let gateway_type = if !is_update {
        let value = ask_question_iter("Gateway type", &GatewayType::values(), None)?;

        gateway_config.type_ = Some(value.clone());

        value
    } else {
        let value = gateway_config.type_.clone().unwrap();

        gateway_config.type_ = None;

        value
    };

    match gateway_type {
        GatewayType::Internal => {
            gateway_config.protocol = None;
            gateway_config.target_port = None;

            let internal_domain = gateway_config.internal_domain.clone().unwrap_or_default();

            let internal_domain_regex = Regex::new(r"(?i)^[a-z0-9-.]+.hop$").unwrap();

            gateway_config.internal_domain = Some(
                dialoguer::Input::<String>::new()
                    .with_prompt("Internal domain")
                    .show_default(is_update && !internal_domain.is_empty())
                    .default(internal_domain)
                    .validate_with(|domain: &String| {
                        if domain.is_empty() {
                            Err(anyhow!("The internal domain cannot be empty"))
                        } else if !domain.ends_with(".hop") {
                            Err(anyhow!("The internal domain must end with '.hop'"))
                        } else if !internal_domain_regex.is_match(domain) {
                            Err(anyhow!("The internal domain must be a valid hostname"))
                        } else if domain.len() > 32 {
                            Err(anyhow!(
                                "The internal domain must be less than 32 characters long"
                            ))
                        } else {
                            Ok(())
                        }
                    })
                    .interact()?,
            );

            // because the api adds the .hop suffix to the domain, remove it
            // the suffix requirement is meant to force the user to realize that
            // it will have the suffix later on in container
            gateway_config.internal_domain = gateway_config
                .internal_domain
                .as_ref()
                .map(|domain| domain.strip_suffix(".hop").unwrap().to_string());
        }

        GatewayType::External => {
            gateway_config.internal_domain = None;

            gateway_config.protocol = Some(ask_question_iter(
                "Protocol",
                &GatewayProtocol::values(),
                gateway_config.protocol.clone(),
            )?);

            gateway_config.target_port = Some(
                dialoguer::Input::<u16>::new()
                    .with_prompt("Target port")
                    .default(gateway_config.target_port.unwrap_or(0))
                    .show_default(is_update)
                    .interact()?,
            );
        }
    };

    Ok(())
}

pub fn format_gateways(gateways: &[Gateway], title: bool) -> Vec<String> {
    let mut tw = tabwriter::TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "ID\tNAME\tTYPE\tPROTOCOL\tTARGET_PORT\tDOMAIN").unwrap();
    }

    for gateway in gateways {
        writeln!(
            &mut tw,
            "{}\t{}\t{}\t{}\t{}\t{}",
            gateway.id,
            gateway.name.clone().unwrap_or_else(|| "-".to_string()),
            gateway.type_,
            gateway
                .protocol
                .clone()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            gateway
                .target_port
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            match gateway.type_ {
                GatewayType::Internal => gateway
                    .internal_domain
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                GatewayType::External => gateway
                    .hopsh_domain
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            },
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
