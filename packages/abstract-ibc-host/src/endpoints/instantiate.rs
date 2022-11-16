use abstract_os::ibc_host::InstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use abstract_sdk::{
    base::{Handler, InstantiateEndpoint},
    feature_objects::AnsHost,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    state::{Host, HostState, CLOSED_CHANNELS},
    HostError,
};

use cw2::set_contract_version;

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    /// Instantiate the API
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&msg.base.ans_host_address)?,
        };

        // Base state
        let state = HostState {
            chain: self.chain.to_string(),
            ans_host,
            cw1_code_id: msg.base.cw1_code_id,
            admin: info.sender.clone(),
        };
        let (name, version) = self.info();
        // Keep track of all the closed channels, allows for fund recovery if channel closes.
        let closed_channels = vec![];
        CLOSED_CHANNELS.save(deps.storage, &closed_channels)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.app)
    }
}