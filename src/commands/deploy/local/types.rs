use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct DockerAuthStore {
    pub auths: HashMap<String, DockerAuth>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DockerAuth {
    // probably base64 but doesnt matter
    pub auth: String,
}
