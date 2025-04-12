use anyhow;
use anyhow::bail;
use bark;
use bark::ark::bitcoin::Network;
use bark::Config;
use bark::SqliteClient;
use bark::Wallet;
mod ffi;
mod utils;
use logger::log::{debug, info, warn};
use std::fs;
use std::path::Path;
use utils::try_create_wallet;
use utils::DB_FILE;
const MNEMONIC_FILE: &str = "mnemonic";

pub use utils::*;

use std::str::FromStr;

use anyhow::Context;

pub async fn create_mnemonic() -> anyhow::Result<String> {
    let mnemonic = bip39::Mnemonic::generate(12).context("failed to generate mnemonic")?;
    Ok(mnemonic.to_string())
}

pub async fn create_wallet(datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
    debug!("Creating wallet in {}", datadir.display());

    let net = match (opts.bitcoin, opts.signet, opts.regtest) {
        (true, false, false) => Network::Bitcoin,
        (false, true, false) => Network::Signet,
        (false, false, true) => Network::Regtest,
        _ => bail!("A network must be specified. Use either --signet, --regtest or --bitcoin"),
    };

    let mut config = Config {
        // required args
        asp_address: opts
            .config
            .asp
            .clone()
            .context("ASP address missing, use --asp")?,
        ..Default::default()
    };
    opts.config
        .merge_into(&mut config)
        .context("invalid configuration")?;

    // check if dir doesn't exists, then create it
    if datadir.exists() {
        if opts.force {
            fs::remove_dir_all(datadir)?;
        } else {
            bail!("Directory {} already exists", datadir.display());
        }
    }

    info!("Attempting to open database...");

    try_create_wallet(&datadir, net, config, opts.mnemonic, opts.birthday_height).await?;

    Ok(())
}

pub struct Balance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64,
}

/// Get offchain and onchain balances
pub async fn get_balance(datadir: &Path, no_sync: bool) -> anyhow::Result<Balance> {
    let mut w = open_wallet(&datadir)
        .await
        .context("error opening wallet")?;

    if !no_sync {
        info!("Syncing wallet...");
        if let Err(e) = w.sync().await {
            warn!("Sync error: {}", e)
        }
    }

    let onchain = w.onchain.balance().to_sat();
    let offchain = w.offchain_balance().await?.to_sat();
    let pending_exit = w.exit.pending_total().await?.to_sat();

    let balances = Balance {
        onchain,
        offchain,
        pending_exit,
    };
    Ok(balances)
}

pub async fn open_wallet(datadir: &Path) -> anyhow::Result<Wallet<SqliteClient>> {
    debug!("Opening bark wallet in {}", datadir.display());

    // read mnemonic file
    let mnemonic_path = datadir.join(MNEMONIC_FILE);
    let mnemonic_str = fs::read_to_string(&mnemonic_path).with_context(|| {
        format!(
            "failed to read mnemonic file at {}",
            mnemonic_path.display()
        )
    })?;
    let mnemonic = bip39::Mnemonic::from_str(&mnemonic_str).context("broken mnemonic")?;

    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::open(&mnemonic, db).await
}
