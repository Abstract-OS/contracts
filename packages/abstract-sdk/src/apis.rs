pub mod bank;
pub mod dex;
pub mod execution;
pub mod ibc;
pub mod modules;
pub mod respond;
pub mod vault;
pub mod verify;
pub mod version_registry;

pub(crate) use crate::base::features::*;

#[cfg(test)]
mod test_common {
    use crate::apis::{AbstractNameService, Identification, ModuleIdentification};
    use crate::AbstractSdkResult;
    pub use abstract_testing::mock_module::*;
    pub use abstract_testing::*;
    pub use cosmwasm_std::testing::*;
    pub use cosmwasm_std::*;
    use os::objects::ans_host::AnsHost;
    pub use speculoos::prelude::*;

    // We implement the following traits here for the mock module (in this package) to avoid a circular dependency
    impl Identification for MockModule {
        fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
            Ok(Addr::unchecked(TEST_PROXY))
        }
    }

    impl ModuleIdentification for MockModule {
        fn module_id(&self) -> &'static str {
            "mock_module"
        }
    }

    impl AbstractNameService for MockModule {
        fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
            Ok(AnsHost {
                address: Addr::unchecked("ans"),
            })
        }
    }
}
