use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::Value;

use super::utils::get_seconds_from_docker_duration;
use crate::commands::containers::types::ContainerType;
use crate::commands::ignite::health::types::CreateHealthCheck;
use crate::commands::ignite::types::{Config, Deployment, Image, RestartPolicy, Volume};
use crate::commands::ignite::utils::{env_file_to_map, get_shell_array};
use crate::utils::parse_key_val;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerCompose {
    pub name: Option<String>,
    pub version: Option<String>,
    pub secrets: Option<HashMap<String, Secret>>,
    pub services: Option<HashMap<String, Service>>,
    pub volumes: Option<HashMap<String, Value>>,

    // ignored
    pub networks: Option<Value>,
}

impl DockerCompose {
    pub async fn validate_and_update(&mut self, path: &Path) -> Result<()> {
        if self.services.is_none() {
            bail!("No services found in docker-compose.yml");
        }

        if self.networks.is_some() {
            log::warn!("Networks will be ignored when creating deployments from the Compose file");
        }

        let cloned_volumes = self.volumes.clone().unwrap_or_default();
        let mut used_volumes = vec![];

        let services = self.services.clone().unwrap();
        let mut parsed_services = HashMap::new();

        for (name, mut service) in services {
            if service.image.is_none() && service.build.is_none() {
                bail!("Service {name} must have either an image or a build context");
            }

            if let Some(vols) = service.volumes.as_ref() {
                let vol_name = vols.0.clone();

                if used_volumes.contains(&vol_name) {
                    bail!("Volume `{name}` is already used by another service");
                }

                let find = cloned_volumes
                    .keys()
                    .find(|v| v.to_lowercase() == vol_name.to_lowercase());

                if find.is_none() && (!vol_name.starts_with('/') || vol_name != ".") {
                    bail!("Volume `{name}` not found in volumes section");
                }

                used_volumes.push(vol_name);
            }

            if let Some(files) = service.env_file.as_ref() {
                for env_file in files.0.iter() {
                    let env_file_path = path.join(env_file);

                    if !env_file_path.exists() {
                        bail!(
                            "Env file `{}` does not exist but is referenced in service `{}`",
                            env_file_path.display(),
                            name
                        );
                    }

                    let env_file = env_file_to_map(env_file_path.clone()).await?;

                    let mut env = service.environment.unwrap_or_default();
                    env.0.extend(env_file);

                    service.environment = Some(env);
                }
            }

            parsed_services.insert(name, service);
        }

        self.services = Some(parsed_services);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ConfigExternalUnion {
    Bool(bool),
    Named { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Secret {
    pub driver: Option<String>,
    pub external: Option<bool>,
    pub name: Option<String>,

    // Unknown value
    pub labels: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ServiceBuildUnion {
    String(String),
    Map {
        context: String,
        // TODO: support custom dockerfile and args
        dockerfile: Option<String>,
        // args: Option<HashMap<String, Value>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Restart {
    Always,
    UnlessStopped,
    OnFailure,
    #[default]
    Never,
}

impl<'de> Deserialize<'de> for Restart {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "always" => Ok(Restart::Always),
            "unless-stopped" => Ok(Restart::UnlessStopped),
            "on-failure" => Ok(Restart::OnFailure),
            _ => Ok(Restart::Never),
        }
    }
}

impl From<Restart> for RestartPolicy {
    fn from(policy: Restart) -> Self {
        match policy {
            Restart::Always => RestartPolicy::Always,
            Restart::UnlessStopped => RestartPolicy::Always,
            Restart::OnFailure => RestartPolicy::OnFailure,
            Restart::Never => RestartPolicy::Never,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub expose: Option<Vec<Port>>,
    pub ports: Option<Vec<Port>>,
    pub environment: Option<Env>,
    pub env_file: Option<EnvFile>,
    pub restart: Option<Restart>,
    pub image: Option<String>,
    pub build: Option<ServiceBuildUnion>,
    pub depends_on: Option<Vec<String>>,
    pub volumes: Option<DockerVolume>,
    pub entrypoint: Option<DockerShellString>,
    pub command: Option<DockerShellString>,
    pub healthcheck: Option<DockerHealthcheck>,
    // ignored
    pub networks: Option<Value>,
}

impl From<Service> for Deployment {
    fn from(service: Service) -> Self {
        Self {
            config: Config {
                image: Image {
                    name: service.image.unwrap_or_default(),
                },
                restart_policy: service.restart.clone().map(|r| r.into()),
                env: service.environment.unwrap_or_default().0,
                type_: if service.volumes.is_some() {
                    ContainerType::Stateful
                } else {
                    ContainerType::Persistent
                },
                volume: service.volumes.map(|volume| Volume {
                    mount_path: volume.1,
                    ..Default::default()
                }),
                entrypoint: service.entrypoint.map(|ep| ep.0),
                cmd: service.command.map(|cmd| cmd.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, remote = "Self")]
pub struct DockerHealthcheck {
    pub test: HealthCheckTest,
    pub interval: Option<DockerDuration>,
    pub timeout: Option<DockerDuration>,
    pub retries: Option<u32>,
    pub start_period: Option<DockerDuration>,
}

impl<'de> Deserialize<'de> for DockerHealthcheck {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let this = Self::deserialize(deserializer)?;

        if let Some(interval) = this.interval.clone() {
            if interval.0 < 5 {
                return Err(serde::de::Error::custom(
                    "interval must be greater than 5 seconds",
                ));
            } else if interval.0 > 120 {
                return Err(serde::de::Error::custom(
                    "interval must be less than 120 seconds",
                ));
            }
        }

        if let Some(retries) = this.retries {
            if retries < 1 {
                return Err(serde::de::Error::custom("retries must be greater than 1"));
            } else if retries > 10 {
                return Err(serde::de::Error::custom("retries must be less than 10"));
            }
        }

        Ok(Self {
            test: this.test,
            interval: this.interval,
            timeout: this.timeout,
            retries: this.retries,
            start_period: this.start_period,
        })
    }
}

impl From<DockerHealthcheck> for CreateHealthCheck {
    fn from(healthcheck: DockerHealthcheck) -> Self {
        let mut current = Self {
            path: healthcheck.test.0,
            port: healthcheck.test.1,
            ..Default::default()
        };

        // divide by 1000 to convert from ms to s for the API
        if let Some(interval) = healthcheck.interval {
            current.interval = interval.0;
        }

        if let Some(timeout) = healthcheck.timeout {
            current.timeout = timeout.0;
        }

        if let Some(retries) = healthcheck.retries {
            current.max_retries = retries.into();
        }

        if let Some(start_period) = healthcheck.start_period {
            current.initial_delay = start_period.0;
        }

        current
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckTest(String, u16);

impl<'de> Deserialize<'de> for HealthCheckTest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        // regex to extract the hostname port and path from a given curl command
        let re = Regex::new(r"^curl\s?((?:-|--)[A-Za-z]+)*\s+(https?://)?([^/:\s]+)(:\d+)?(/.*)?$")
            .unwrap();

        let test_string = match value {
            Value::String(s) => s,
            Value::Sequence(a) => {
                // change sequence of values to a sequence of strings
                let collected = a
                    .into_iter()
                    .map(|v| match v {
                        Value::String(s) => Ok(s),
                        _ => Err(serde::de::Error::custom("Invalid healthcheck test")),
                    })
                    .collect::<Result<Vec<String>, _>>()?;

                if collected.first() == Some(&"NONE".to_string()) {
                    return Err(serde::de::Error::custom("Invalid healthcheck test"));
                } else if collected.first() == Some(&"CMD".to_string())
                    || collected.first() == Some(&"CMD-SHELL".to_string())
                {
                    collected[1..].join(" ")
                } else {
                    return Err(serde::de::Error::custom("Invalid healthcheck test"));
                }
            }
            _ => return Err(serde::de::Error::custom("Invalid healthcheck test")),
        };

        let captures = re.captures(&test_string).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Invalid healthcheck test: {test_string}, currently only curl is supported"
            ))
        })?;

        let path = captures
            .get(5)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());

        let port = captures
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| ":80".to_string())
            .trim_start_matches(':')
            .parse::<u16>()
            .map_err(|_| {
                serde::de::Error::custom(format!(
                    "Invalid healthcheck test: {test_string}, port must be a number"
                ))
            })?;

        Ok(HealthCheckTest(path, port))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerDuration(u64);

impl<'de> Deserialize<'de> for DockerDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        get_seconds_from_docker_duration(&s)
            .map(DockerDuration)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Env(HashMap<String, String>);

impl<'de> Deserialize<'de> for Env {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Sequence(seq) => {
                let mut map = HashMap::new();

                // log::debug!("Sequence: {:?}", seq);

                for item in seq {
                    let item_str = item
                        .as_str()
                        .context("Failed to parse environment variable from sequence")
                        .map_err(serde::de::Error::custom)?;

                    let (key, value) = parse_key_val::<String, String>(item_str)
                        .map_err(|error| serde::de::Error::custom(error.to_string()))?;

                    map.insert(key, value);
                }

                Ok(Self(map))
            }

            Value::Mapping(mapping) => {
                let mut map = HashMap::new();

                // log::debug!("Mapping: {:?}", mapping);

                for (key, value) in mapping {
                    let key = key
                        .as_str()
                        .context("Failed to parse environment variable")
                        .map_err(serde::de::Error::custom)?;

                    let value = value
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| value.as_f64().map(|f| f.to_string()))
                        .context("INvalid value in mapping, expected string or number")
                        .map_err(serde::de::Error::custom)?;

                    map.insert(key.to_string(), value.to_string());
                }

                Ok(Self(map))
            }

            _ => Err(serde::de::Error::custom(
                "Failed to parse environment variable",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port(pub u16);

impl<'de> Deserialize<'de> for Port {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Number(number) => {
                let port = number
                    .as_u64()
                    .context("Failed to parse port")
                    .map_err(serde::de::Error::custom)?;

                Ok(Self(port as u16))
            }

            Value::String(string) => {
                if string.contains(['/', '-']) {
                    return Err(serde::de::Error::custom(format!(
                        "Failed to parse port, unsupported format {string}",
                    )));
                }

                let ports = string.as_str().split(':');

                let port = ports
                    .last()
                    .context("Failed to parse port")
                    .map_err(serde::de::Error::custom)?
                    .parse::<u16>()
                    .context("Failed to parse port")
                    .map_err(serde::de::Error::custom)?;

                Ok(Self(port))
            }

            Value::Mapping(mut map) => map
                .remove("target")
                .and_then(|target| target.as_u64())
                .map(|target| Self(target as u16))
                .ok_or_else(|| serde::de::Error::custom("Failed to parse port")),

            _ => Err(serde::de::Error::custom("Failed to parse port")),
        }
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DockerVolume(pub String, pub String);

impl<'de> Deserialize<'de> for DockerVolume {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Sequence(seq) => {
                if seq.len() > 1 {
                    return Err(serde::de::Error::invalid_length(
                        seq.len(),
                        &"Expected a sequence of length 1",
                    ));
                }

                let volume = seq.first().and_then(|v| v.as_str()).and_then(|s| {
                    if s.contains(':') {
                        Some(s.split(':').map(|s| s.to_string()).collect::<Vec<String>>())
                    } else {
                        None
                    }
                });

                if let Some(volume) = volume {
                    // valid if we have 2 or 3 elements
                    // since the 3rd element is optional and is the mode
                    if volume.len() == 2 || volume.len() == 3 {
                        return Ok(Self(volume[0].clone(), volume[1].clone()));
                    }
                }

                Err(serde::de::Error::custom("Failed to parse docker volume"))
            }

            unx => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(&format!("{unx:?}")),
                &"Expected a sequence",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerShellString(pub Vec<String>);

impl<'de> Deserialize<'de> for DockerShellString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Sequence(seq) => {
                let mut cmd = Vec::new();

                for item in seq {
                    let item_str = item
                        .as_str()
                        .context("Failed to parse docker command")
                        .map_err(serde::de::Error::custom)?;

                    cmd.push(item_str.to_string());
                }

                Ok(Self(cmd))
            }

            Value::String(string) => Ok(Self(get_shell_array(&string))),

            unx => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(&format!("{unx:?}")),
                &"Expected a sequence",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvFile(pub Vec<String>);

impl<'de> Deserialize<'de> for EnvFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        match value {
            Value::Sequence(seq) => {
                let mut env_files = Vec::new();

                for item in seq {
                    let item_str = item
                        .as_str()
                        .context("Failed to parse env file")
                        .map_err(serde::de::Error::custom)?;

                    env_files.push(item_str.to_string());
                }

                Ok(Self(env_files))
            }

            Value::String(string) => Ok(Self(vec![string])),

            unx => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other(&format!("{unx:?}")),
                &"Expected a sequence",
            )),
        }
    }
}
