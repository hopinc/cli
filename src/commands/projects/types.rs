use serde::Deserialize;

// types for the API response
#[derive(Debug, Deserialize)]
pub struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateResponse {
    pub project: ProjectRes,
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
