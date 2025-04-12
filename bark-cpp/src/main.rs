use anyhow;
use bark_cpp::{create_wallet, get_balance, ConfigOpts, CreateOpts};
use bip39::Mnemonic;
use log::{debug, error, info};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with explicit debug level if environment variable isn't set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    debug!("Starting wallet application in debug mode");

    // Get home directory using environment variables
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
    let datadir = PathBuf::from(home).join(".bark");
    debug!("Using data directory: {:?}", datadir);

    fs::create_dir_all(datadir.clone()).await?;

    let mnemonic = Mnemonic::from_str("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")?;

    let config = ConfigOpts {
        asp: Some("ark.signet.2nd.dev".to_string()),
        esplora: Some("esplora.signet.2nd.dev".to_string()),
        bitcoind: None,
        bitcoind_cookie: None,
        bitcoind_user: None,
        bitcoind_pass: None,
    };
    debug!(
        "Configuration created: asp={:?}, esplora={:?}",
        config.asp, config.esplora
    );

    let opts = CreateOpts {
        force: true,
        bitcoin: false,
        signet: true,
        regtest: false,
        mnemonic,
        birthday_height: None,
        config,
    };
    debug!(
        "Create options prepared: force={}, signet={}",
        opts.force, opts.signet
    );

    debug!("Attempting to create wallet...");
    if let Err(e) = create_wallet(&datadir, opts).await {
        error!("Failed to create wallet: {}", e);
        return Err(anyhow::anyhow!("Wallet creation failed: {}", e));
    }

    info!("Wallet created successfully at {}", datadir.display());

    debug!("Retrieving wallet balance...");
    let balance = get_balance(&datadir, true).await?;
    info!("Wallet balance is {}", balance.offchain);

    debug!("Application completed successfully");
    Ok(())
}
