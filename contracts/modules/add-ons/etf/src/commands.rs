use abstract_add_on::state::AddOnState;

use abstract_sdk::AnsHostOperation;
use cosmwasm_std::{
    from_binary, to_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo};

use abstract_os::etf::DepositHookMsg;
use abstract_os::objects::deposit_info::DepositInfo;
use abstract_os::objects::fee::Fee;
use abstract_sdk::cw20::query_supply;
use abstract_sdk::proxy::{os_module_action, query_enabled_asset_names, query_total_value};

use crate::contract::{EtfAddOn, EtfResult};
use crate::error::VaultError;
use crate::state::{State, FEE, STATE};

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    dapp: EtfAddOn,
    cw20_msg: Cw20ReceiveMsg,
) -> EtfResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::WithdrawLiquidity {} => {
            let state: State = STATE.load(deps.storage)?;
            if msg_info.sender != state.liquidity_token_addr {
                return Err(VaultError::NotLPToken {
                    token: msg_info.sender.to_string(),
                });
            }
            try_withdraw_liquidity(deps, env, dapp, cw20_msg.sender, cw20_msg.amount)
        }
        DepositHookMsg::ProvideLiquidity {} => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            try_provide_liquidity(deps, msg_info, dapp, asset, Some(cw20_msg.sender))
        }
    }
}

/// Called when either providing liquidity with a native token or when providing liquidity
/// with a CW20.
pub fn try_provide_liquidity(
    deps: DepsMut,
    msg_info: MessageInfo,
    dapp: EtfAddOn,
    asset: Asset,
    sender: Option<String>,
) -> EtfResult {
    // Load all needed states
    let base_state = dapp.load_state(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    // Get the liquidity provider address
    let liq_provider = match sender {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => {
            // Check if deposit matches claimed deposit.
            match asset.info {
                AssetInfo::Native(..) => {
                    // If native token, assert claimed amount is correct
                    let coin = msg_info.funds.last();
                    if coin.is_none() {
                        return Err(VaultError::WrongNative {});
                    }

                    let coin = coin.unwrap().clone();
                    if Asset::native(coin.denom, coin.amount) != asset {
                        return Err(VaultError::WrongNative {});
                    }
                    msg_info.sender
                }
                AssetInfo::Cw20(_) => return Err(VaultError::NotUsingCW20Hook {}),
                AssetInfo::Cw1155(_, _) => return Err(VaultError::NotUsingCW20Hook {}),
                _ => panic!("unsupported asset"),
            }
        }
    };

    // Get all the required asset information from the ans_host contract
    let (_, base_asset) = query_enabled_asset_names(deps.as_ref(), &base_state.proxy_address)?;
    let deposit_asset = dapp.resolve(deps.as_ref(), &base_asset)?;
    // Construct deposit info
    let deposit_info = DepositInfo {
        asset_info: deposit_asset,
    };

    // Assert deposited asset and claimed asset infos are the same
    deposit_info.assert(&asset.info)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to vault")),
        ("Received funds:", asset.to_string()),
    ];

    // Received deposit to vault
    let deposit: Uint128 = asset.amount;

    // Get total value in Vault
    let value = query_total_value(deps.as_ref(), &base_state.proxy_address)?;
    // Get total supply of LP tokens and calculate share
    let total_share = query_supply(&deps.querier, state.liquidity_token_addr.clone())?;

    let share = if total_share == Uint128::zero() || value.is_zero() {
        // Initial share = deposit amount
        deposit
    } else {
        // lt: liquidity token
        // lt_to_receive = deposit * lt_price
        // lt_to_receive = deposit * lt_supply / previous_total_vault_value )
        // lt_to_receive = deposit * ( lt_supply / ( current_total_vault_value - deposit ) )
        let value_increase = Decimal::from_ratio(value + deposit, value);
        (total_share * value_increase) - total_share
    };

    // mint LP token to liq_provider
    let mint_lp = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.liquidity_token_addr.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: liq_provider.to_string(),
            amount: share,
        })?,
        funds: vec![],
    });

    // Send received asset to the vault.
    let send_to_vault = asset.transfer_msg(base_state.proxy_address)?;

    let response = Response::new()
        .add_attributes(attrs)
        .add_message(mint_lp)
        .add_message(send_to_vault);

    Ok(response)
}

