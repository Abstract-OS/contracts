use crate::contract::OsFactoryResult;
use crate::state::*;
use crate::{error::OsFactoryError, response::MsgInstantiateContractResponse};
use abstract_macros::abstract_response;
use abstract_os::{app, OS_FACTORY};
use abstract_sdk::helpers::cosmwasm_std::wasm_smart_query;
use abstract_sdk::os::version_control::{ExecuteMsg as VCExecuteMsg, QueryMsg as VCQuery};
use abstract_sdk::os::{
    manager::{ExecuteMsg::UpdateModuleAddresses, InstantiateMsg as ManagerInstantiateMsg},
    proxy::{ExecuteMsg as ProxyExecMsg, InstantiateMsg as ProxyInstantiateMsg},
};
use abstract_sdk::os::{
    objects::{gov_type::GovernanceDetails, module::ModuleInfo, module_reference::ModuleReference},
    os_factory::ExecuteMsg,
    subscription::{
        DepositHookMsg as SubDepositHook, SubscriptionExecuteMsg, SubscriptionFeeResponse,
        SubscriptionQueryMsg,
    },
    version_control::{Core, ModuleResponse},
};
use cosmwasm_std::{
    from_binary, to_binary, wasm_execute, Addr, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, ReplyOn, StdError, StdResult, SubMsg, SubMsgResult, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use protobuf::Message;

pub const CREATE_OS_MANAGER_MSG_ID: u64 = 1u64;
pub const CREATE_OS_PROXY_MSG_ID: u64 = 2u64;

use abstract_sdk::os::{MANAGER, PROXY};

#[abstract_response(OS_FACTORY)]
struct OsFactoryResponse;

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> OsFactoryResult {
    match from_binary(&cw20_msg.msg)? {
        ExecuteMsg::CreateOs {
            governance,
            description,
            link,
            name,
        } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender),
                amount: cw20_msg.amount,
            };
            execute_create_os(deps, env, governance, Some(asset), name, description, link)
        }
        _ => Err(OsFactoryError::Std(StdError::generic_err(
            "unknown send msg hook",
        ))),
    }
}

/// Function that starts the creation of the OS
pub fn execute_create_os(
    deps: DepsMut,
    env: Env,
    governance: GovernanceDetails,
    asset: Option<Asset>,
    name: String,
    description: Option<String>,
    link: Option<String>,
) -> OsFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    let mut msgs = vec![];
    if let Some(sub_addr) = &config.subscription_address {
        let subscription_fee: SubscriptionFeeResponse =
            query_subscription_fee(&deps.querier, sub_addr)?;
        if !subscription_fee.fee.amount.is_zero() {
            forward_payment(asset, &config, &mut msgs, sub_addr)?;
        }
    }
    // Get address of OS root user, depends on gov-type
    let root_user: Addr = match &governance {
        GovernanceDetails::Monarchy { monarch } => deps.api.addr_validate(monarch)?,
        GovernanceDetails::External {
            governance_address,
            governance_type: _,
        } => deps.api.addr_validate(governance_address)?,
    };

    // Query version_control for code_id of Manager contract
    let module_resp: ModuleResponse =
        query_code_id(&deps.querier, &config.version_control_contract, MANAGER)?;

    if let ModuleReference::Core(manager_code_id) = module_resp.module.reference {
        Ok(
            OsFactoryResponse::new("create_os", vec![("os_id", &config.next_os_id.to_string())])
                // Create manager
                .add_submessage(SubMsg {
                    id: CREATE_OS_MANAGER_MSG_ID,
                    gas_limit: None,
                    msg: WasmMsg::Instantiate {
                        code_id: manager_code_id,
                        funds: vec![],
                        // Currently set admin to self, update later when we know the contract's address.
                        admin: Some(env.contract.address.to_string()),
                        label: format!("Abstract OS: {}", config.next_os_id),
                        msg: to_binary(&ManagerInstantiateMsg {
                            os_id: config.next_os_id,
                            root_user: root_user.to_string(),
                            version_control_address: config.version_control_contract.to_string(),
                            subscription_address: config.subscription_address.map(Addr::into),
                            module_factory_address: config.module_factory_address.to_string(),
                            name,
                            description,
                            link,
                            governance_type: governance.to_string(),
                        })?,
                    }
                    .into(),
                    reply_on: ReplyOn::Success,
                })
                // Add as subscription registration as last. Gets called after the reply sequence is done.
                .add_messages(msgs),
        )
    } else {
        Err(OsFactoryError::WrongModuleKind(
            module_resp.module.info.to_string(),
            "app".to_string(),
        ))
    }
}

