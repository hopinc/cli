use serde_json::Value;

use crate::manager::types::ShardRunnerUpdate;

#[derive(Debug)]
pub enum ShardMessengerMessage {
    Json(Value),
    Update(ShardRunnerUpdate),
}
