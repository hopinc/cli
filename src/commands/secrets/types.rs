use serde::Deserialize;

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

#[derive(Debug, Deserialize, Clone)]
pub struct SecretResponse {
    pub secret: Secret,
}
