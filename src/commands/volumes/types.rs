use serde::Deserialize;

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
