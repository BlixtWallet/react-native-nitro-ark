use anyhow;
use bark_cpp::{create_wallet, get_balance, ConfigOpts, CreateOpts};
use logger::log::{error, info};
use logger::Logger;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _logger = Logger::new();

    // Get home directory using environment variables
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
    let datadir = PathBuf::from(home).join(".bark");

    let config = ConfigOpts {
        asp: Some("ark.signet.2nd.dev".to_string()),
        esplora: Some("esplora.signet.2nd.dev".to_string()),
        bitcoind: None,
        bitcoind_cookie: None,
        bitcoind_user: None,
        bitcoind_pass: None,
    };

    let opts = CreateOpts {
        force: true,
        bitcoin: false,
        signet: true,
        regtest: false,
        mnemonic: None,
        birthday_height: None,
        config,
    };

    if let Err(e) = create_wallet(&datadir, opts).await {
        error!("Failed to create wallet: {}", e);
        // return Err(e);
    }

    println!("Wallet created successfully at {}", datadir.display());

    let balance = get_balance(&datadir, true).await?;

    println!("Wallet balance is {}", balance.offchain);

    Ok(())
}
