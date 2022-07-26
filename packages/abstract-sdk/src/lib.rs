//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.

pub mod _modules;
pub mod common_namespace;
pub mod manager;
mod module_traits;
pub mod proxy;
pub mod tendermint_staking;
pub mod version_control;
pub mod memory {
    pub use abstract_os::objects::memory::{
        query_asset_from_mem, query_assets_from_mem, query_contract_from_mem,
        query_contracts_from_mem, Memory,
    };
}

pub use module_traits::{LoadMemory, OsExecute};

pub extern crate abstract_os;
