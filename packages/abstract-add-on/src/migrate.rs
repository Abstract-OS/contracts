use abstract_os::add_on::MigrateMsg;
use abstract_sdk::{MigrateEndpoint, Handler};
use cosmwasm_std::{StdError, Response};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use crate::{AddOnError, AddOnContract};

impl<
Error: From<cosmwasm_std::StdError> + From<AddOnError>,
CustomExecMsg,
CustomInitMsg,
CustomQueryMsg,
CustomMigrateMsg,
ReceiveMsg,
>
MigrateEndpoint for AddOnContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg,CustomMigrateMsg, ReceiveMsg>
{
    type MigrateMsg<Msg> = MigrateMsg<CustomMigrateMsg>;

    fn migrate(
        self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Self::MigrateMsg<Self::CustomMigrateMsg>,
    ) -> Result<cosmwasm_std::Response, Self::Error> 
        {
        let (name, version_string) = self.info();
        let version: Version = Version::parse(version_string).map_err(|e|StdError::generic_err(e.to_string()))?;
        let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
        if storage_version < version {
            set_contract_version(deps.storage, name, version_string)?;
        }
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.custom);
        }
        Ok(Response::default())
    }
}