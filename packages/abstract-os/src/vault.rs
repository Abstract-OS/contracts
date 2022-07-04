//! # Vault Add-On
//!
//! [`crate::vault`] is an add-on which allows users to deposit into or withdraw from a [`crate::proxy`] contract.
//!
//! ## Description  
//! This contract uses the proxy's value calculation configuration to get the value of the assets held in the proxy and the relative value of the deposit asset.
//! It then mints LP tokens that are claimable for an equal portion of the proxy assets at a later date.  
//!
//! ---
//! **WARNING:** This mint/burn mechanism can be mis-used by flash-loan attacks if the assets contained are of low-liquidity compared to the vault's size.
//!
//! ## Creation
//! The vault contract can be added on an OS by calling [`ExecuteMsg::CreateModule`](crate::manager::ExecuteMsg::CreateModule) on the manager of the os.
//! ```ignore
//! let vault_init_msg = InstantiateMsg{
//!                deposit_asset: "juno".to_string(),
//!                base: AddOnInstantiateMsg{memory_address: "juno1...".to_string()},
//!                fee: Decimal::percent(10),
//!                provider_addr: "juno1...".to_string(),
//!                token_code_id: 3,
//!                vault_lp_token_name: Some("demo_vault".to_string()),
//!                vault_lp_token_symbol: Some("DEMO".to_string()),
//!        };
//! let create_module_msg = ExecuteMsg::CreateModule {
//!                 module: Module {
//!                     info: ModuleInfo {
//!                         name: VAULT.into(),
//!                         version: None,
//!                     },
//!                     kind: crate::core::modules::ModuleKind::External,
//!                 },
//!                 init_msg: Some(to_binary(&vault_init_msg).unwrap()),
//!        };
//! // Call create_module_msg on manager
//! ```
//!
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` on [`crate::manager`] with `crate::registry::VAULT` as module.

use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetUnchecked;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::add_on::{AddOnExecuteMsg, AddOnInstantiateMsg, AddOnQueryMsg};

/// Migrate msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// Init msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Base init msg, sets memory address
    pub base: AddOnInstantiateMsg,
    /// Code-id used to create the LP token
    pub token_code_id: u64,
    /// Fee charged on withdrawal
    pub fee: Decimal,
    /// Address of the service provider which receives the fee.
    pub provider_addr: String,
    /// Asset required to deposit into the vault.
    pub deposit_asset: String,
    /// Name of the vault token
    pub vault_lp_token_name: Option<String>,
    /// Symbol of the vault token
    pub vault_lp_token_symbol: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Execute on the base-add-on contract logic
    Base(AddOnExecuteMsg),
    /// Handler called by the CW-20 contract on a send-call
    /// Acts as the withdraw/provide liquidity function.
    /// Provide the token send message with a [`DepositHookMsg`]
    Receive(Cw20ReceiveMsg),
    /// Provide liquidity to the attached proxy using a native token.
    ProvideLiquidity { asset: AssetUnchecked },
    /// Update the vault pool information
    /// Asset names are resolved using [`abstract_os::memory`].
    UpdatePool {
        deposit_asset: Option<String>,
        assets_to_add: Vec<String>,
        assets_to_remove: Vec<String>,
    },
    /// Set the withdraw fee
    SetFee { fee: Decimal },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(AddOnQueryMsg),
    // Add dapp-specific queries here
    /// Returns [`StateResponse`]
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DepositHookMsg {
    WithdrawLiquidity {},
    ProvideLiquidity {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub liquidity_token: String,
}
