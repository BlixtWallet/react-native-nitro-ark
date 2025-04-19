use anyhow;
use anyhow::bail;
use bark;
use bark::ark::bitcoin::address;
use bark::ark::bitcoin::Address;
use bark::ark::bitcoin::Amount;
use bark::ark::bitcoin::Network;
use bark::ark::bitcoin::Txid;
use bark::ark::ArkInfo;
use bark::Config;
use bark::SqliteClient;
use bark::Wallet;
mod ffi;
mod utils;
use bip39::Mnemonic;
use logger::log::{debug, info, warn};
use std::fs;
use std::path::Path;
use utils::try_create_wallet;
use utils::DB_FILE;
const MNEMONIC_FILE: &str = "mnemonic";

pub use utils::*;

use std::str::FromStr;

use anyhow::Context;

pub fn create_mnemonic() -> anyhow::Result<String> {
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
pub async fn get_balance(
    datadir: &Path,
    no_sync: bool,
    mnemonic: Mnemonic,
) -> anyhow::Result<Balance> {
    let mut w = open_wallet(&datadir, mnemonic)
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

pub async fn open_wallet(
    datadir: &Path,
    mnemonic: Mnemonic,
) -> anyhow::Result<Wallet<SqliteClient>> {
    debug!("Opening bark wallet in {}", datadir.display());

    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::open(&mnemonic, db).await
}

pub async fn get_ark_info(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<ArkInfo> {
    let w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet in get_ark_info")?;

    let info = w.ark_info();

    if let Some(info) = info {
        Ok(info.clone())
    } else {
        bail!("Failed to get ark info")
    }
}

/// Get an onchain address from the wallet
pub async fn get_onchain_address(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<Address> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for get_onchain_address")?;

    // Wallet::address() returns Result<Address, Error>
    let address = w
        .onchain
        .address()
        .context("Wallet failed to generate address")?;

    Ok(address)
}

/// Send funds using the onchain wallet
pub async fn send_onchain(
    datadir: &Path,
    mnemonic: Mnemonic,
    destination_str: &str, // Take string to handle validation here
    amount: Amount,
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for send_onchain")?;

    let net = w.properties()?.network;

    // Parse the address first without network requirement
    let address_unchecked = Address::<address::NetworkUnchecked>::from_str(destination_str)
        .with_context(|| format!("invalid destination address format: '{}'", destination_str))?;

    // Now require the network to match the wallet's network
    let destination_address = address_unchecked.require_network(net).with_context(|| {
        format!(
            "address '{}' is not valid for configured network {}",
            destination_str, net
        )
    })?;

    if !no_sync {
        info!("Syncing onchain wallet before sending...");
        // Sync only the onchain part as we are doing an onchain send
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during send: {}", e);
            // Decide if this should be a hard error or just a warning like the CLI
            // Let's treat it as a warning for now, but return error might be safer
            // return Err(e).context("Failed to sync onchain wallet before send");
        }
    }

    info!(
        "Sending {} to onchain address {}",
        amount, destination_address
    );
    let txid = w
        .onchain
        .send(destination_address.clone(), amount)
        .await
        .with_context(|| format!("failed to send {} to {}", amount, destination_address))?;

    info!("Onchain send successful, TxID: {}", txid);
    Ok(txid)
}
