use serde::Deserialize;

use crate::commands::projects::types::Project;

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub username: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AuthorizedClient {
    pub id: String,
    pub name: String,
    pub leap_token: String,
    pub projects: Vec<Project>,
    pub email: String,
    pub email_verified: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserMe {
    pub leap_token: String,
    pub user: User,
    pub projects: Vec<Project>,
}
