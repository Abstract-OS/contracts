//! # Abstract API Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract APIs.
//!
//! ## Description
//! An Abstract API contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The API structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg, Empty, QueryRequest};
use serde::Serialize;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation on the app chain
    pub memory_address: String,
    /// Code-id for cw1 proxy contract
    pub cw1_code_id: u64,
}

/// This is the message we send over the IBC channel
#[cosmwasm_schema::cw_serde]
pub enum PacketMsg<T: Serialize> {
    /// execute the custom host function
    App(T),
    Dispatch {
        sender: String,
        msgs: Vec<CosmosMsg<Empty>>,
        callback_id: Option<String>,
    },
    Query {
        sender: String,
        msgs: Vec<QueryRequest<Empty>>,
        callback_id: Option<String>,
    },
    WhoAmI {},
    Balances {},
    SendAllBack {
        sender: String,
    },
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg<Q: Serialize> {
    App(Q),
    /// A configuration message to whitelist traders.
    Base(BaseQueryMsg),
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`HostConfigResponse`].
    #[returns(HostConfigResponse)]
    Config {},
    /// Returns (reflect) account that is attached to this channel,
    /// or none.
    #[returns(AccountResponse)]
    Account { channel_id: String },
    /// Returns all (channel, reflect_account) pairs.
    /// No pagination - this is a test contract
    #[returns(ListAccountsResponse)]
    ListAccounts {},
}

#[cosmwasm_schema::cw_serde]
pub struct HostConfigResponse {
    pub memory_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub account: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountInfo {
    pub account: String,
    pub channel_id: String,
}
