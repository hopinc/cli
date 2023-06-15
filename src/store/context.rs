use std::path::PathBuf;

use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::utils::home_path;
use super::Storable;
use crate::commands::auth::types::AuthorizedClient;
use crate::commands::projects::types::Project;
use crate::config::EXEC_NAME;
use crate::impl_store;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Context {
    /// stored in the context store file
    pub default_project: Option<String>,
    /// stored in the context store file
    pub default_user: Option<String>,
    /// api url override, only save if its not null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_api_url: Option<String>,
    // latest version of the cli and time it was last checked
    pub last_version_check: Option<(String, String)>,

    /// runtime context
    #[serde(skip)]
    pub current: Option<AuthorizedClient>,
    /// runtime context
    #[serde(skip)]
    pub project_override: Option<String>,
}

impl Storable for Context {
    fn path() -> Result<PathBuf> {
        home_path(".hop/context.json")
    }
}

impl_store!(Context);

impl Context {
    pub fn find_project_by_id_or_namespace(&self, id_or_namespace: &str) -> Option<Project> {
        self.current
            .as_ref()
            .and_then(|me| {
                me.projects.iter().find(|p| {
                    p.id == id_or_namespace
                        || p.namespace.to_lowercase() == id_or_namespace.to_lowercase()
                })
            })
            .cloned()
    }

    pub fn current_project(&self) -> Option<Project> {
        match self.project_override.clone() {
            Some(project) => self.find_project_by_id_or_namespace(&project),

            None => self
                .default_project
                .clone()
                .and_then(|id| self.find_project_by_id_or_namespace(&id)),
        }
    }

    #[inline]
    pub fn current_project_error(&self) -> Result<Project> {
        self.current_project().with_context(|| anyhow!("No project specified, run `{EXEC_NAME} projects switch` or use --project to specify a project"))
    }

    // for future use with external package managers
    #[cfg(feature = "update")]
    pub fn update_command(&self) -> String {
        format!("{EXEC_NAME} update")
    }
}
