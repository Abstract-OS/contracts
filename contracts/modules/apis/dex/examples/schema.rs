use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_os::{
    api::{ExecuteMsg, QueryApiConfigResponse, QueryMsg as ApiQueryMsg, QueryTradersResponse},
    dex::{QueryMsg, RequestMsg, SimulateSwapResponse},
};
use cosmwasm_schema::{
    export_schema, export_schema_with_title, remove_schemas, schema_for, write_api,
};
use cosmwasm_std::Empty;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: ApiQueryMsg<QueryMsg>,
        execute: ExecuteMsg<RequestMsg>,
        sudo: Empty ,
        migrate: MigrateMsg,
    }
}
