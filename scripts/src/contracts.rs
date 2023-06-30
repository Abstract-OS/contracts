use crate::EntryDif;
use cw_orch::prelude::*;

use abstract_core::ans_host::*;
use abstract_core::objects::UncheckedContractEntry;
use abstract_interface::{AbstractInterfaceError, AnsHost};

use serde_json::Value;
use std::collections::HashMap;

pub fn get_scraped_entries(
    chain_name: &String,
    chain_id: &String,
) -> Result<HashMap<UncheckedContractEntry, String>, AbstractInterfaceError> {
    let raw_scraped_entries = crate::get_scraped_json_data("contracts");

    let binding = raw_scraped_entries[chain_name][chain_id].clone();
    let parsed_scraped_entries: &Vec<Value> = binding.as_array().unwrap();

    let scraped_entries_vec: Vec<(UncheckedContractEntry, String)> = parsed_scraped_entries
        .iter()
        .map(|value| {
            let contract: (UncheckedContractEntry, String) =
                serde_json::from_value(value.clone()).unwrap();
            contract
        })
        .collect();

    Ok(scraped_entries_vec.into_iter().collect())
}

pub fn get_on_chain_entries(
    ans_host: &AnsHost<Daemon>,
) -> Result<HashMap<UncheckedContractEntry, String>, AbstractInterfaceError> {
    let mut on_chain_entries = HashMap::new();
    let mut last_asset = None;
    loop {
        let ContractListResponse { contracts } = ans_host.contract_list(None, None, last_asset)?;
        if contracts.is_empty() {
            break;
        }
        last_asset = contracts.last().map(|l| l.0.clone());
        on_chain_entries.extend(
            contracts
                .into_iter()
                .map(|(a, b)| (a.into(), b.to_string())),
        );
    }

    Ok(on_chain_entries)
}

pub fn update(
    ans_host: &AnsHost<Daemon>,
    diff: EntryDif<UncheckedContractEntry, String>,
) -> Result<(), AbstractInterfaceError> {
    
    let to_add: Vec<_> = diff.1.into_iter().collect();
    let to_remove: Vec<_> = diff.0.into_iter().collect();

    // add the contracts
    ans_host.execute_chunked(&to_add, 25, |chunk| ExecuteMsg::UpdateContractAddresses {
        to_add: chunk.to_vec(),
        to_remove: vec![],
    })?;

    // remove the contracts
    ans_host.execute_chunked(&to_remove, 25, |chunk| {
        ExecuteMsg::UpdateContractAddresses {
            to_add: vec![],
            to_remove: chunk.to_vec(),
        }
    })?;

    Ok(())
}

// fn update_channels(ans: &AnsHost<Daemon>) -> Result<(), crate::CwOrchError> {
//     let path = env::var("ANS_HOST_CHANNELS").unwrap();
//     let file =
//         File::open(&path).unwrap_or_else(|_| panic!("file should be present at {}", &path));
//     let json: serde_json::Value = from_reader(file)?;
//     let chain_name = &ans.get_chain().state().chain_data.chain_name;
//     let chain_id = ans.get_chain().state().chain_data.chain_id.to_string();
//     let channels = json
//         .get(chain_name)
//         .unwrap()
//         .get(chain_id)
//         .ok_or_else(|| CwOrchError::StdErr("network not found".into()))?;

//     let channels = channels.as_object().unwrap();
//     let channels_to_add: Vec<(UncheckedChannelEntry, String)> = channels
//         .iter()
//         .map(|(name, value)| {
//             let id = value.as_str().unwrap().to_owned();
//             let key = UncheckedChannelEntry::try_from(name.clone()).unwrap();
//             (key, id)
//         })
//         .collect();

//     ans.execute_chunked(&channels_to_add, 25, |chunk| ExecuteMsg::UpdateChannels {
//         to_add: chunk.to_vec(),
//         to_remove: vec![],
//     })?;

//     Ok(())
// }
