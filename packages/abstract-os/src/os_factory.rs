//! # Memory
//!
//! `abstract_os::memory` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the proxy contract and are retrievable trough smart or raw queries.
//! This is useful when managing a large set of contracts. 


use crate::objects::gov_type::GovernanceDetails;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Msg used on instantiation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register OS
    pub version_control_address: String,
    /// Memory contract
    pub memory_address: String,
    /// Address of module factory. Used for instantiating manager.
    pub module_factory_address: String,
}

/// Execute function entrypoint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Handler called by the CW-20 contract on a send-call
    Receive(Cw20ReceiveMsg),
    /// Update config
    UpdateConfig {
        /// New admin
        admin: Option<String>,
        /// New memory contract
        memory_contract: Option<String>,
        /// New version control contract
        version_control_contract: Option<String>,
        /// New module factory contract
        module_factory_address: Option<String>,
        /// New subscription contract
        subscription_address: Option<String>,
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    CreateOs {
        /// Governance details
        /// Use [`crate::objects::GovernanceDetails::Monarchy`] to use a custom governance modal.
        /// TODO: add support for other types of gov.
        governance: GovernanceDetails,
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
    pub module_factory_address: String,
    pub subscription_address: Option<String>,
    pub next_os_id: u32,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
