use crate::ApiError;
use abstract_os::objects::dependency::StaticDependency;
use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
        QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
    },
    feature_objects::AnsHost,
    namespaces::BASE_STATE,
    os::version_control::Core,
    AbstractSdkError,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

pub const TRADER_NAMESPACE: &str = "traders";

/// The BaseState contains the main addresses needed for sending and verifying messages
/// Every DApp should use the provided **ans_host** contract for token/contract address resolution.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}

/// The state variables for our ApiContract.
pub struct ApiContract<
    Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError> + 'static,
    CustomInitMsg: 'static = Empty,
    CustomExecMsg: 'static = Empty,
    CustomQueryMsg: 'static = Empty,
    Receive: 'static = Empty,
> {
    pub(crate) contract:
        AbstractContract<Self, Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, Empty, Receive>,
    pub(crate) base_state: Item<'static, ApiState>,
    /// Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'static, Addr, HashSet<Addr>>,
    /// The OS on which commands are executed. Set each time in the [`abstract_os::api::ExecuteMsg::Base`] handler.
    pub target_os: Option<Core>,
}

/// Constructor
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    pub const fn new(
        name: &'static str,
        version: &'static str,
        metadata: Option<&'static str>,
    ) -> Self {
        Self {
            contract: AbstractContract::new(name, version, metadata),
            base_state: Item::new(BASE_STATE),
            traders: Map::new(TRADER_NAMESPACE),
            target_os: None,
        }
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.contract = self.contract.with_dependencies(dependencies);
        self
    }

    pub const fn with_replies(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_replies([&[], reply_handlers]);
        self
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'static [(&'static str, IbcCallbackHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_ibc_callbacks(callbacks);
        self
    }
    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Self, CustomInitMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_instantiate(instantiate_handler);
        self
    }

    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Self, ReceiveMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_receive(receive_handler);
        self
    }

    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Self, CustomExecMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_execute(execute_handler);
        self
    }

    pub const fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Self, CustomQueryMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    /// Return the address of the proxy for the OS associated with this API.
    /// Set each time in the [`abstract_os::api::ExecuteMsg::Base`] handler.
    pub fn target(&self) -> Result<&Addr, ApiError> {
        Ok(&self
            .target_os
            .as_ref()
            .ok_or_else(|| StdError::generic_err("No target OS specified to execute on."))?
            .proxy)
    }
}
