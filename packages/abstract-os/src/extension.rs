//! # Abstract Extension Base
//!
//! `abstract_os::extension` implements shared functionality that's useful for creating new Abstract extensions.
//!
//! ## Description
//! An Abstract extension contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The extension structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use serde::Serialize;

use crate::base::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    QueryMsg as MiddlewareQueryMsg,
};

pub type ExecuteMsg<T, R = Empty> = MiddlewareExecMsg<BaseExecuteMsg, ExtensionRequestMsg<T>, R>;
pub type QueryMsg<T = Empty> = MiddlewareQueryMsg<BaseQueryMsg, T>;
pub type InstantiateMsg<T = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, T>;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddExtension`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation
    pub ans_host_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

impl<T, R> From<BaseExecuteMsg> for MiddlewareExecMsg<BaseExecuteMsg, T, R> {
    fn from(extension_msg: BaseExecuteMsg) -> Self {
        Self::Base(extension_msg)
    }
}

impl<T, R, Q> From<ExtensionRequestMsg<T>> for MiddlewareExecMsg<Q, ExtensionRequestMsg<T>, R> {
    fn from(request_msg: ExtensionRequestMsg<T>) -> Self {
        Self::App(request_msg)
    }
}

/// An extension request.
/// If proxy is None, then the sender must be an OS manager and the proxy address is extrapolated from the OS id.
#[cosmwasm_schema::cw_serde]
pub struct ExtensionRequestMsg<T> {
    pub proxy_address: Option<String>,
    /// The actual request
    pub request: T,
}

impl<T: Serialize> ExtensionRequestMsg<T> {
    pub fn new(proxy_address: Option<String>, request: T) -> Self {
        Self {
            proxy_address,
            request,
        }
    }
}

/// Configuration message for the extension
#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Add or remove traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
    /// Remove the extension
    Remove {},
}

/// Query extension message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`ExtensionConfigResponse`].
    #[returns(ExtensionConfigResponse)]
    Config {},
    /// Returns [`TradersResponse`].
    /// TODO: enable pagination
    #[returns(TradersResponse)]
    Traders { proxy_address: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ExtensionConfigResponse {
    pub version_control_address: Addr,
    pub ans_host_address: Addr,
    pub dependencies: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct TradersResponse {
    /// Contains all traders
    pub traders: Vec<Addr>,
}
