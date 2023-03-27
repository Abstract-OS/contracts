use crate::{
    state::{Host, HostState, CLOSED_CHANNELS},
    HostError,
};
use abstract_interface::objects::module_version::set_module_data;
use abstract_sdk::{
    base::{Handler, InstantiateEndpoint},
    feature_objects::AnsHost,
    interfaces::ibc_host::InstantiateMsg,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError> + From<abstract_sdk::AbstractSdkError>,
        CustomInitMsg: Serialize + JsonSchema,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    /// Instantiate the api
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(
        self,
        mut deps: DepsMut,
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
        };
        let (name, version, metadata) = self.info();
        // Keep track of all the closed channels, allows for fund recovery if channel closes.
        CLOSED_CHANNELS.save(deps.storage, &vec![])?;
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;

        self.base_state.save(deps.storage, &state)?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        self.admin.set(deps.branch(), Some(info.sender.clone()))?;
        handler(deps, env, info, self, msg.module)
    }
}
