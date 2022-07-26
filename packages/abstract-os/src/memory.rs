//! # Memory
//!
//! `abstract_os::memory` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the memory contract and are retrievable trough smart or raw queries.

use cw_asset::{AssetInfo, AssetInfoUnchecked};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Memory state details
pub mod state {
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    /// Post-fix for asset trading pair addresses
    pub const PAIR_POSTFIX: &str = "pair";

    /// Admin address store
    pub const ADMIN: Admin = Admin::new("admin");
    /// stores name and address of tokens and pairs
    /// LP tokens are stored alphabetically
    pub const ASSET_ADDRESSES: Map<&str, AssetInfo> = Map::new("assets");

    /// Stores contract addresses
    /// Pairs are stored here like LP tokens but with a post-fix
    pub const CONTRACT_ADDRESSES: Map<&str, Addr> = Map::new("contracts");
}

/// Memory Instantiate msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

/// Memory Execute msg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the contract addressbook
    UpdateContractAddresses {
        /// Contracts to update or add
        to_add: Vec<(String, String)>,
        /// Contracts to remove
        to_remove: Vec<String>,
    },
    /// Updates the Asset addressbook
    UpdateAssetAddresses {
        /// Assets to update or add
        to_add: Vec<(String, AssetInfoUnchecked)>,
        /// Assets to remove
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

/// Memory smart-query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries assets based on name
    /// returns [`QueryAssetsResponse`]
    Assets {
        /// Names of assets to query
        names: Vec<String>,
    },
    /// Queries contracts based on name
    /// returns [`QueryContractsResponse`]
    Contracts {
        /// Names of contracts to query
        names: Vec<String>,
    },
    /// Page over contracts
    /// returns [`QueryContractListResponse`]
    ContractList {
        last_contract_name: Option<String>,
        iter_limit: Option<u8>,
    },
    /// Page over assets
    /// returns [`QueryAssetListResponse`]
    AssetList {
        last_asset_name: Option<String>,
        iter_limit: Option<u8>,
    },
}
/// Query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryAssetsResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(String, AssetInfo)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryContractsResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(String, String)>,
}

/// Query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryAssetListResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(String, AssetInfo)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryContractListResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(String, String)>,
}
