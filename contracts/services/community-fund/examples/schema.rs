use std::env::current_dir;
use std::fs::create_dir_all;

use community_fund::msg::InstantiateMsg;
use community_fund::state::State;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use pandora::community_fund::msg::{ConfigResponse, ExecuteMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
}
