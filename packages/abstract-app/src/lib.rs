mod endpoints;
pub mod error;
pub mod features;
pub(crate) mod handler;
pub mod schema;
pub mod state;
pub(crate) use abstract_sdk::base::*;

pub use crate::state::AppContract;
pub use error::AppError;
pub type AppResult<C = Empty> = Result<Response<C>, AppError>;

use cosmwasm_std::{Empty, Response};
#[cfg(feature = "test-utils")]
pub mod mock {
    pub use abstract_os::app;
    pub use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, to_binary, Addr, StdError};

    pub type AppTestResult = Result<(), MockError>;

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockExecMsg;

    impl app::AppExecuteMsg for MockExecMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryMsg;

    impl app::AppQueryMsg for MockQueryMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    use crate::{AppContract, AppError};
    use abstract_os::{module_factory::ContextResponse, version_control::Core};
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::prelude::{
        MockDeps, MockQuerierBuilder, TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_FACTORY,
        TEST_MODULE_ID, TEST_PROXY, TEST_VERSION,
    };
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        DappError(#[from] AppError),

        #[error("{0}")]
        AbstractOs(#[from] abstract_os::AbstractOsError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    pub type MockAppContract = AppContract<
        // MockModule,
        MockError,
        MockInitMsg,
        MockExecMsg,
        MockQueryMsg,
        MockMigrateMsg,
        MockReceiveMsg,
    >;

    pub const MOCK_APP: MockAppContract = MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None);

    crate::export_endpoints!(MOCK_APP, MockAppContract);

    pub fn app_base_mock_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default().with_smart_handler(TEST_MODULE_FACTORY, |msg| {
            match from_binary(msg).unwrap() {
                abstract_os::module_factory::QueryMsg::Context {} => {
                    let resp = ContextResponse {
                        core: Some(Core {
                            manager: Addr::unchecked(TEST_MANAGER),
                            proxy: Addr::unchecked(TEST_PROXY),
                        }),
                        module: None,
                    };
                    Ok(to_binary(&resp).unwrap())
                }
                _ => panic!("unexpected message"),
            }
        })
    }

    /// Instantiate the contract with the default [`TEST_MODULE_FACTORY`].
    /// This will set the [`TEST_MANAGER`] as the admin.
    pub fn mock_init() -> MockDeps {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
            app: MockInitMsg {},
        };

        MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();

        deps
    }
}
