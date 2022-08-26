use abstract_api::{ApiContract, ApiResult};
use abstract_os::{
    api::{ApiInstantiateMsg, ApiInterfaceMsg},
    dex::{QueryMsg, RequestMsg},
    EXCHANGE,
};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    commands::{provide_liquidity, provide_liquidity_symmetric, swap, withdraw_liquidity},
    error::DexError,
};

pub type DexApi<'a> = ApiContract<'a, RequestMsg>;
pub type DexResult = Result<Response, DexError>;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Supported exchanges on XXX
// ...

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    DexApi::instantiate(deps, env, info, msg, EXCHANGE, CONTRACT_VERSION, vec![])?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<RequestMsg>,
) -> DexResult {
    DexApi::handle_request(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    api: DexApi,
    msg: RequestMsg,
) -> DexResult {
    match msg {
        RequestMsg::ProvideLiquidity {
            assets,
            dex,
            max_spread,
        } => {
            let dex_name = dex.unwrap();
            if assets.len() < 2 {
                return Err(DexError::TooFewAssets {});
            }
            provide_liquidity(deps.as_ref(), env, info, api, assets, dex_name, max_spread)
        }
        RequestMsg::ProvideLiquiditySymmetric {
            offer_asset,
            paired_assets,
            dex,
        } => {
            let dex_name = dex.unwrap();
            if paired_assets.is_empty() {
                return Err(DexError::TooFewAssets {});
            }
            provide_liquidity_symmetric(
                deps.as_ref(),
                env,
                info,
                api,
                offer_asset,
                paired_assets,
                dex_name,
            )
        }
        RequestMsg::WithdrawLiquidity {
            lp_token,
            amount,
            dex,
        } => {
            let dex_name = dex.unwrap();
            withdraw_liquidity(deps.as_ref(), env, info, api, (lp_token, amount), dex_name)
        }

        RequestMsg::Swap {
            offer_asset,
            ask_asset,
            dex,
            max_spread,
            belief_price,
        } => {
            // add default dex in future (osmosis??)
            let dex_name = dex.unwrap();
            swap(
                deps.as_ref(),
                env,
                info,
                api,
                offer_asset,
                ask_asset,
                dex_name,
                max_spread,
                belief_price,
            )
        }
    }
    // .map_err(From::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(api_query) => DexApi::default().query(deps, env, api_query),
    }
}
