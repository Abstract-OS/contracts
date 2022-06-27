use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

use abstract_os::{native::version_control::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CodeIdResponse, ConfigResponse, OsAddrResponse}, core::{manager::msg::EnabledModulesResponse, modules::ModuleInfo}};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ModuleInfo), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    // export_schema(&schema_for!(EnabledModulesResponse), &out_dir);
    export_schema_with_title(
        &schema_for!(EnabledModulesResponse),
        &out_dir,
        "QueryEnabledModulesResponse",
    );
    export_schema_with_title(&schema_for!(CodeIdResponse), &out_dir, "QueryCodeIdResponse");
    export_schema_with_title(&schema_for!(OsAddrResponse), &out_dir, "QueryOsAddrResponse");
}
