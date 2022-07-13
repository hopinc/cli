use crate::commands::projects::types::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserMe {
    pub user: User,
    pub projects: Vec<Project>,
}
