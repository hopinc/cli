use anyhow::Result;

use super::types::Balance;
use crate::state::http::HttpClient;

pub async fn get_project_balance(http: &HttpClient, project_id: &str) -> Result<Balance> {
    let balance = http
        .request::<Balance>(
            "GET",
            &format!("/projects/{project_id}/finance/balance"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Error while parsing response"))?;

    Ok(balance)
}
