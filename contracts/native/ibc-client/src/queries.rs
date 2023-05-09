use abstract_core::{
    ibc_client::{
        state::{Config, ACCOUNTS, ADMIN, CHANNELS, CONFIG},
        AccountResponse, ConfigResponse, LatestQueryResponse, ListAccountsResponse,
        ListChannelsResponse,
    },
    objects::{chain_name::ChainName, AccountId},
};
use cosmwasm_std::{Deps, Env, Order, StdResult};

// TODO: paging
pub fn list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<
        Vec<(
            AccountId, abstract_core::objects::chain_name::ChainName,
            String,
        )>> = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|((a,c), s)| (a,c,s)))
        .collect();

    Ok(ListAccountsResponse { accounts: accounts? })
}

pub fn list_channels(deps: Deps) -> StdResult<ListChannelsResponse> {
    let channels = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListChannelsResponse { channels })
}

pub fn config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let Config {
        version_control_address,
    } = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    let chain = ChainName::new(&env);
    Ok(ConfigResponse {
        admin: admin.into(),
        version_control_address: version_control_address.into_string(),
        chain: chain.into_string(),
    })
}

pub fn account(
    deps: Deps,
    host_chain: String,
    account_id: AccountId,
) -> StdResult<AccountResponse> {
    let host_chain = ChainName::from(host_chain);
    host_chain.check().unwrap();
    let remote_proxy_addr = ACCOUNTS.load(deps.storage, (&account_id, &host_chain))?;
    Ok(AccountResponse { remote_proxy_addr })
}
