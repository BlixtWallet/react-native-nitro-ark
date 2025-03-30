use anyhow;
use bark_cpp::{create_wallet, ConfigOpts, CreateOpts};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        force: false,
        bitcoin: false,
        signet: true,
        regtest: false,
        mnemonic: None,
        birthday_height: None,
        config,
    };

    create_wallet(&datadir, opts).await?;
    println!("Wallet created successfully at {}", datadir.display());

    Ok(())
}
