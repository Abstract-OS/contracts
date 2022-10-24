use std::{collections::HashSet, marker::PhantomData};

use abstract_os::version_control::Core;
use abstract_sdk::{memory::Memory, BASE_STATE};

use cosmwasm_std::{Addr, Deps, Empty, Env, StdError, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use simple_ica::IbcResponseMsg;

use crate::{execute::IbcHandlerFn, ApiError};

pub const TRADER_NAMESPACE: &str = "traders";

/// The state variables for our ApiContract.
pub struct ApiContract<
    'a,
    Request: Serialize + DeserializeOwned,
    Error: From<cosmwasm_std::StdError> + From<ApiError>,
    Callback: Serialize + DeserializeOwned = Empty,
> {
    // Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'a, Addr, HashSet<Addr>>,
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, ApiState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,

    pub dependencies: &'static [&'static str],
    pub ibc_callbacks: &'a [(&'static str, IbcHandlerFn<Request, Error, Callback>)],

    pub target_os: Option<Core>,
    _phantom_data: PhantomData<Request>,
    _phantom_data_error: PhantomData<Request>,
    _phantom_data_callbacks: PhantomData<Callback>,
}

impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<ApiError>,
    > Default for ApiContract<'a, T, E, C>
{
    fn default() -> Self {
        Self::new(&[])
    }
}

/// Constructor
impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<ApiError>,
    > ApiContract<'a, T, E, C>
{
    pub const fn new(dependencies: &'static [&'static str]) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(BASE_STATE),
            traders: Map::new(TRADER_NAMESPACE),
            target_os: None,
            ibc_callbacks: &[],
            dependencies,
            _phantom_data: PhantomData,
            _phantom_data_callbacks: PhantomData,
            _phantom_data_error: PhantomData,
        }
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'a [(&'static str, IbcHandlerFn<T, E, C>)],
    ) -> Self {
        self.ibc_callbacks = callbacks;
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }

    pub fn target(&self) -> Result<&Addr, ApiError> {
        Ok(&self
            .target_os
            .as_ref()
            .ok_or(ApiError::NoTargetOS {})?
            .proxy)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
