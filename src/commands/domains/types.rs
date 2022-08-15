use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AttachDomain<'a> {
    pub domain: &'a str,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Domain {
    pub id: String,
    pub domain: String,
    pub created_at: String,
    pub state: DomainState,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DomainState {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "ssl_active")]
    SslActive,
    #[serde(rename = "valid_cname")]
    ValidCname,
}

// this is only display for LIST
impl Display for DomainState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap()
                .replace('"', "")
                .replace('_', " ")
        )
    }
}

#[derive(Deserialize)]
pub struct MultipleDomainsResponse {
    pub domains: Vec<Domain>,
}
