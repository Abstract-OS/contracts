use crate::{
    abstract_ica::StdAck,
    ibc_host::HostAction,
    objects::{account::AccountId, chain_name::ChainName},
};
use abstract_ica::IbcResponseMsg;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{from_slice, Binary, Coin, CosmosMsg, StdResult, Timestamp, Addr};

pub mod state {

    use super::LatestQueryResponse;
    use crate::{
        objects::{
            account::AccountId, ans_host::AnsHost, chain_name::ChainName,
            common_namespace::ADMIN_NAMESPACE,
        },
        ANS_HOST as ANS_HOST_KEY,
    };
    use cosmwasm_std::{Addr};
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_address: Addr,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    /// chain -> channel-id
    /// these channels have been verified by the host. 
    pub const CHANNELS: Map<&ChainName, String> = Map::new("channels");
    pub const CONFIG: Item<Config> = Item::new("config");
    /// (account_id, chain_name) -> remote proxy account address
    pub const ACCOUNTS: Map<(&AccountId, &ChainName), String> = Map::new("accounts");

    pub const ANS_HOST: Item<AnsHost> = Item::new(ANS_HOST_KEY);
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct CallbackInfo {
    pub id: String,
    pub receiver: String,
}

impl CallbackInfo {
    pub fn to_callback_msg(self, ack_data: &Binary) -> StdResult<CosmosMsg> {
        let msg: StdAck = from_slice(ack_data)?;
        IbcResponseMsg { id: self.id, msg }.into_cosmos_account_msg(self.receiver)
    }
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin {
        admin: String,
    },
    /// Changes the config
    UpdateConfig {
        ans_host: Option<String>,
        version_control: Option<String>,
    },
    /// Only callable by Account proxy
    /// Will attempt to forward the specified funds to the corresponding
    /// address on the remote chain.
    SendFunds {
        host_chain: ChainName,
        funds: Vec<Coin>,
    },
    /// Register an Account on a remote chain over IBC
    /// This action creates a proxy for them on the remote chain.
    Register {
        host_chain: ChainName,
    },
    SendPacket {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: ChainName,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_info: Option<CallbackInfo>,
        // Number of retries if packet errors
        retries: u8,
    },
    RemoveHost {
        host_chain: ChainName,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Returns config
    #[returns(ConfigResponse)]
    Config {},
    // Shows all open channels (incl. remote info)
    #[returns(ListAccountsResponse)]
    ListAccounts {},
    // Get channel info for one chain
    #[returns(AccountResponse)]
    Account {
        chain: String,
        account_id: AccountId,
    },
    // get the channels
    #[returns(ListChannelsResponse)]
    ListChannels {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub version_control_address: String,
    pub chain: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<(AccountId, ChainName,String)>,
}
#[cosmwasm_schema::cw_serde]
pub struct ListChannelsResponse {
    pub channels: Vec<(ChainName, String)>,
}
#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub remote_proxy_addr: String,
}

#[cosmwasm_schema::cw_serde]
pub struct LatestQueryResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    pub response: StdAck,
}

#[cosmwasm_schema::cw_serde]
pub struct RemoteProxyResponse {
    /// last block balance was updated (0 is never)
    pub channel_id: String,
    /// address of the remote proxy
    pub proxy_address: String,
}