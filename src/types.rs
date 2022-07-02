use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Base<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub icon: Option<String>,
    pub namespace: String,
    #[serde(rename = "type")]
    pub p_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Projects {
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct UsersMe {
    pub user: User,
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
