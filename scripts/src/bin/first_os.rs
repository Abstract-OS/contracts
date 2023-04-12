use abstract_boot::Abstract;
use abstract_core::objects::gov_type::GovernanceDetails;
use boot_core::{
    networks::{ChainInfo, NetworkInfo, NetworkKind},
    *,
};
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const KUJIRA_CHAIN: ChainInfo = ChainInfo {
    chain_id: "kujira",
    pub_address_prefix: "kujira",
    coin_type: 118u32,
};

pub const HARPOON_4: NetworkInfo = NetworkInfo {
    kind: NetworkKind::Testnet,
    id: "harpoon-4",
    gas_denom: "ukuji",
    gas_price: 0.025,
    grpc_urls: &["https://kujira-testnet-grpc.polkachu.com:11890"],
    chain_info: KUJIRA_CHAIN,
    lcd_url: None,
    fcd_url: None,
};

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Script that registers the first Account in abstract (our Account)
pub fn first_os(network: NetworkInfo) -> anyhow::Result<()> {
    let abstract_version: Version = ABSTRACT_VERSION.parse().unwrap();
    // let network = LOCAL_JUNO;
    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (sender, chain) = instantiate_daemon_env(&rt, options?)?;

    let deployment = Abstract::new(chain, abstract_version);

    // NOTE: this assumes that the deployment has been deployed

    deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        })?;

    deployment.ans_host.update_all()?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = first_os(HARPOON_4) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
