use abstract_os::objects::module::ModuleVersion;
use abstract_os::{ANS_HOST, VERSION_CONTROL};
use boot_core::BootError::StdErr;
use boot_core::{prelude::*, BootEnvironment, BootError};
use cosmwasm_std::Addr;
use semver::Version;
use serde::Serialize;
use std::fmt::Debug;

use crate::{AnsHost, VersionControl};

/// An Abstract module deployer that can deploy modules to a chain.
pub struct ModuleDeployer<'a, Chain: BootEnvironment> {
    pub chain: &'a Chain,
    pub version: Version,
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
}

impl<'a, Chain: BootEnvironment> ModuleDeployer<'a, Chain> {
    /// Create a new instance of the module deployer, loaded from the STATE_FILE.
    pub fn new(chain: &'a Chain, version: Version) -> Self {
        let ans_host = AnsHost::new(ANS_HOST, chain);
        let version_control = VersionControl::new(VERSION_CONTROL, chain);
        Self {
            chain,
            ans_host,
            version_control,
            version,
        }
    }
    /// Loads a deployment instance from a live chain given the **version_control_address**.
    pub fn load_from_version_control(
        chain: &'a Chain,
        abstract_version: &Version,
        version_control_address: &Addr,
    ) -> Result<Self, BootError> {
        let version_control = VersionControl::load(chain, version_control_address);

        // TODO: get the version dynamically
        // let info = &self.chain.runtime.block_on(DaemonQuerier::contract_info(
        //     chain.sender.channel(),
        //     self.address()?,
        // ))?;

        let result = version_control.get_api_addr(ANS_HOST, ModuleVersion::Latest);

        let ans_host = AnsHost::load(chain, &result?);

        Ok(Self {
            chain,
            ans_host,
            version_control,
            version: abstract_version.clone(),
        })
    }

    /// Uploads, instantiates, and registers a new API module.
    pub fn deploy_api<TInitMsg>(
        &self,
        api: &mut Contract<Chain>,
        version: Version,
        api_init_msg: TInitMsg,
    ) -> Result<(), BootError>
    where
        TInitMsg: Serialize + Debug,
    {
        // check for existing version
        let version_check = self
            .version_control
            .get_api_addr(&api.id, ModuleVersion::from(version.to_string()));

        if version_check.is_ok() {
            return Err(StdErr(format!(
                "API {} already exists with version {}",
                api.id, version
            )));
        };

        api.upload()?;
        let init_msg = abstract_os::api::InstantiateMsg {
            app: api_init_msg,
            base: abstract_os::api::BaseInstantiateMsg {
                ans_host_address: self.ans_host.address()?.into(),
                version_control_address: self.version_control.address()?.into(),
            },
        };
        api.instantiate(&init_msg, None, None)?;

        self.version_control.register_apis(vec![api], &version)?;
        Ok(())
    }
}
