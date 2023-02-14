use crate::{
    state::{AppContract, AppState},
    AppError,
};
use crate::{Handler, InstantiateEndpoint};
use abstract_os::objects::module_version::set_module_data;
use abstract_sdk::helpers::cosmwasm_std::wasm_smart_query;
use abstract_sdk::{
    feature_objects::AnsHost,
    os::{
        app::{BaseInstantiateMsg, InstantiateMsg},
        module_factory::{ContextResponse, QueryMsg as FactoryQuery},
    },
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > InstantiateEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(
        self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let BaseInstantiateMsg { ans_host_address } = msg.base;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = deps.querier.query(&wasm_smart_query(
            info.sender.to_string(),
            &FactoryQuery::Context {},
        )?)?;

        let Some(core) = resp.core else {
            return Err(
                StdError::generic_err("context of module factory not properly set.").into(),
            );
        };

        // Base state
        let state = AppState {
            proxy_address: core.proxy.clone(),
            ans_host,
        };
        let (name, version, metadata) = self.info();
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps.branch(), Some(core.manager))?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.app)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_common::*;

    use abstract_testing::{TEST_ANS_HOST, TEST_MODULE_FACTORY};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
            app: MockInitMsg {},
        };

        let res = MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();
        assert_that!(res.messages).is_empty();
    }
}
