pub mod api;
pub mod app;
pub mod bank;
pub mod execution;
pub mod ibc;
pub mod modules;
pub mod respond;
mod splitter;
pub mod vault;
pub mod verify;
pub mod version_registry;

#[cfg(test)]
mod test_common {
    use crate::{
        features::{AbstractNameService, Dependencies, Identification, ModuleIdentification},
        AbstractSdkResult,
    };
    pub use abstract_testing::{prelude::*, *};
    // pub use abstract_testing::{mock_module::*, *};
    pub use cosmwasm_std::{testing::*, *};
    use os::objects::{ans_host::AnsHost, dependency::StaticDependency, module::ModuleId};
    // pub use speculoos::prelude::*;

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

    impl Dependencies for MockModule {
        fn dependencies(&self) -> &[StaticDependency] {
            &[TEST_MODULE_DEP]
        }
    }

    pub const TEST_MODULE_DEP: StaticDependency =
        StaticDependency::new(TEST_MODULE_ID, &[">1.0.0"]);
    /// Nonexistent module
    pub const FAKE_MODULE_ID: ModuleId = "fake_module";
}
