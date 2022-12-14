use std::io::Write;

use anyhow::Result;
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{
    CreateHealthCheck, HealthCheck, HealthCheckState, MultipleHealthCheckState,
    MultipleHealthChecks, SingleHealthCheck,
};
use crate::state::http::HttpClient;
use crate::utils::relative_time;

pub fn create_health_check_config(
    config: super::create::HealthCheckCreate,
) -> Result<CreateHealthCheck> {
    let mut health_check = CreateHealthCheck::default();

    if config != Default::default() {
        update_config_from_args(&mut health_check, config)?;
    } else {
        update_config_from_guided(&mut health_check)?;
    }

    Ok(health_check)
}

pub fn update_config_from_args(
    config: &mut CreateHealthCheck,
    args: super::create::HealthCheckCreate,
) -> Result<()> {
    if let Some(port) = args.port {
        config.port = port;
    }

    if let Some(path) = args.path {
        config.path = path;
    }

    if let Some(interval) = args.interval {
        config.interval = interval;
    }

    if let Some(timeout) = args.timeout {
        config.timeout = timeout;
    }

    if let Some(failure_threshold) = args.max_retries {
        config.max_retries = failure_threshold;
    }

    if let Some(initial_delay) = args.initial_delay {
        config.initial_delay = initial_delay;
    }

    Ok(())
}

pub fn update_config_from_guided(config: &mut CreateHealthCheck) -> Result<()> {
    config.port = dialoguer::Input::<u16>::new()
        .with_prompt("Port of the health check")
        .default(config.port)
        .interact()?;

    config.path = dialoguer::Input::<String>::new()
        .with_prompt("Path of the health check")
        .default(config.path.clone())
        .interact()?;

    config.interval = dialoguer::Input::<u64>::new()
        .with_prompt("Interval of the health check")
        .default(config.interval)
        .interact()?;

    config.timeout = dialoguer::Input::<u64>::new()
        .with_prompt("Timeout of the health check")
        .default(config.timeout)
        .interact()?;

    config.max_retries = dialoguer::Input::<u64>::new()
        .with_prompt("Max retries of the health check")
        .default(config.max_retries)
        .interact()?;

    config.initial_delay = dialoguer::Input::<u64>::new()
        .with_prompt("Initial delay of the health check")
        .default(config.initial_delay)
        .interact()?;

    Ok(())
}

pub async fn create_health_check(
    http: &HttpClient,
    deployment_id: &str,
    config: CreateHealthCheck,
) -> Result<HealthCheck> {
    let check = http
        .request::<SingleHealthCheck>(
            "POST",
            &format!("/ignite/deployments/{deployment_id}/health-checks"),
            Some((serde_json::to_vec(&config)?.into(), "application/json")),
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not parse response"))?
        .health_check;

    Ok(check)
}

pub async fn get_all_health_checks(
    http: &HttpClient,
    deployment_id: &str,
) -> Result<Vec<HealthCheck>> {
    let checks = http
        .request::<MultipleHealthChecks>(
            "GET",
            &format!("/ignite/deployments/{deployment_id}/health-checks"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not parse response"))?;

    Ok(checks.health_checks)
}

pub async fn delete_health_check(http: &HttpClient, health_check_id: &str) -> Result<()> {
    http.request::<Value>(
        "DELETE",
        &format!("/ignite/health-checks/{health_check_id}",),
        None,
    )
    .await?;

    Ok(())
}

pub async fn get_health_state(
    http: &HttpClient,
    deployment_id: &str,
) -> Result<Vec<HealthCheckState>> {
    let state = http
        .request::<MultipleHealthCheckState>(
            "GET",
            &format!("/ignite/deployments/{deployment_id}/health-check-state"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not parse response"))?;

    Ok(state.health_check_states)
}

pub fn format_health_checks(checks: &[HealthCheck], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(
            &mut tw,
            "ID\tINTERVAL\tTIMEOUT\tMAX RETRIES\tINITIAL DELAY\tPORT\tPROTOCOL\tPATH\tCREATED AT"
        )
        .unwrap();
    }

    for check in checks {
        writeln!(
            &mut tw,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            check.id,
            check.interval,
            check.timeout,
            check.max_retries,
            check.initial_delay,
            check.port,
            check.protocol,
            check.path,
            check.created_at,
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn format_health_state(state: &[HealthCheckState], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "CONTAINER ID\tSTATE\tNEXT CHECK IN").unwrap();
    }

    for check in state {
        writeln!(
            &mut tw,
            "{}\t{}\t{}",
            check.container_id,
            check.state,
            relative_time(check.next_check),
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
