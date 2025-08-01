use anyhow;
use anyhow::bail;
use anyhow::Ok;
use bark;

use bark::ark::bitcoin::Address;
use bark::ark::bitcoin::Amount;
use bark::ark::bitcoin::Network;
use bark::ark::lightning::Preimage;
use bark::ark::rounds::RoundId;
use bark::ark::ArkInfo;
use bark::ark::Vtxo;
use bark::ark::VtxoId;
use bark::lightning_invoice::Bolt11Invoice;
use bark::lnurllib::lightning_address::LightningAddress;
use bark::onchain::OnchainWallet;
use bark::Config;
use bark::Offboard;
use bark::SendOnchain;
use bark::SqliteClient;
use bark::Wallet;
use bdk_wallet::bitcoin::key::Keypair;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
mod cxx;
mod onchain;
mod utils;

use bip39::Mnemonic;
use logger::log::{debug, info};
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Once;
use utils::try_create_wallet;
use utils::DB_FILE;

pub use utils::*;

use std::str::FromStr;

use anyhow::Context;
#[cfg(test)]
mod tests;
// Use a static Once to ensure the logger is initialized only once.
static LOGGER_INIT: Once = Once::new();
static GLOBAL_WALLET: LazyLock<Mutex<Option<(Wallet, OnchainWallet)>>> =
    LazyLock::new(|| Mutex::new(None));
pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

// function to explicitly initialize the logger.
// This should be called once from your FFI entry point.
pub fn init_logger() {
    LOGGER_INIT.call_once(|| {
        // The logger::Logger::new() function now handles the platform-specific
        // setup and initialization.
        logger::Logger::new();
    });
}

pub fn create_mnemonic() -> anyhow::Result<String> {
    info!("Attempting to create a new mnemonic using cxx bridge...");
    let mnemonic = Mnemonic::generate(12).context("failed to generate mnemonic")?;
    info!("Successfully created a new mnemonic using cxx bridge.");
    Ok(mnemonic.to_string())
}

pub async fn load_wallet(datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
    debug!("Loading wallet in {}", datadir.display());
    let mut wallet_guard = GLOBAL_WALLET.lock().await;

    if wallet_guard.is_some() {
        bail!("Wallet is already loaded. Please close it first.");
    }

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
    if !datadir.exists() {
        info!("Wallet directory does not exist, creating it...");
        try_create_wallet(datadir, net, config, Some(opts.mnemonic.clone())).await?;
    }

    info!("Attempting to open wallet...");
    let (wallet, onchain_wallet) = open_wallet(datadir, opts.mnemonic).await?;
    *wallet_guard = Some((wallet, onchain_wallet));

    Ok(())
}

pub async fn close_wallet() -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    if wallet_guard.is_none() {
        bail!("No wallet is currently loaded.");
    }
    *wallet_guard = None;
    info!("Wallet closed successfully.");
    Ok(())
}

/// Check if a wallet is loaded
pub async fn is_wallet_loaded() -> bool {
    let wallet_guard = GLOBAL_WALLET.lock().await;
    wallet_guard.is_some()
}

/// Change the wallet config
pub async fn persist_config(config: Config) -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    // Persist the config to the wallet
    w.set_config(config);

    w.persist_config()
        .context("Failed to persist wallet config to disk")?;

    info!("Wallet configuration updated successfully.");
    Ok(())
}

pub struct Balance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64,
}

pub async fn balance() -> anyhow::Result<bark::Balance> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    Ok(w.balance()?)
}

pub async fn open_wallet(
    datadir: &Path,
    mnemonic: Mnemonic,
) -> anyhow::Result<(Wallet, OnchainWallet)> {
    debug!("Opening bark wallet in {}", datadir.display());

    let db = Arc::new(SqliteClient::open(datadir.join(DB_FILE))?);

    let wallet = Wallet::open(&mnemonic, db.clone()).await?;
    let onchain_wallet = OnchainWallet::load_or_create(
        wallet.properties().unwrap().network,
        mnemonic.to_seed(""),
        db,
    )?;

    Ok((wallet, onchain_wallet))
}

pub async fn get_ark_info() -> anyhow::Result<ArkInfo> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    let info = w.ark_info();

    if let Some(info) = info {
        Ok(info.clone())
    } else {
        bail!("Failed to get ark info")
    }
}

/// Derive the next keypair for the VTXO store
pub async fn derive_store_next_keypair() -> anyhow::Result<Keypair> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    Ok(w.derive_store_next_keypair()?)
}

/// Peak the keypair at the specified index
pub async fn peak_keypair(index: u32) -> anyhow::Result<Keypair> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    Ok(w.peak_keypair(index).context("Failed to peak keypair")?)
}

