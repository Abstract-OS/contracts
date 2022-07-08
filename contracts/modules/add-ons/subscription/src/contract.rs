use abstract_add_on::AddOnContract;

use abstract_os::SUBSCRIPTION;
use abstract_sdk::version_control::get_os_core;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::Asset;

use semver::Version;

use crate::commands;
use crate::commands::BLOCKS_PER_MONTH;
use crate::error::SubscriptionError;
use abstract_os::subscription::state::*;
use abstract_os::subscription::{
    ConfigResponse, ContributorStateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StateResponse, SubscriberStateResponse, SubscriptionFeeResponse,
};

pub type SubscriptionResult = Result<Response, SubscriptionError>;
pub type SubscriptionAddOn<'a> = AddOnContract<'a>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> SubscriptionResult {
    let _version = CONTRACT_VERSION.parse::<Version>()?;
    let _storage_version = get_contract_version(deps.storage)?
        .version
        .parse::<Version>()?;
    set_contract_version(deps.storage, SUBSCRIPTION, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> SubscriptionResult {
    let subscription_config: SubscriptionConfig = SubscriptionConfig {
        payment_asset: msg.subscription.payment_asset.check(deps.api, None)?,
        subscription_cost_per_block: msg.subscription.subscription_cost_per_block,
        version_control_address: deps
            .api
            .addr_validate(&msg.subscription.version_control_addr)?,
        factory_address: deps.api.addr_validate(&msg.subscription.factory_addr)?,
        subscription_per_block_emissions: msg
            .subscription
            .subscription_per_block_emissions
            .check(deps.api)?,
    };

    let subscription_state: SubscriptionState = SubscriptionState { active_subs: 0 };

    // Optional contribution setup
    if let Some(msg) = msg.contribution {
        let contributor_config: ContributionConfig = ContributionConfig {
            emissions_amp_factor: msg.emissions_amp_factor,
            emission_user_share: msg.emission_user_share,
            emissions_offset: msg.emissions_offset,
            protocol_income_share: msg.protocol_income_share,
            max_emissions_multiple: msg.max_emissions_multiple,
            token_info: msg.token_info.check(deps.api, None)?,
        }
        .verify()?;

        let contributor_state: ContributionState = ContributionState {
            income_target: Decimal::zero(),
            expense: Decimal::zero(),
            total_weight: Uint128::zero(),
            emissions: Decimal::zero(),
        };
        CONTRIBUTION_CONFIG.save(deps.storage, &contributor_config)?;
        CONTRIBUTION_STATE.save(deps.storage, &contributor_state)?;
        INCOME_TWA.instantiate(deps.storage, &env, None, msg.income_averaging_period.u64())?;
    }

    SubscriptionAddOn::default().instantiate(
        deps.branch(),
        env,
        info,
        msg.base,
        SUBSCRIPTION,
        CONTRACT_VERSION,
    )?;

    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> SubscriptionResult {
    let add_on = SubscriptionAddOn::default();

    match msg {
        ExecuteMsg::Base(message) => add_on
            .execute(deps, env, info, message)
            .map_err(|e| e.into()),
        ExecuteMsg::Receive(msg) => commands::receive_cw20(add_on, deps, env, info, msg),
        ExecuteMsg::Pay { os_id } => {
            let maybe_received_coin = info.funds.last();
            if let Some(coin) = maybe_received_coin.cloned() {
                commands::try_pay(add_on, deps, env, info, Asset::from(coin), os_id)
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        ExecuteMsg::Unsubscribe { os_ids } => commands::unsubscribe(deps, env, add_on, os_ids),
        ExecuteMsg::ClaimCompensation { os_id } => {
            commands::try_claim_compensation(add_on, deps, env, os_id)
        }
        ExecuteMsg::ClaimEmissions { os_id } => {
            commands::claim_subscriber_emissions(&add_on, deps.as_ref(), &env, os_id)
        }
        ExecuteMsg::UpdateContributor {
            contributor_os_id,
            base_per_block,
            weight,
            expiration_block,
        } => commands::update_contributor_compensation(
            deps,
            env,
            info,
            add_on,
            contributor_os_id,
            base_per_block,
            weight.map(|w| w.u64() as u32),
            expiration_block.map(|w| w.u64()),
        ),
        ExecuteMsg::RemoveContributor { os_id } => commands::remove_contributor(deps, info, os_id),
        ExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            version_control_address,
            factory_address,
            subscription_cost,
        } => commands::update_subscription_config(
            deps,
            env,
            info,
            payment_asset,
            version_control_address,
            factory_address,
            subscription_cost,
        ),
        ExecuteMsg::UpdateContributionConfig {
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        } => commands::update_contribution_config(
            deps,
            env,
            info,
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => SubscriptionAddOn::default().query(deps, env, message),
        // handle dapp-specific queries here
        QueryMsg::State {} => {
            let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
            let contributor_state = CONTRIBUTION_STATE.load(deps.storage)?;
            to_binary(&StateResponse {
                contribution: contributor_state,
                subscription: subscription_state,
            })
        }
        QueryMsg::Fee {} => {
            let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let minimal_cost = Uint128::from(BLOCKS_PER_MONTH) * config.subscription_cost_per_block;
            to_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: minimal_cost,
                },
            })
        }
        QueryMsg::Config {} => {
            let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let contributor_config = CONTRIBUTION_CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                contribution: contributor_config,
                subscription: subscription_config,
            })
        }
        QueryMsg::SubscriberState { os_id } => {
            let maybe_sub = SUBSCRIBERS.may_load(deps.storage, os_id)?;
            let maybe_dormant_sub = DORMANT_SUBSCRIBERS.may_load(deps.storage, os_id)?;
            let subscription_state = if let Some(sub) = maybe_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else if let Some(sub) = maybe_dormant_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else {
                return Err(StdError::generic_err("os has os_id 0 or does not exist"));
            };
            Ok(subscription_state)
        }
        QueryMsg::ContributorState { os_id } => {
            let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let contributor_addr = get_os_core(
                &deps.querier,
                os_id,
                &subscription_config.version_control_address,
            )?
            .manager;
            let maybe_contributor = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;
            let subscription_state = if let Some(compensation) = maybe_contributor {
                to_binary(&ContributorStateResponse { compensation })?
            } else {
                return Err(StdError::generic_err(
                    "provided address is not a contributor",
                ));
            };
            Ok(subscription_state)
        }
    }
}
