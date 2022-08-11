use abstract_sdk::{MemoryOperation};
use cosmwasm_std::{Decimal, Deps, Env, MessageInfo};
use cw_asset::{Asset, AssetInfo};

use crate::{
    contract::{DexApi, DexResult},
    error::DexError,
    DEX,
};
use abstract_os::{dex::OfferAsset, objects::AssetEntry};

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};

fn resolve_exchange(value: String) -> Result<&'static dyn DEX, DexError> {
    match value.as_str() {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        _ => Err(DexError::UnknownDex(value)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn swap(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
    max_spread: Option<Decimal>,
    belief_price: Option<Decimal>,
) -> DexResult {
    let exchange = resolve_exchange(dex)?;
    let (mut offer_asset, offer_amount) = offer_asset;
    offer_asset.format();
    ask_asset.format();
    let offer_asset_info = api.resolve(deps, &offer_asset)?;
    let ask_asset_info = api.resolve(deps, &ask_asset)?;

    let pair_address = exchange.pair_address(deps, &api, &mut [offer_asset, ask_asset])?;
    let offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);

    exchange.swap(
        deps,
        api,
        pair_address,
        offer_asset,
        ask_asset_info,
        belief_price,
        max_spread,
    )
}

pub fn provide_liquidity(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_assets: Vec<OfferAsset>,
    dex: String,
    max_spread: Option<Decimal>,
) -> DexResult {
    let exchange = resolve_exchange(dex)?;
    let mut assets = vec![];
    for offer in &offer_assets {
        let info = api.resolve(deps, &offer.0)?;
        let asset = Asset::new(info, offer.1);
        assets.push(asset);
    }
    let pair_address = exchange.pair_address(
        deps,
        &api,
        offer_assets
            .into_iter()
            .map(|(a, _)| a)
            .collect::<Vec<AssetEntry>>()
            .as_mut(),
    )?;
    exchange.provide_liquidity(deps, api, pair_address, assets, max_spread)
}

pub fn provide_liquidity_symmetric(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    offer_asset: OfferAsset,
    mut paired_assets: Vec<AssetEntry>,
    dex: String,
) -> DexResult {
    let exchange = resolve_exchange(dex)?;
    let paired_asset_infos: Result<Vec<AssetInfo>, _> = paired_assets
        .iter()
        .map(|entry| api.resolve(deps, entry))
        .collect();
    paired_assets.push(offer_asset.0.clone());
    let pair_address = exchange.pair_address(deps, &api, &mut paired_assets)?;
    let offer_asset = Asset::new(api.resolve(deps, &offer_asset.0)?, offer_asset.1);
    exchange.provide_liquidity_symmetric(deps, api, pair_address, offer_asset, paired_asset_infos?)
}
