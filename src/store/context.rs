use std::path::PathBuf;

use super::utils::get_path;
use crate::commands::auth::types::UserMe;
use crate::commands::projects::types::Project;
use crate::config::CONTEXT_STORE_PATH;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    /// stored in the context store file
    pub default_project: Option<String>,
    /// stored in the context store file
    pub default_user: Option<String>,
    /// api url override
    pub override_api_url: Option<String>,

    /// runtime context
    #[serde(skip)]
    pub me: Option<UserMe>,
    /// runtime context
    #[serde(skip)]
    pub project: Option<String>,
}

impl Context {
    fn path() -> PathBuf {
        get_path(CONTEXT_STORE_PATH)
    }

    fn find_project_by_id_or_namespace(self, id_or_namespace: String) -> Option<Project> {
        self.me
            .as_ref()
            .and_then(|me| {
                me.projects.iter().find(|p| {
                    p.id == id_or_namespace
                        || p.namespace.to_lowercase() == id_or_namespace.to_lowercase()
                })
            })
            .cloned()
    }

    pub fn current_project(self) -> Option<Project> {
        match self.project.clone() {
            Some(project) => Some(
                self.find_project_by_id_or_namespace(project.clone())
                    .expect(format!("Could not find project `{}`", project).as_str()),
            ),

            None => self
                .default_project
                .clone()
                .and_then(|id| self.find_project_by_id_or_namespace(id)),
        }
    }

    pub fn current_project_error(self) -> Project {
        self.current_project().expect(
            "No project specified, run `hop projects switch` or use --project to specify a project",
        )
    }

    pub fn default() -> Context {
        Context {
            me: None,
            project: None,
            override_api_url: None,
            default_project: None,
            default_user: None,
        }
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

                    let auth: Self = serde_json::from_str(&buffer).unwrap();
                    auth
                }

                Err(err) => {
                    panic!("Error opening auth file: {}", err)
                }
            },
            Err(_) => Self::default().save().await.unwrap(),
        }
    }

    pub async fn save(mut self) -> Result<Self, std::io::Error> {
        if let Some(ref me) = self.me {
            self.default_user = Some(me.user.id.clone());
        }

        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create auth store directory");

        let mut file = File::create(path.clone())
            .await
            .expect("Error opening auth file:");

        file.write(
            serde_json::to_string(&self)
                .expect("Failed to deserialize auth")
                .as_bytes(),
        )
        .await
        .expect("Failed to write auth store");

        if !self.default_user.is_none() || !self.project.is_none() {
            println!("Saved context to {}", path.display());
        }

        Ok(self)
    }
}
