use crate::{
    cw_helpers::wasm_smart_query, features::AbstractRegistryAccess, AbstractSdkError,
    AbstractSdkResult,
};
use abstract_core::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
    },
    version_control::{state::REGISTERED_MODULES, ModulesResponse, QueryMsg},
};
use cosmwasm_std::Deps;

/// Access the Abstract Version Control and access module information.
pub trait ModuleRegistryInterface: AbstractRegistryAccess {
    /**
        API for querying module information from the Abstract version control contract.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref());
        ```
    */
    fn module_registry<'a>(&'a self, deps: Deps<'a>) -> ModuleRegistry<Self> {
        ModuleRegistry { base: self, deps }
    }
}

impl<T> ModuleRegistryInterface for T where T: AbstractRegistryAccess {}

#[derive(Clone)]
/**
    API for querying module information from the Abstract version control contract.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let mod_registry: ModuleRegistry<MockModule>  = module.module_registry(deps.as_ref());
    ```
*/
pub struct ModuleRegistry<'a, T: ModuleRegistryInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleRegistryInterface> ModuleRegistry<'a, T> {
    /// Raw query for a module reference
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
    ) -> AbstractSdkResult<ModuleReference> {
        let registry_addr = self.base.abstract_registry(self.deps)?;
        REGISTERED_MODULES
            .query(&self.deps.querier, registry_addr.clone(), module_info)?
            .ok_or_else(|| AbstractSdkError::ModuleNotFound {
                module: module_info.to_string(),
                registry_addr,
            })
    }

    /// Smart query for a module
    pub fn query_module(&self, module_info: ModuleInfo) -> AbstractSdkResult<Module> {
        let registry_addr = self.base.abstract_registry(self.deps)?;
        let ModulesResponse { mut modules } = self.deps.querier.query(&wasm_smart_query(
            registry_addr.into_string(),
            &QueryMsg::Modules {
                infos: vec![module_info],
            },
        )?)?;
        Ok(modules.swap_remove(0))
    }
}
