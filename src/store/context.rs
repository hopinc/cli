use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::utils::home_path;
use crate::commands::auth::types::AuthorizedClient;
use crate::commands::projects::types::Project;
use crate::config::EXEC_NAME;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Context {
    /// stored in the context store file
    pub default_project: Option<String>,
    /// stored in the context store file
    pub default_user: Option<String>,
    /// api url override
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

impl Context {
    fn path() -> PathBuf {
        home_path(".hop/context.json")
    }

    pub fn find_project_by_id_or_namespace(&self, id_or_namespace: String) -> Option<Project> {
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

    pub fn find_project_by_id_or_namespace_error(&self, id_or_namespace: String) -> Project {
        self.find_project_by_id_or_namespace(id_or_namespace)
            .expect("Project not found, please check your spelling or switch accounts")
    }

    pub fn current_project(&self) -> Option<Project> {
        match self.project_override.clone() {
            Some(project) => Some(
                self.find_project_by_id_or_namespace(project.clone())
                    .unwrap_or_else(|| panic!("Could not find project `{}`", project)),
            ),

            None => self
                .default_project
                .clone()
                .and_then(|id| self.find_project_by_id_or_namespace(id)),
        }
    }

    pub fn current_project_error(self) -> Project {
        self.current_project().unwrap_or_else(|| panic!("No project specified, run `{} projects switch` or use --project to specify a project",
            EXEC_NAME))
    }

    pub async fn new() -> Self {
        let path = Self::path();

        match fs::metadata(path.clone()).await {
            Ok(_) => match File::open(path).await {
                Ok(mut file) => {
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)
                        .await
                        .expect("Failed to read auth store");

                    serde_json::from_str(&buffer).unwrap()
                }

                Err(err) => {
                    panic!("Error opening auth file: {}", err)
                }
            },
            Err(_) => Self::default().save().await.unwrap(),
        }
    }

    pub async fn save(&mut self) -> Result<Self> {
        if let Some(ref authorized) = self.current {
            self.default_user = Some(authorized.id.clone());
        }

        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create auth store directory");

        let mut file = File::create(path.clone())
            .await
            .expect("Error opening auth file:");

        file.write_all(
            serde_json::to_string(&self)
                .expect("Failed to deserialize auth")
                .as_bytes(),
        )
        .await
        .expect("Failed to write auth store");

        log::debug!("Saved context to {}", path.display());

        Ok(self.clone())
    }

    // for future use with external package managers
    pub fn update_command(&self) -> String {
        format!("{EXEC_NAME} update")
    }
}
