use crate::{
    dex_trait::{Fee, FeeOnInput, Return, Spread},
    error::DexError,
    DEX,
};

use cosmwasm_std::{
    from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, QueryRequest,
    StdError, StdResult, Uint128, Uint256,
};
use cw_asset::{Asset, AssetInfo};

use osmosis_std::types::{
    cosmos::base::v1beta1::Coin as OsmoCoin,
    osmosis::gamm::v1beta1::{Pool, QueryPoolRequest, QueryPoolResponse},
};

use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgExitPool, MsgJoinPool, MsgSwapExactAmountIn, QuerySwapExactAmountInRequest,
    QuerySwapExactAmountInResponse, SwapAmountInRoute,
};

pub const OSMOSIS: &str = "osmosis";
// Source https://github.com/wasmswap/wasmswap-contracts
pub struct Osmosis {
    pub local_proxy_addr: Option<Addr>,
}

/// Osmosis app-chain dex implementation
impl DEX for Osmosis {
    fn over_ibc(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        OSMOSIS
    }

    fn swap(
        &self,
        _deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        _belief_price: Option<Decimal>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let token_out_denom = match ask_asset {
            AssetInfo::Native(denom) => denom,
            _ => return Err(DexError::Cw1155Unsupported),
        };

        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom,
        }];

        let token_in = Coin::try_from(offer_asset)?;

        let swap_msg: CosmosMsg = MsgSwapExactAmountIn {
            sender: self.local_proxy_addr.as_ref().unwrap().to_string(),
            routes,
            token_in: Some(token_in.into()),
            token_out_min_amount: Uint128::zero().to_string(),
        }
        .into();

        Ok(vec![swap_msg])
    }

    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        // The offer_assets have already been sent to the host contract
        // The ask_assets are the assets we want to receive
        // Generate the swap message(s) between the offer and ask assets
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }

        let pool_id = pair_address.to_string().parse::<u64>().unwrap();
        let token_in_maxs: Vec<OsmoCoin> = offer_assets
            .iter()
            .map(|asset| Coin::try_from(asset).unwrap().into())
            .collect();

        let pool = query_pool_data(deps, pool_id);

        // check for symmetric pools
        if pool.pool_assets[0].weight != pool.pool_assets[1].weight {
            return Err(DexError::BalancerNotSupported(OSMOSIS.to_string()));
        }

        let pool_assets: Vec<OsmoCoin> = pool
            .pool_assets
            .into_iter()
            .map(|asset| asset.token.unwrap())
            .collect();

        let deposits: [Uint128; 2] = [
            token_in_maxs
                .iter()
                .find(|coin| coin.denom == pool_assets[0].denom)
                .map(|coin| coin.amount.parse::<Uint128>().unwrap())
                .expect("wrong asset provided"),
            token_in_maxs
                .iter()
                .find(|coin| coin.denom == pool_assets[0].denom)
                .map(|coin| coin.amount.parse::<Uint128>().unwrap())
                .expect("wrong asset provided"),
        ];

        assert_slippage_tolerance(&max_spread, &deposits, pool_assets)?;

        let total_share = pool
            .total_shares
            .unwrap()
            .amount
            .parse::<Uint128>()
            .unwrap();

        let share_out_amount =
            compute_osmo_share_out_amount(&pool_assets, &deposits, total_share)?.to_string();

        let osmo_msg: CosmosMsg = MsgJoinPool {
            sender: self.local_proxy_addr.as_ref().unwrap().to_string(),
            pool_id,
            share_out_amount,
            token_in_maxs,
        }
        .into();

        Ok(vec![osmo_msg])
    }

    fn provide_liquidity_symmetric(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pair_address: Addr,
        lp_token: Asset,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let osmo_msg: CosmosMsg = MsgExitPool {
            sender: self.local_proxy_addr.as_ref().unwrap().to_string(),
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            share_in_amount: lp_token.amount.to_string(),
            token_out_mins: vec![], // This is fine! see: https://github.com/osmosis-labs/osmosis/blob/c51a248d67cd58e47587d6a955c3d765734eddd7/x/gamm/keeper/pool_service.go#L372
        }
        .into();

        Ok(vec![osmo_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom: match ask_asset {
                AssetInfo::Native(denom) => denom,
                _ => return Err(DexError::Cw1155Unsupported),
            },
        }];

        let token_in = Coin::try_from(offer_asset)?.to_string();

        let sim_msg = QuerySwapExactAmountInRequest {
            sender: self.local_proxy_addr.as_ref().unwrap().to_string(),
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_in,
            routes,
        };
        // .into();

        let query_request = QueryRequest::Stargate {
            path: QuerySwapExactAmountInRequest::TYPE_URL.to_string(),
            data: to_binary(&sim_msg)?,
        };
        let res = deps.querier.query(&query_request)?; // Querier is on osmosis!
        let swap_exact_amount_in_response: QuerySwapExactAmountInResponse = from_binary(&res)?;

        Ok((
            swap_exact_amount_in_response
                .token_out_amount
                .parse::<Uint128>()
                .unwrap(),
            Uint128::zero(),
            Uint128::zero(),
            false,
        ))
    }
}

