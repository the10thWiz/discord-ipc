use serde::{Deserialize, Serialize};

use crate::discord::Snowflake;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OauthScope {
    // TODO:
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Application {
    description: String,
    icon: String,
    id: Snowflake,
    rpc_origins: Vec<String>,
    name: String,
}
