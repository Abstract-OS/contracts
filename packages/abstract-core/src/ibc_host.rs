use crate::{
    ibc_client::CallbackInfo,
    manager,
    objects::{account::AccountId, chain_name::ChainName},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg, Empty, QueryRequest};

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Used to easily perform address translation on the app chain
    pub ans_host_address: String,
    /// Used to create remote abstract accounts
    pub account_factory_address: String,
    /// Version control address
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum InternalAction {
    /// Registers a new account on the remote chain
    Register {
        account_proxy_address: String,
        name: String,
        description: Option<String>,
        link: Option<String>,
    },
    WhoAmI {
        // Client chain to assign to the channel
        client_chain: ChainName,
    },
}

/// Callable actions on a remote host
#[cosmwasm_schema::cw_serde]
pub enum HostAction {
    /// Dispatch a message to the Account's manager
    Dispatch {
        manager_msg: manager::ExecuteMsg,
    },
    Query {
        msgs: Vec<QueryRequest<Empty>>,
    },
    SendAllBack {},
    Balances {},
    /// Can't be called through the packet endpoint directly
    Internal(InternalAction),
}

impl HostAction {
    pub fn into_packet(
        self,
        account_id: AccountId,
        retries: u8,
        host_chain: ChainName,
        callback_info: Option<CallbackInfo>,
    ) -> PacketMsg {
        PacketMsg {
            host_chain,
            retries,
            callback_info,
            account_id,
            action: self,
        }
    }
}
/// This is the message we send over the IBC channel
#[cosmwasm_schema::cw_serde]
pub struct PacketMsg {
    /// `ChainName` of the host
    pub host_chain: ChainName,
    /// Amount of retries to attempt if packet returns with StdAck::Error
    pub retries: u8,
    pub account_id: AccountId,
    /// Callback performed after receiving an StdAck::Result
    pub callback_info: Option<CallbackInfo>,
    /// execute the custom host function
    pub action: HostAction,
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin {
        admin: String,
    },
    UpdateConfig {
        ans_host_address: Option<String>,
        account_factory_address: Option<String>,
        version_control_address: Option<String>,
    },
    RegisterChainClient {
        chain_id: String,
        client: String,
    },
    /// Allow for fund recovery through the Admin
    RecoverAccount {
        closed_channel: String,
        account_id: AccountId,
        msgs: Vec<CosmosMsg>,
    },
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns [`ConfigResponse`].
    #[returns(ConfigResponse)]
    Config {},
    #[returns(RegisteredChainsResponse)]
    RegisteredChains {},
    #[returns(RegisteredChainResponse)]
    AssociatedClient { chain: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub account_factory_address: Addr,
    pub version_control_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredChainsResponse {
    pub chains: Vec<(ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredChainResponse {
    pub client: String,
}