/// instantiates the Treasury contract of the newly created DAO
pub fn after_manager_create_proxy(deps: DepsMut, result: SubMsgResult) -> OsFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let manager_address = res.get_contract_address();

    CONTEXT.save(
        deps.storage,
        &Context {
            os_manager_address: deps.api.addr_validate(manager_address)?,
        },
    )?;

    // Query version_control for code_id of proxy
    let module_resp: ModuleResponse =
        query_code_id(&deps.querier, &config.version_control_contract, PROXY)?;

    if let ModuleReference::Core(proxy_code_id) = module_resp.module.reference {
        Ok(OsFactoryResponse::new(
            "create_manager",
            vec![("manager_address", manager_address.to_string())],
        )
        // Instantiate proxy contract
        .add_submessage(SubMsg {
            id: CREATE_OS_PROXY_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: proxy_code_id,
                funds: vec![],
                admin: Some(manager_address.to_string()),
                label: format!("Proxy of OS: {}", config.next_os_id),
                msg: to_binary(&ProxyInstantiateMsg {
                    os_id: config.next_os_id,
                    ans_host_address: config.ans_host_contract.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
    } else {
        Err(OsFactoryError::WrongModuleKind(
            module_resp.module.info.to_string(),
            "app".to_string(),
        ))
    }
}

fn query_code_id(
    querier: &QuerierWrapper,
    version_control_addr: &Addr,
    module_id: &str,
) -> StdResult<ModuleResponse> {
    querier.query(&wasm_smart_query(
        version_control_addr.to_string(),
        &VCQuery::Module {
            module: ModuleInfo::from_id_latest(module_id)?,
        },
    )?)
}

/// Registers the DAO on the version_control contract and
/// adds proxy contract address to Manager
pub fn after_proxy_add_to_manager_and_set_admin(
    deps: DepsMut,
    result: SubMsgResult,
) -> OsFactoryResult {
    let mut config = CONFIG.load(deps.storage)?;
    let context = CONTEXT.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let proxy_address = res.get_contract_address();

    // construct OS core
    let core = Core {
        manager: context.os_manager_address.clone(),
        proxy: deps.api.addr_validate(proxy_address)?,
    };

    // Add OS core to version_control
    let add_os_core_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddOs {
            os_id: config.next_os_id,
            core,
        })?,
    });

    // add manager to whitelisted addresses
    let whitelist_manager: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::AddModule {
            module: context.os_manager_address.to_string(),
        })?,
    });

    let set_proxy_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::SetAdmin {
            admin: context.os_manager_address.to_string(),
        })?,
    });

    let set_manager_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
        contract_addr: context.os_manager_address.to_string(),
        admin: context.os_manager_address.to_string(),
    });

    // Update id sequence
    config.next_os_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(OsFactoryResponse::new(
        "create_proxy",
        vec![("proxy_address", res.get_contract_address())],
    )
    .add_message(add_os_core_to_version_control_msg)
    .add_message(wasm_execute(
        context.os_manager_address.to_string(),
        &UpdateModuleAddresses {
            to_add: Some(vec![(PROXY.to_string(), proxy_address.to_string())]),
            to_remove: None,
        },
        vec![],
    )?)
    .add_message(whitelist_manager)
    .add_message(set_proxy_admin_msg)
    .add_message(set_manager_admin_msg))
}

// Only owner can execute it
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    ans_host_contract: Option<String>,
    version_control_contract: Option<String>,
    module_factory_address: Option<String>,
    subscription_address: Option<String>,
) -> OsFactoryResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(ans_host_contract) = ans_host_contract {
        // validate address format
        config.ans_host_contract = deps.api.addr_validate(&ans_host_contract)?;
    }

    if let Some(version_control_contract) = version_control_contract {
        // validate address format
        config.version_control_contract = deps.api.addr_validate(&version_control_contract)?;
    }

    if let Some(module_factory_address) = module_factory_address {
        // validate address format
        config.module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    }

    if let Some(subscription_address) = subscription_address {
        config.subscription_address = Some(deps.api.addr_validate(&subscription_address)?);
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(admin) = admin {
        let addr = deps.api.addr_validate(&admin)?;
        ADMIN.set(deps, Some(addr))?;
    }

    Ok(OsFactoryResponse::action("update_config"))
}

fn query_subscription_fee(
    querier: &QuerierWrapper,
    subscription_address: &Addr,
) -> StdResult<SubscriptionFeeResponse> {
    let subscription_fee_response: SubscriptionFeeResponse = querier.query(&wasm_smart_query(
        subscription_address.to_string(),
        &app::QueryMsg::App(SubscriptionQueryMsg::Fee {}),
    )?)?;
    Ok(subscription_fee_response)
}

// Does not do any payment verifications.
// This provides more flexibility on the subscription contract to handle different payment options
fn forward_payment(
    maybe_received_payment: Option<Asset>,
    config: &Config,
    msgs: &mut Vec<CosmosMsg>,
    sub_addr: &Addr,
) -> Result<(), OsFactoryError> {
    if let Some(received_payment) = maybe_received_payment {
        // Forward payment to subscription module and registers the OS
        let forward_payment_to_module: CosmosMsg<Empty> = match received_payment.info {
            AssetInfoBase::Cw20(_) => received_payment.send_msg(
                sub_addr,
                to_binary(&SubDepositHook::Pay {
                    os_id: config.next_os_id,
                })?,
            )?,
            AssetInfoBase::Native(denom) => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: sub_addr.into(),
                msg: to_binary::<app::ExecuteMsg<SubscriptionExecuteMsg>>(&app::ExecuteMsg::App(
                    SubscriptionExecuteMsg::Pay {
                        os_id: config.next_os_id,
                    },
                ))?,
                funds: vec![Coin::new(received_payment.amount.u128(), denom)],
            }),
            AssetInfoBase::Cw1155(_, _) => {
                return Err(OsFactoryError::Std(StdError::generic_err(
                    "Cw1155 not supported.",
                )))
            }
            _ => panic!("unsupported asset"),
        };

        msgs.push(forward_payment_to_module);
        Ok(())
    } else {
        Err(OsFactoryError::NoPaymentReceived {})
    }
}
