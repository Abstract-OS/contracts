use abstract_add_on::AddOnError;
use cosmwasm_std::{OverflowError, StdError, DecimalRangeExceeded};
use cw_asset::AssetInfo;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SubscriptionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("{0}")]
    DecimalError(#[from] DecimalRangeExceeded),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    AddOnError(#[from] AddOnError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("This contract does not implement the cw20 swap function")]
    NoSwapAvailable {},

    #[error("The provided token is not the payment token {0}")]
    WrongToken(AssetInfo),

    #[error("It's required to use cw20 send message to add pay with cw20 tokens")]
    NotUsingCW20Hook {},

    #[error("The provided fee is invalid")]
    InvalidFee {},

    #[error("The actual amount of tokens transferred is different from the claimed amount.")]
    InvalidAmount {},

    #[error("The provided native coin is not the same as the required native coin")]
    WrongNative {},

    #[error("The contributor you wanted to remove is not registered.")]
    ContributorNotRegistered,

    #[error("You can't claim before the end of the current period.")]
    CompensationAlreadyClaimed,

    #[error("Your contribution compensation expired")]
    ContributionExpired,

    #[error("emissions for this OS are already claimed")]
    EmissionsAlreadyClaimed,

    #[error("only the factory can register new subscribers")]
    CallerNotFactory,

    #[error("compensation does not yield you any assets.")]
    NoAssetsToSend,

    #[error("income target is zero, no contributions can be paid out.")]
    TargetIsZero,

    #[error("you need to deposit at least {0} {1} to (re)activate this OS")]
    InsufficientPayment(u64, String),

    #[error("Subscriber emissions are not enabled")]
    SubscriberEmissionsNotEnabled,

    #[error("Contribution function must be enabled to use this feature")]
    ContributionNotEnabled,

    #[error("contributor must be a manager address")]
    ContributorNotManager,

    #[error("no os found with id {0}")]
    OsNotFound(u32),

    #[error("You must wait one TWA period before claiming can start")]
    AveragingPeriodNotPassed,
}

impl From<semver::Error> for SubscriptionError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
