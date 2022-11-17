use abstract_sdk::os::{
    dex::{OfferAsset, SimulateSwapResponse},
    objects::AssetEntry,
};
use abstract_sdk::AnsInterface;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

use crate::{contract::resolve_exchange, contract::DEX_EXTENSION};

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    mut offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> StdResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;
    let extension = DEX_EXTENSION;
    let ans = extension.ans(deps);
    // format input
    offer_asset.info.format();
    ask_asset.format();
    // get addresses
    let swap_offer_asset = ans.query(&offer_asset)?;
    let ask_asset_info = ans.query(&ask_asset)?;
    let pair_address =
        exchange.pair_address(deps, ans.host(), &mut vec![&offer_asset.info, &ask_asset])?;
    let pool_info = exchange.pair_contract(&mut vec![&offer_asset.info, &ask_asset]);

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.info
    };
    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset, commission_amount),
    };
    to_binary(&resp).map_err(From::from)
}
