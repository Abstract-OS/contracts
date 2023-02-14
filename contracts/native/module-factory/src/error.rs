use abstract_os::AbstractOsError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractOsError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("Asset error encountered while handling assets: {0}")]
    CwAsset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Calling contract is not a registered OS Manager")]
    UnknownCaller(),

    #[error("Reply ID does not match any known Reply ID")]
    UnexpectedReply(),

    #[error("This module type can not be installed on your OS")]
    ModuleNotInstallable {},
}
