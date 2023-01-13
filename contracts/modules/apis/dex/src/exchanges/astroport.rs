use crate::{
    dex_trait::{Fee, FeeOnInput, Identify, Return, Spread},
    error::DexError,
    DEX,
};
use abstract_os::objects::PoolAddress;
use abstract_sdk::helpers::cosmwasm_std::wasm_smart_query;
use astroport::pair::{PoolResponse, SimulationResponse};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Deps, StdResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
pub const ASTROPORT: &str = "astroport";

// Source https://github.com/astroport-fi/astroport-core
pub struct Astroport {}

impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
    }
    fn over_ibc(&self) -> bool {
        false
    }
}

/// This structure describes a CW20 hook message.
#[cosmwasm_schema::cw_serde]
pub enum StubCw20HookMsg {
    /// Withdraw liquidity from the pool
    WithdrawLiquidity {},
}

impl DEX for Astroport {
    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => vec![wasm_execute(
                pair_address.to_string(),
                &astroport::pair::ExecuteMsg::Swap {
                    offer_asset: cw_asset_to_astroport(&offer_asset)?,
                    ask_asset_info: None,
                    belief_price,
                    max_spread,
                    to: None,
                },
                vec![offer_asset.clone().try_into()?],
            )?
            .into()],
            AssetInfo::Cw20(addr) => vec![wasm_execute(
                addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&astroport::pair::Cw20HookMsg::Swap {
                        ask_asset_info: None,
                        belief_price,
                        max_spread,
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()],
            AssetInfo::Cw1155(..) => return Err(DexError::Cw1155Unsupported {}),
            _ => panic!("unsupported asset"),
        };
        Ok(swap_msg)
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }
        let mut astroport_assets = offer_assets
            .iter()
            .map(cw_asset_to_astroport)
            .collect::<Result<Vec<_>, _>>()?;

        // if there is only one asset, we need to simulate swap, swap and provide liquidity.
        let mut msgs: Vec<CosmosMsg> = vec![];
        if astroport_assets.len() == 1 {

            let mut offer_asset = offer_assets[0].clone();
            let other_asset = other_asset(deps, &pair_address, &offer_asset)?;

            offer_asset.amount = offer_asset.clone().amount * Decimal::percent(50);
            let astro_offer_asset = cw_asset_to_astroport(&offer_asset)?;

            let simulation: SimulationResponse = deps.querier.query(&wasm_smart_query(
                pair_address.to_string(),
                &astroport::pair::QueryMsg::Simulation {
                    offer_asset: astro_offer_asset.clone(),
                    ask_asset_info: Some(other_asset.clone()),
                })?)?;

            let mut msg = self.swap(deps, pool_id, offer_asset, astroport_assetinfo_to_cw(&other_asset) , None, max_spread)?;
            msgs.append(&mut msg);
        
            astroport_assets = vec![astro_offer_asset, astroport::asset::Asset {
                info: other_asset.into(),
                amount: simulation.return_amount,
            }];
        }

        // execute msg
        let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: astroport_assets,
            slippage_tolerance: max_spread,
            auto_stake: Some(false),
            receiver: None,
        };

        // filter out assets that have amount zero
        let offer_assets = offer_assets
        .into_iter()
        .filter(|asset| !asset.amount.is_zero())
        .collect::<Vec<_>>();
        
        // approval msgs for cw20 tokens (if present)
        msgs.append(&mut cw_approve_msgs(&offer_assets, &pair_address)?);
        let coins = coins_in_assets(&offer_assets);

        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_config: PoolResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Pool {},
        )?)?; 
        let astroport_offer_asset = cw_asset_to_astroport(&offer_asset)?;
        let other_asset = if pair_config.assets[0].info == astroport_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[1].amount, pair_config.assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_config.assets[1].info == astroport_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[0].amount, pair_config.assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config
                    .assets
                    .iter()
                    .map(|e| e.info.to_string())
                    .collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;

        // construct execute msg
        let astroport_assets = offer_assets
            .iter()
            .map(cw_asset_to_astroport)
            .collect::<Result<Vec<_>, _>>()?;

        let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: vec![astroport_assets[0].clone(), astroport_assets[1].clone()],
            slippage_tolerance: None,
            receiver: None,
            auto_stake: None,
        };

        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        #[cfg(not(feature = "testing"))]
        let hook_msg = StubCw20HookMsg::WithdrawLiquidity {};
        #[cfg(feature = "testing")]
        let hook_msg = astroport::pair::Cw20HookMsg::WithdrawLiquidity { assets: vec![] };

        let withdraw_msg = lp_token.send_msg(pair_address, to_binary(&hook_msg)?)?;
        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_address = pool_id.expect_contract()?;
        // Do simulation
        let SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Simulation {
                offer_asset: cw_asset_to_astroport(&offer_asset)?,
                ask_asset_info: None,
            },
        )?)?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

fn other_asset(deps: Deps, pair_address: &Addr, offer_asset: &Asset ) -> Result<astroport::asset::AssetInfo, DexError> {
    // Get pair info
    let pair_config: PoolResponse = deps.querier.query(&wasm_smart_query(
        pair_address.to_string(),
        &astroport::pair::QueryMsg::Pool {},
    )?)?;
    let astroport_offer_asset = cw_asset_to_astroport(offer_asset)?;

    let other_asset = if pair_config.assets[0].info == astroport_offer_asset.info {
        pair_config.assets[1].clone()
    } else if pair_config.assets[1].info == astroport_offer_asset.info {
        pair_config.assets[0].clone()
    } else {
        return Err(DexError::ArgumentMismatch(
            offer_asset.to_string(),
            pair_config
                .assets
                .iter()
                .map(|e| e.info.to_string())
                .collect(),
        ));
    };
    Ok(other_asset.info)
}

fn astroport_assetinfo_to_cw(asset_info: &astroport::asset::AssetInfo) -> AssetInfo {
    match asset_info {
        astroport::asset::AssetInfo::NativeToken { denom } => AssetInfo::Native( denom.clone() ),
        astroport::asset::AssetInfo::Token { contract_addr } => AssetInfo::Cw20(contract_addr.clone()),
    }
}

fn cw_asset_to_astroport(asset: &Asset) -> Result<astroport::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::Token {
                contract_addr: contract_addr.clone(),
            },
        }),
        _ => Err(DexError::Cw1155Unsupported {}),
    }
}

fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        if let AssetInfo::Cw20(addr) = &asset.info {
            let msg = cw20_junoswap::Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: asset.amount,
                expires: None,
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }))
        }
    }
    Ok(msgs)
}

fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        if let AssetInfo::Native(denom) = &asset.info {
            coins.push(Coin::new(asset.amount.u128(), denom.clone()));
        }
    }
    coins
}
