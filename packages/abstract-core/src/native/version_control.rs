//! # Version Control
//!
//! `abstract_core::version_control` stores chain-specific code-ids, addresses and an account_id map.
//!
//! ## Description
//! Code-ids and api-contract addresses are stored on this address. This data can not be changed and allows for complex factory logic.
//! Both code-ids and addresses are stored on a per-module version basis which allows users to easily upgrade their modules.
//!
//! An internal account-id store provides external verification for manager and proxy addresses.  

pub type ModuleMapEntry = (ModuleInfo, ModuleReference);

pub mod state {
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    use crate::objects::{
        common_namespace::ADMIN_NAMESPACE, core::AccountId, module::ModuleInfo,
        module_reference::ModuleReference,
    };

    use super::AccountBase;

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const FACTORY: Admin = Admin::new("factory");

    // We can iterate over the map giving just the prefix to get all the versions
    pub const MODULE_LIBRARY: Map<&ModuleInfo, ModuleReference> = Map::new("module_lib");
    /// Maps Account ID to the address of its core contracts
    pub const ACCOUNT_ADDRESSES: Map<AccountId, AccountBase> = Map::new("account");
}

use crate::objects::{
    core::AccountId,
    module::{Module, ModuleInfo},
    module_reference::ModuleReference,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

/// Contains the minimal Abstract Account contract addresses.
#[cosmwasm_schema::cw_serde]
pub struct AccountBase {
    pub manager: Addr,
    pub proxy: Addr,
}

/// Version Control Instantiate Msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

/// Version Control Execute Msg
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Remove some version of a module
    RemoveModule { module: ModuleInfo },
    /// Add new modules
    AddModules { modules: Vec<ModuleMapEntry> },
    /// Register a new Account to the deployed Accounts.  
    /// Only Factory can call this
    AddAccount {
        account_id: AccountId,
        account_base: AccountBase,
    },
    /// Sets a new Factory
    SetFactory { new_factory: String },
}

/// A ModuleFilter that mirrors the [`ModuleInfo`] struct.
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct ModuleFilter {
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

/// Version Control Query Msg
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Query Core of an Account
    /// Returns [`AccountBaseResponse`]
    #[returns(AccountBaseResponse)]
    AccountBase { account_id: AccountId },
    /// Queries api addresses
    /// Returns [`ModulesResponse`]
    #[returns(ModulesResponse)]
    Modules { infos: Vec<ModuleInfo> },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`ModulesListResponse`]
    #[returns(ModulesListResponse)]
    ModuleList {
        filter: Option<ModuleFilter>,
        start_after: Option<ModuleInfo>,
        limit: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct AccountBaseResponse {
    pub account_base: AccountBase,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesResponse {
    pub modules: Vec<Module>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModulesListResponse {
    pub modules: Vec<Module>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: Addr,
    pub factory: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
