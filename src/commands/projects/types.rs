use serde::{Deserialize, Serialize};

// types for the API response
#[derive(Debug, Deserialize)]
pub struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SingleProjectResponse {
    pub project: Project,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub icon: Option<String>,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct ThisProjectResponse {
    pub leap_token: String,
    pub project: Project,
}

#[derive(Debug, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub namespace: String,
    pub payment_method_id: String,
}
