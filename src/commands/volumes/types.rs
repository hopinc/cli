use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Files {
    Single { file: File },
    Multiple { file: Vec<File> },
}

#[derive(Debug, Deserialize, Clone)]
pub struct File {
    pub name: String,
    pub directory: bool,
    pub permissions: u64,
    pub created_at: String,
    pub updated_at: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct MoveRequest {
    #[serde(rename = "oldPath")]
    pub source: String,
    #[serde(rename = "newPath")]
    pub target: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CreateDirectory {
    pub recursive: bool,
}
