use abstract_boot::Abstract;
use abstract_core::objects::gov_type::GovernanceDetails;

use boot_core::{
    networks::{parse_network, NetworkInfo},
    *,
};
use clap::Parser;
use semver::Version;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy(network: NetworkInfo) -> anyhow::Result<()> {
    let abstract_version: Version = ABSTRACT_VERSION.parse().unwrap();

    let rt = Arc::new(Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(network).build();
    let (sender, chain) = instantiate_daemon_env(&rt, options?)?;
    let deployment = Abstract::deploy_on(chain, abstract_version)?;

    // CReate the Abstract Account because it's needed for the fees for the dex module
    deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        })?;

    // let _dex = DexApi::new("dex", chain);

    // deployment.deploy_modules()?;

    let ans_host = deployment.ans_host;
    ans_host.update_all()?;

    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id);

    if let Err(ref err) = full_deploy(network) {
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
