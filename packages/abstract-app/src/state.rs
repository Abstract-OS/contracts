use abstract_sdk::{
    feature_objects::AnsHost,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
};
use cosmwasm_std::{Addr, Empty, StdResult, Storage};

use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
};

use crate::AppError;

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AppState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}
/// The state variables for our AppContract.
pub struct AppContract<
    Error: From<cosmwasm_std::StdError> + From<AppError> + 'static,
    CustomExecMsg: 'static = Empty,
    CustomInitMsg: 'static = Empty,
    CustomQueryMsg: 'static = Empty,
    CustomMigrateMsg: 'static = Empty,
    Receive: 'static = Empty,
> {
    // Scaffolding contract that handles type safety and provides helper methods
    pub(crate) contract: AbstractContract<
        Self,
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        Receive,
    >,
    // Custom state for every App
    pub admin: Admin<'static>,
    pub(crate) base_state: Item<'static, AppState>,
}

/// Constructor
impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self {
            base_state: Item::new(BASE_STATE),
            admin: Admin::new(ADMIN_NAMESPACE),
            contract: AbstractContract::new(name, version),
        }
    }

    pub fn load_state(&self, store: &dyn Storage) -> StdResult<AppState> {
        self.base_state.load(store)
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [&'static str]) -> Self {
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

    pub const fn with_migrate(
        mut self,
        migrate_handler: MigrateHandlerFn<Self, CustomMigrateMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_migrate(migrate_handler);
        self
    }

    pub const fn with_query(mut self, query_handler: QueryHandlerFn<Self, CustomQueryMsg>) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }
}
