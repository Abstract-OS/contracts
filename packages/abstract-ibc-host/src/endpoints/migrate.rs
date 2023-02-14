use crate::{Host, HostError};
use abstract_os::objects::module_version::{get_module_data, set_module_data};
use abstract_sdk::{
    base::{Handler, MigrateEndpoint},
    os::ibc_host::MigrateMsg,
};
use cosmwasm_std::{Response, StdError};
use cw2::set_contract_version;
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg,
    > MigrateEndpoint
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    type MigrateMsg = MigrateMsg<CustomMigrateMsg>;

    fn migrate(
        self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Self::MigrateMsg,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        let (name, version_string, metadata) = self.info();
        let version: Version =
            Version::parse(version_string).map_err(|e| StdError::generic_err(e.to_string()))?;
        let storage_version: Version = get_module_data(deps.storage)?.version.parse().unwrap();
        if storage_version < version {
            set_module_data(
                deps.storage,
                name,
                version_string,
                self.dependencies(),
                metadata,
            )?;
            set_contract_version(deps.storage, name, version_string)?;
        }

        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.app);
        }
        Ok(Response::default())
    }
}
