use crate::{core::modules::Module, native::version_control::state::Core};
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_rust_script_derive::CosmWasmContract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
pub struct InstantiateMsg {
    /// Version control address used to get code-ids and register OS
    pub version_control_address: String,
    /// Memory address
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        admin: Option<String>,
        memory_address: Option<String>,
        version_control_address: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Context {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub memory_address: String,
    pub version_control_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContextResponse {
    pub core: Option<Core>,
    pub module: Option<Module>,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