/// Get a Bolt 11 invoice
pub async fn bolt11_invoice(amount: u64) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    let invoice = w
        .bolt11_invoice(Amount::from_sat(amount))
        .await
        .context("Failed to create bolt11_invoice")?;
    Ok(invoice.to_string())
}

/// Claim a lightning payment
pub async fn finish_lightning_receive(bolt11: Bolt11Invoice) -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    let _ = w
        .finish_lightning_receive(bolt11)
        .await
        .context("Failed to claim bolt11 payment")?;

    Ok(())
}

/// Performs maintenance tasks on the wallet
///
/// This tasks include onchain-sync, off-chain sync,
/// registering onboard with the server.
///
/// This tasks will only include anything that has to wait
/// for a round. The maintenance call cannot be used to
/// refresh VTXOs.
pub async fn maintenance() -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, onchain_w) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.maintenance(onchain_w)
        .await
        .context("Failed to perform wallet maintenance")?;
    Ok(())
}

/// Sync both the onchain and offchain wallet.
pub async fn sync() -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.sync().await.context("Failed to sync wallet")?;
    Ok(())
}

/// Get the list of VTXOs from the wallet as a JSON string
pub async fn get_vtxos() -> anyhow::Result<Vec<Vtxo>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    Ok(w.vtxos()?)
}

/// Refresh VTXOs based on specified criteria. Returns RoundId status.
pub async fn refresh_vtxos(vtxos: Vec<Vtxo>) -> anyhow::Result<Option<RoundId>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.refresh_vtxos(vtxos)
        .await
        .context("Failed to refresh vtxos")
}

/// Board a specific amount from the onchain wallet to Ark. Returns JSON status.
pub async fn board_amount(amount: Amount) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, onchain_w) = wallet_guard.as_mut().context("Wallet not loaded")?;

    info!("Attempting to board amount: {}", amount);
    let board_result = w.board_amount(onchain_w, amount).await?;

    let json_string = serde_json::to_string_pretty(&board_result)
        .context("Failed to serialize board status to JSON")?;

    Ok(json_string)
}

pub async fn board_all() -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, onchain_w) = wallet_guard.as_mut().context("Wallet not loaded")?;

    info!("Attempting to board all onchain funds...");
    let board_result = w.board_all(onchain_w).await?;

    let json_string = serde_json::to_string_pretty(&board_result)
        .context("Failed to serialize board status to JSON")?;

    Ok(json_string)
}

/// Fetch new rounds from the Ark Server and check if one of their VTXOs
/// is in the provided set of public keys
pub async fn sync_rounds() -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.sync_rounds().await.context("Failed to sync rounds")?;

    Ok(())
}

/// Send a payment based on destination type. Returns Vec<Vtxos> status.
pub async fn send_arkoor_payment(
    destination: bark::ark::Address,
    amount_sat: Amount,
) -> anyhow::Result<Vec<Vtxo>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    info!(
        "Attempting to send OOR payment of {} to pubkey {:?}",
        amount_sat, destination
    );
    let oor_result = w.send_arkoor_payment(&destination, amount_sat).await?;

    Ok(oor_result)
}

/// Send bolt11 payment via Ark. Returns preimage.
pub async fn send_lightning_payment(
    destination: Bolt11Invoice,
    amount_sat: Option<Amount>,
) -> anyhow::Result<Preimage> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.send_lightning_payment(&destination, amount_sat).await
}

/// Send an onchain payment via an Ark round. Returns JSON status.
pub async fn send_round_onchain_payment(
    addr: Address,
    amount: Amount,
) -> anyhow::Result<SendOnchain> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    Ok(w.send_round_onchain_payment(addr, amount).await?)
}

/// Send a Lightning Address payment. Returns a tuple containing the Bolt11Invoice and a 32-byte preimage.
pub async fn send_lnaddr(
    addr: &str,
    amount: Amount,
    comment: Option<&str>,
) -> anyhow::Result<(Bolt11Invoice, Preimage)> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    let lightning_address = LightningAddress::from_str(addr)
        .with_context(|| format!("Invalid Lightning Address format: '{}'", addr))?;

    w.send_lnaddr(&lightning_address, amount, comment).await
}

/// Offboard specific VTXOs. Returns JSON result.
pub async fn offboard_specific(
    vtxo_ids: Vec<VtxoId>,
    address: Address, // Optional destination address string
) -> anyhow::Result<Offboard> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.offboard_vtxos(vtxo_ids, address).await
}

/// Offboard all VTXOs. Returns RoundId result.
pub async fn offboard_all(
    address: Address, // Optional destination address string
) -> anyhow::Result<Offboard> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.offboard_all(address).await
}

/// Sync status of unilateral exits.
pub async fn sync_exits() -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (w, onchain_w) = wallet_guard.as_mut().context("Wallet not loaded")?;

    w.sync_exits(onchain_w)
        .await
        .context("Failed to sync exits")?;
    Ok(())
}
