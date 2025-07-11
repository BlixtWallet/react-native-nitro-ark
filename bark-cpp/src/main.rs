use anyhow;
use bark::ark::bitcoin::FeeRate;
use bark_cpp::{
    bolt11_invoice, create_mnemonic, get_ark_info, get_balance, init_logger, load_wallet,
    ConfigOpts, CreateOpts,
};
use bip39::Mnemonic;

use logger::log::{debug, error, info};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with explicit debug level if environment variable isn't set
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    init_logger();

    debug!("Starting wallet application in debug mode");

    // Get home directory using environment variables
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
    let datadir = PathBuf::from(home).join(".bark");
    debug!("Using data directory: {:?}", datadir);

    // fs::create_dir_all(datadir.clone()).await?;

    let mnemonic = Mnemonic::from_str(create_mnemonic().unwrap().as_str())?;

    let config = ConfigOpts {
        asp: Some("http://127.0.0.1:3535".to_string()),
        esplora: None,
        bitcoind: Some("http://127.0.0.1:18443".to_string()),
        bitcoind_cookie: None,
        bitcoind_user: Some("second".to_string()),
        bitcoind_pass: Some("ark".to_string()),
        fallback_fee_rate: Some(FeeRate::from_sat_per_kwu(100000)),
        vtxo_refresh_expiry_threshold: 288,
    };

    // let config = ConfigOpts {
    //     asp: Some("ark.signet.2nd.dev".to_string()),
    //     esplora: Some("esplora.signet.2nd.dev".to_string()),
    //     bitcoind: None,
    //     bitcoind_cookie: None,
    //     bitcoind_user: None,
    //     bitcoind_pass: None,
    //     fallback_fee_rate: None,
    //     vtxo_refresh_expiry_threshold: 288,
    // };

    debug!(
        "Configuration created: asp={:?}, esplora={:?}",
        config.asp, config.esplora
    );

    let opts = CreateOpts {
        bitcoin: false,
        signet: false,
        regtest: true,
        mnemonic: mnemonic.clone(),
        birthday_height: None,
        config,
    };
    debug!("Create options prepared: signet={}", opts.signet);

    debug!("Attempting to create wallet...");
    if let Err(e) = load_wallet(&datadir, opts).await {
        error!("Failed to load wallet: {}", e);
        return Err(anyhow::anyhow!("Wallet loading failed: {}", e));
    }

    info!("Wallet loaded successfully from {}", datadir.display());

    debug!("Retrieving wallet balance...");
    let balance = get_balance(true).await?;
    info!("Wallet balance is {}", balance.offchain);

    let info = get_ark_info().await?;
    info!("Wallet info is {:?}", info);

    let invoice = bolt11_invoice(100000).await?;
    info!("Invoice created: {}", invoice);

    debug!("Application completed successfully");
    Ok(())
}
