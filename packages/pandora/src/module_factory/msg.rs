use crate::modules::Module;
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register OS
    pub version_control_contract: String,
    /// Memory contract
    pub memory_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        admin: Option<String>,
        memory_contract: Option<String>,
        version_control_contract: Option<String>,
    },
    /// Creates the core contracts for the OS
    CreateModule {
        /// Module details
        module: Module,
        init_msg: Option<Binary>,
    },
    UpdateFactoryBinaryMsgs {
        to_add: Vec<((String, String), Binary)>,
        to_remove: Vec<(String, String)>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub memory_contract: String,
    pub version_control_contract: String,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
