use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_os::{
    api::{ApiConfigResponse, ExecuteMsg, TradersResponse},
    dex::{DexRequestMsg, SimulateSwapResponse},
};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use osmosis_host::contract::OsmoHost;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    OsmoHost::export_schema(&out_dir);

    export_schema_with_title(&schema_for!(SimulateSwapResponse), &out_dir, "ApiResponse");

    export_schema(&schema_for!(ExecuteMsg<DexRequestMsg>), &out_dir);
    export_schema_with_title(&schema_for!(TradersResponse), &out_dir, "TradersResponse");
    export_schema_with_title(&schema_for!(ApiConfigResponse), &out_dir, "ConfigResponse");

    // export_schema_with_title(&schema_for!(ApiQueryMsg), &out_dir, "QueryMsg");
}
