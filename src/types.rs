use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Base<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub icon: Option<String>,
    pub namespace: String,
    #[serde(rename = "type")]
    pub p_type: String,
}

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

#[derive(Debug, Deserialize)]
pub struct ErrorContent {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorContent,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Secret {
    pub id: String,
    pub name: String,
    pub digest: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Secrets {
    pub secrets: Vec<Secret>,
}