fn query_pool_data(deps: Deps, pool_id: u64) -> Pool {
    let res: QueryPoolResponse = deps
        .querier
        .query(&QueryRequest::Stargate {
            path: QueryPoolRequest::TYPE_URL.to_string(),
            data: to_binary(&QueryPoolRequest { pool_id }).unwrap(),
        })
        .unwrap();
    let pool = Pool::try_from(res.pool.unwrap()).unwrap();
    pool
}

fn compute_osmo_share_out_amount(
    pool_assets: &[OsmoCoin],
    deposits: &[Uint128; 2],
    total_share: Uint128,
) -> StdResult<Uint128> {
    // let res: QueryPoolResponse = deps
    //     .querier
    //     .query(&QueryRequest::Stargate {
    //         path: QueryPoolRequest::TYPE_URL.to_string(),
    //         data: to_binary(&QueryPoolRequest { pool_id }).unwrap(),
    //     })
    //     .unwrap();

    // let pool = Pool::try_from(res.pool.unwrap()).unwrap();

    // let pool_assets: Vec<OsmoCoin> = pool
    //     .pool_assets
    //     .into_iter()
    //     .map(|asset| asset.token.unwrap())
    //     .collect();

    // let deposits: [Uint128; 2] = [
    //     offer_assets
    //         .iter()
    //         .find(|coin| coin.denom == pool_assets[0].denom)
    //         .map(|coin| coin.amount.parse::<Uint128>().unwrap())
    //         .expect("wrong asset provided"),
    //     offer_assets
    //         .iter()
    //         .find(|coin| coin.denom == pool_assets[0].denom)
    //         .map(|coin| coin.amount.parse::<Uint128>().unwrap())
    //         .expect("wrong asset provided"),
    // ];

    // let total_share = pool
    //     .total_shares
    //     .unwrap()
    //     .amount
    //     .parse::<Uint128>()
    //     .unwrap();

    // ~ source: terraswap contract ~
    // min(1, 2)
    // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (total_share / sqrt(pool_0 * pool_1))
    // == deposit_0 * total_share / pool_0
    // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (total_share / sqrt(pool_1 * pool_1))
    // == deposit_1 * total_share / pool_1
    let share_amount_out = std::cmp::min(
        deposits[0].multiply_ratio(
            total_share,
            pool_assets[0].amount.parse::<Uint128>().unwrap(),
        ),
        deposits[1].multiply_ratio(
            total_share,
            pool_assets[1].amount.parse::<Uint128>().unwrap(),
        ),
    );

    Ok(share_amount_out)
}

fn assert_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Uint128; 2],
    pool_assets: Vec<OsmoCoin>,
) -> Result<(), DexError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        if slippage_tolerance > Decimal256::one() {
            return Err(DexError::Std(
                StdError::generic_err("slippage_tolerance cannot bigger than 1").into(),
            ));
        }

        let one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
        let pools: [Uint256; 2] = [
            pool_assets[0].amount.parse::<Uint256>().unwrap(),
            pool_assets[1].amount.parse::<Uint256>().unwrap(),
        ];

        // Ensure each prices are not dropped as much as slippage tolerance rate
        if Decimal256::from_ratio(deposits[0], deposits[1]) * one_minus_slippage_tolerance
            > Decimal256::from_ratio(pools[0], pools[1])
            || Decimal256::from_ratio(deposits[1], deposits[0]) * one_minus_slippage_tolerance
                > Decimal256::from_ratio(pools[1], pools[0])
        {
            return Err(DexError::MaxSlippageAssertion(
                slippage_tolerance.to_string(),
                OSMOSIS.to_owned(),
            ));
        }
    }

    Ok(())
}
