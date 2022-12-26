use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub balance: String,
    #[serde(rename = "outstanding_balance")]
    pub outstanding: String,
    pub next_billing_cycle: String,
}
