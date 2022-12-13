//! # AnsHost
//!
//! `abstract_os::ans_host` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the ans_host contract and are retrievable trough smart or raw queries.

use cosmwasm_schema::QueryResponses;
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use crate::objects::pool_id::{PoolId, UncheckedPoolId};
use crate::objects::{
    asset_entry::AssetEntry,
    contract_entry::{ContractEntry, UncheckedContractEntry},
    pool_info::PoolMetadata,
    pool_type::PoolType,
    ChannelEntry, UncheckedChannelEntry,
};

pub type UniquePoolId = u64;

pub type AssetPair = (String, String);
type DexName = String;
/// The key for an asset pairing
pub type AssetPairingEntry = (String, String, DexName);

/// A map entry of ((asset_x, asset_y, dex) -> compound_pool_id)
pub type AssetPairingMapEntry = (AssetPairingEntry, Vec<PoolReference>);
/// A map entry of (unique_pool_id -> pool_metadata)
pub type PoolMetadataMapEntry = (UniquePoolId, PoolMetadata);


#[cosmwasm_schema::cw_serde]
pub struct PoolReference {
    pub id: UniquePoolId,
    pub pool_id: PoolId,
}


/// AnsHost state details
pub mod state {
    use crate::ans_host::{PoolReference, UniquePoolId, AssetPairingEntry};
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        asset_entry::AssetEntry, common_namespace::ADMIN_NAMESPACE, contract_entry::ContractEntry,
        pool_info::PoolMetadata, ChannelEntry,
    };

    /// Ans host configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub next_unique_pool_id: UniquePoolId,
    }

    pub const CONFIG: Item<Config> = Item::new("config");

    /// Admin address store
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);

    /// Stores name and address of tokens and pairs
    /// LP token pairs are stored alphabetically
    pub const ASSET_ADDRESSES: Map<AssetEntry, AssetInfo> = Map::new("assets");

    /// Stores contract addresses
    pub const CONTRACT_ADDRESSES: Map<ContractEntry, Addr> = Map::new("contracts");

    /// stores channel-ids
    pub const CHANNELS: Map<ChannelEntry, String> = Map::new("channels");

    /// Stores the registered dex names
    pub const REGISTERED_DEXES: Item<Vec<String>> = Item::new("registered_dexes");

    /// Stores the asset pairing entries to their pool ids
    /// (asset1, asset2, dex_name) -> {id: uniqueId, pool_id: poolId}
    pub const ASSET_PAIRINGS: Map<AssetPairingEntry, Vec<PoolReference>> = Map::new("pool_ids");

    /// Stores the metadata for the pools using the unique pool id as the key
    pub const POOL_METADATA: Map<UniquePoolId, PoolMetadata> = Map::new("pools");
}

/// AnsHost Instantiate msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

/// AnsHost Execute msg
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Updates the contract addressbook
    UpdateContractAddresses {
        /// Contracts to update or add
        to_add: Vec<(UncheckedContractEntry, String)>,
        /// Contracts to remove
        to_remove: Vec<UncheckedContractEntry>,
    },
    /// Updates the Asset addressbook
    UpdateAssetAddresses {
        /// Assets to update or add
        to_add: Vec<(String, AssetInfoUnchecked)>,
        /// Assets to remove
        to_remove: Vec<String>,
    },
    /// Updates the Asset addressbook
    UpdateChannels {
        /// Assets to update or add
        to_add: Vec<(UncheckedChannelEntry, String)>,
        /// Assets to remove
        to_remove: Vec<UncheckedChannelEntry>,
    },
    /// Registers a dex
    RegisterDex {
        /// Name of the dex
        name: String,
    },
    /// Update the pools
    UpdatePools {
        /// Pools to update or add
        to_add: Vec<(UncheckedPoolId, PoolMetadata)>,
        /// Pools to remove
        to_remove: Vec<UniquePoolId>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[cosmwasm_schema::cw_serde]
pub struct AssetPairingFilter {
    /// Filter by asset pair
    pub asset_pair: Option<AssetPair>,
    /// Filter by dex
    pub dex: Option<String>,
}

/// Filter on the pool metadatas
#[cosmwasm_schema::cw_serde]
pub struct PoolMetadataFilter {
    /// Filter by pool type
    pub pool_type: Option<PoolType>,
    // /// Filter by pool status
    // pub pool_status: Option<PoolStatus>,
}

/// AnsHost smart-query
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Queries assets based on name
    /// returns [`AssetsResponse`]
    #[returns(AssetsResponse)]
    Assets {
        /// Names of assets to query
        names: Vec<String>,
    },
    /// Page over assets
    /// returns [`AssetListResponse`]
    #[returns(AssetListResponse)]
    AssetList {
        page_token: Option<String>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ContractsResponse`]
    #[returns(ContractsResponse)]
    Contracts {
        /// Project and contract names of contracts to query
        names: Vec<ContractEntry>,
    },
    /// Page over contracts
    /// returns [`ContractListResponse`]
    #[returns(ContractListResponse)]
    ContractList {
        page_token: Option<ContractEntry>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ChannelsResponse`]
    #[returns(ChannelsResponse)]
    Channels {
        /// Project and contract names of contracts to query
        names: Vec<ChannelEntry>,
    },
    /// Page over contracts
    /// returns [`ChannelListResponse`]
    #[returns(ChannelListResponse)]
    ChannelList {
        page_token: Option<ChannelEntry>,
        page_size: Option<u8>,
    },
    /// Retrieve the registered dexes
    /// returns [`RegisteredDexesResponse`]
    #[returns(RegisteredDexesResponse)]
    RegisteredDexes {},
    /// Retrieve the pools with the specified keys
    /// returns [`PoolsResponse`]
    /// TODO: this may need to take a page_token and page_size for the return
    #[returns(PoolsResponse)]
    Pools {
        keys: Vec<AssetPairingEntry>,
    },
    /// Retrieve the (optionally-filtered) list of pools.
    /// returns [`PoolIdListResponse`]
    #[returns(PoolIdListResponse)]
    PoolList {
        filter: Option<AssetPairingFilter>,
        page_token: Option<AssetPairingEntry>,
        page_size: Option<u8>,
    },
    /// Get the pool metadatas for given pool ids
    /// returns [`PoolMetadatasResponse`]
    #[returns(PoolMetadatasResponse)]
    PoolMetadatas {
        keys: Vec<UniquePoolId>,
    },
    /// Retrieve the (optionally-filtered) list of pool metadatas
    /// returns [`PoolMetadataListResponse`]
    #[returns(PoolMetadataListResponse)]
    PoolMetadataList {
        filter: Option<PoolMetadataFilter>,
        page_token: Option<UniquePoolId>,
        page_size: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetsResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetListResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractsResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractListResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelsResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelListResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredDexesResponse {
    pub dexes: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolIdListResponse {
    pub pools: Vec<AssetPairingMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolsResponse {
    pub pools: Vec<AssetPairingMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolMetadatasResponse {
    pub metadatas: Vec<PoolMetadataMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolMetadataListResponse {
    pub metadatas: Vec<PoolMetadataMapEntry>,
}
