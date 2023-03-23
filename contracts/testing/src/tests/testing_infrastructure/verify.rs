use super::common_integration::NativeContracts;
use abstract_sdk::interfaces::version_control::{AccountBaseResponse, Core};
use abstract_sdk::interfaces::*;
use cw_multi_test::App;
use std::collections::HashMap;

pub fn os_store_as_expected(
    app: &App,
    native_contracts: &NativeContracts,
    os_store: &HashMap<u32, Core>,
) -> bool {
    let resp: account_factory::ConfigResponse = app
        .wrap()
        .query_wasm_smart(
            &native_contracts.account_factory,
            &account_factory::QueryMsg::Config {},
        )
        .unwrap();
    let max_acct_id = resp.next_acct_id - 1;

    for account_id in 0..max_acct_id {
        // Check OS
        let core: AccountBaseResponse = app
            .wrap()
            .query_wasm_smart(
                &native_contracts.version_control,
                &version_control::QueryMsg::OsCore { account_id },
            )
            .unwrap();
        if core.account.ne(os_store.get(&account_id).unwrap()) {
            return false;
        }
    }
    true
}
