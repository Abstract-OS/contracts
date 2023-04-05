use crate::{AppContract, AppError, Handler, MigrateEndpoint};
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_core::{
    app::MigrateMsg,
    objects::module_version::{get_module_data, set_module_data},
};
use cosmwasm_std::{Response, StdError};
use cw2::set_contract_version;
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg,
    > MigrateEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
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
        assert_contract_upgrade(storage_version, version)?;
        set_module_data(
            deps.storage,
            name,
            version_string,
            self.dependencies(),
            metadata,
        )?;
        set_contract_version(deps.storage, name, version_string)?;
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.module);
        }
        Ok(Response::default())
    }
}