/// Attempt to withdraw deposits. Fees are calculated and deducted in liquidity tokens.
/// This allows the war-chest to accumulate a stake in the vault.
/// The refund is taken out of Anchor if possible.
/// Luna holdings are not eligible for withdrawal.
pub fn try_withdraw_liquidity(
    deps: DepsMut,
    _env: Env,
    dapp: EtfAddOn,
    sender: String,
    amount: Uint128,
) -> EtfResult {
    let state: State = STATE.load(deps.storage)?;
    let base_state: AddOnState = dapp.load_state(deps.storage)?;
    let ans_host = base_state.ans_host;
    let fee: Fee = FEE.load(deps.storage)?;
    // Get assets
    let (assets, _) = query_enabled_asset_names(deps.as_ref(), &base_state.proxy_address)?;
    let assets = ans_host.query_assets(deps.as_ref(), assets)?;

    // Logging var
    let mut attrs = vec![
        ("Action:", String::from("Withdraw from vault")),
        ("Received liquidity tokens:", amount.to_string()),
    ];

    // Calculate share of pool and requested pool value
    let total_share: Uint128 = query_supply(&deps.querier, state.liquidity_token_addr.clone())?;

    // Get provider fee in LP tokens
    let provider_fee = fee.compute(amount);

    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - provider_fee, total_share);

    // Init response
    let mut response = Response::new();

    if !provider_fee.is_zero() {
        // LP token fee
        let lp_token_provider_fee = Asset {
            info: AssetInfo::Cw20(state.liquidity_token_addr.clone()),
            amount: provider_fee,
        };

        // Construct provider fee msg
        let provider_fee_msg = fee.msg(lp_token_provider_fee, state.provider_addr.clone())?;

        // Transfer fee
        response = response.add_message(provider_fee_msg);
    }
    attrs.push(("Treasury fee:", provider_fee.to_string()));

    // Get asset holdings of vault and calculate amount to return
    let mut pay_back_assets: Vec<Asset> = vec![];
    // Get asset holdings of vault and calculate amount to return
    for (_, info) in assets.into_iter() {
        pay_back_assets.push(Asset {
            info: info.clone(),
            amount: share_ratio
                // query asset held in proxy
                * info.query_balance(&deps.querier,
                                     base_state.proxy_address.clone(),
            )
                ?,
        });
    }

    // Construct repay msgs
    let mut refund_msgs: Vec<CosmosMsg> = vec![];
    for asset in pay_back_assets.into_iter() {
        if asset.amount != Uint128::zero() {
            // Unchecked ok as sender is already validated by VM
            refund_msgs.push(
                asset
                    .clone()
                    .transfer_msg(Addr::unchecked(sender.clone()))?,
            );
            attrs.push(("Repaying:", asset.to_string()));
        }
    }

    // Msg that gets called on the vault address
    let vault_refund_msg = os_module_action(refund_msgs, &base_state.proxy_address)?;

    // LP burn msg
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.liquidity_token_addr.into(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - provider_fee),
        })?,
        funds: vec![],
    });

    Ok(response
        .add_attribute("Action:", "Withdraw Liquidity")
        // Burn LP tokens
        .add_message(burn_msg)
        // Send proxy funds to owner
        .add_message(vault_refund_msg)
        .add_attributes(attrs))
}

pub fn set_fee(
    deps: DepsMut,
    msg_info: MessageInfo,
    dapp: EtfAddOn,
    new_fee: Decimal,
) -> EtfResult {
    // Only the admin should be able to call this
    dapp.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let fee = Fee::new(new_fee)?;

    FEE.save(deps.storage, &fee)?;
    Ok(Response::new().add_attribute("Update:", "Successful"))
}
