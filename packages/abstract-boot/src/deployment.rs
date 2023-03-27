use crate::{
    get_native_contracts, AbstractAccount, AccountFactory, AnsHost, Manager, ModuleFactory, Proxy,
    VersionControl,
};

use boot_core::*;

use semver::Version;

pub struct Abstract<Chain: BootEnvironment> {
    pub chain: Chain,
    pub version: Version,
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub account_factory: AccountFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
}

use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, PROXY, VERSION_CONTROL};
#[cfg(feature = "integration")]
use boot_core::ContractWrapper;

impl<Chain: BootEnvironment> boot_core::Deploy<Chain> for Abstract<Chain> {
    // We don't have a custom error type
    type Error = BootError;
    type DeployData = semver::Version;

    #[allow(unused_mut)]
    fn deploy_on(chain: Chain, version: semver::Version) -> Result<Self, BootError> {
        let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
        let mut account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
        let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
        let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
        let mut manager = Manager::new(MANAGER, chain.clone());
        let mut proxy = Proxy::new(PROXY, chain.clone());
        #[cfg(feature = "integration")]
        if cfg!(feature = "integration") {
            ans_host
                .as_instance_mut()
                .set_mock(Box::new(ContractWrapper::new_with_empty(
                    ::ans_host::contract::execute,
                    ::ans_host::contract::instantiate,
                    ::ans_host::contract::query,
                )));

            account_factory.as_instance_mut().set_mock(Box::new(
                ContractWrapper::new_with_empty(
                    ::account_factory::contract::execute,
                    ::account_factory::contract::instantiate,
                    ::account_factory::contract::query,
                )
                .with_reply_empty(::account_factory::contract::reply),
            ));

            module_factory.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::module_factory::contract::execute,
                    ::module_factory::contract::instantiate,
                    ::module_factory::contract::query,
                )
                .with_reply_empty(::module_factory::contract::reply),
            ));

            version_control.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::version_control::contract::execute,
                    ::version_control::contract::instantiate,
                    ::version_control::contract::query,
                ),
            ));

            manager.as_instance_mut().set_mock(Box::new(
                boot_core::ContractWrapper::new_with_empty(
                    ::manager::contract::execute,
                    ::manager::contract::instantiate,
                    ::manager::contract::query,
                ),
            ));

            proxy
                .as_instance_mut()
                .set_mock(Box::new(boot_core::ContractWrapper::new_with_empty(
                    ::proxy::contract::execute,
                    ::proxy::contract::instantiate,
                    ::proxy::contract::query,
                )));
        }

        let mut deployment = Abstract {
            chain,
            version,
            ans_host,
            account_factory,
            version_control,
            module_factory,
        };

        let mut account = AbstractAccount { manager, proxy };

        deployment
            .deploy(&mut account)
            .map_err(|e| BootError::StdErr(e.to_string()))?;
        Ok(deployment)
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        let (ans_host, account_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain.clone());
        let version = env!("CARGO_PKG_VERSION").parse().unwrap();
        Ok(Self {
            chain,
            version,
            ans_host,
            version_control,
            account_factory,
            module_factory,
        })
    }
}

impl<Chain: BootEnvironment> Abstract<Chain> {
    pub fn new(chain: Chain, version: Version) -> Self {
        let (ans_host, account_factory, version_control, module_factory, _ibc_client) =
            get_native_contracts(chain.clone());

        Self {
            chain,
            ans_host,
            version_control,
            account_factory,
            module_factory,
            version,
        }
    }

    #[allow(unused)]
    fn get_chain(&self) -> Chain {
        self.chain.clone()
    }

    pub fn upload(
        &mut self,
        account: &mut AbstractAccount<Chain>,
    ) -> Result<(), crate::AbstractBootError> {
        self.ans_host.upload()?;
        self.version_control.upload()?;
        self.account_factory.upload()?;
        self.module_factory.upload()?;

        account.upload()?;

        Ok(())
    }

    pub fn instantiate(&mut self) -> Result<(), crate::AbstractBootError> {
        let sender = &self.chain.sender();

        self.ans_host.instantiate(
            &abstract_core::ans_host::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_core::version_control::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_core::module_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        self.account_factory.instantiate(
            &abstract_core::account_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        Ok(())
    }

    pub fn deploy(
        &mut self,
        account: &mut AbstractAccount<Chain>,
    ) -> Result<(), crate::AbstractBootError> {
        // ########### Upload ##############
        self.upload(account)?;

        // ########### Instantiate ##############
        self.instantiate()?;

        // Set Factory
        self.version_control.execute(
            &abstract_core::version_control::ExecuteMsg::SetFactory {
                new_factory: self.account_factory.address()?.into_string(),
            },
            None,
        )?;

        // ########### upload modules and token ##############

        self.version_control
            .register_base(account, &self.version.to_string())?;

        self.version_control.register_deployment(self)?;

        Ok(())
    }

    pub fn contracts(&self) -> Vec<&Contract<Chain>> {
        vec![
            self.ans_host.as_instance(),
            self.version_control.as_instance(),
            self.account_factory.as_instance(),
            self.module_factory.as_instance(),
        ]
    }
}
