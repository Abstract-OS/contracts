pub use abstract_core::manager::{ExecuteMsgFns as ManagerExecFns, QueryMsgFns as ManagerQueryFns};
use abstract_core::{
    api,
    manager::*,
    objects::module::{ModuleInfo, ModuleVersion},
};
use boot_core::{boot_contract, BootEnvironment, BootExecute, Contract};
use cosmwasm_std::{to_binary, Empty};
use serde::Serialize;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Manager<Chain>;

impl<Chain: BootEnvironment> Manager<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("abstract_manager");
        Self(contract)
    }

    pub fn upgrade_module<M: Serialize>(
        &self,
        module_id: &str,
        migrate_msg: &M,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::Upgrade {
                modules: vec![(
                    ModuleInfo::from_id(module_id, ModuleVersion::Latest)?,
                    Some(to_binary(migrate_msg).unwrap()),
                )],
            },
            None,
        )?;
        Ok(())
    }

    pub fn replace_api(&self, module_id: &str) -> Result<(), crate::AbstractBootError> {
        // this should check if installed?
        self.uninstall_module(module_id.to_string())?;

        self.install_module(module_id, &Empty {})
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: &TInitMsg,
    ) -> Result<(), crate::AbstractBootError> {
        self.install_module_version(module_id, ModuleVersion::Latest, init_msg)
    }

    pub fn install_module_version<M: Serialize>(
        &self,
        module_id: &str,
        version: ModuleVersion,
        init_msg: &M,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(module_id, version)?,
                init_msg: Some(to_binary(init_msg).unwrap()),
            },
            None,
        )?;
        Ok(())
    }

    pub fn execute_on_module(
        &self,
        module: &str,
        msg: impl Serialize,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::ExecOnModule {
                module_id: module.into(),
                exec_msg: to_binary(&msg).unwrap(),
            },
            None,
        )?;
        Ok(())
    }

    pub fn update_api_traders(
        &self,
        module_id: &str,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute_on_module(
            module_id,
            api::ExecuteMsg::<Empty, Empty>::Base(api::BaseExecuteMsg::UpdateTraders {
                to_add,
                to_remove,
            }),
        )?;

        Ok(())
    }

    /// Return the module info installed on the manager
    pub fn module_info(
        &self,
        module_id: &str,
    ) -> Result<Option<ManagerModuleInfo>, crate::AbstractBootError> {
        let module_infos = self.module_infos(None, None)?.module_infos;
        let found = module_infos
            .into_iter()
            .find(|module_info| module_info.id == module_id);
        Ok(found)
    }

    pub fn is_module_installed(&self, module_id: &str) -> Result<bool, crate::AbstractBootError> {
        let module = self.module_info(module_id)?;
        Ok(module.is_some())
    }
}
