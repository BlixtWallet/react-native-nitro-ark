use anyhow;
use anyhow::bail;
use anyhow::Ok;
use bark;

use bark::ark::bitcoin::Address;
use bark::ark::bitcoin::Amount;
use bark::ark::bitcoin::Network;
use bark::Board;

use bark::ark::lightning;
use bark::ark::lightning::Bolt12Invoice;
use bark::ark::lightning::Offer;
use bark::ark::lightning::PaymentHash;
use bark::ark::lightning::Preimage;
use bark::ark::rounds::RoundId;
use bark::ark::ArkInfo;
use bark::ark::Vtxo;
use bark::ark::VtxoId;
use bark::lightning_invoice::Bolt11Invoice;
use bark::lnurllib::lightning_address::LightningAddress;
use bark::movement::Movement;
use bark::onchain::OnchainWallet;
use bark::persist::models::LightningReceive;
use bark::persist::BarkPersister;
use bark::Config;
use bark::Offboard;
use bark::SqliteClient;
use bark::Wallet;
use bark::WalletVtxo;
use bdk_wallet::bitcoin::bip32;
use bdk_wallet::bitcoin::key::Keypair;
use bitcoin_ext::BlockHeight;
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
const ARK_PURPOSE_INDEX: u32 = 350;

pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

// Global wallet manager instance
static GLOBAL_WALLET_MANAGER: LazyLock<Mutex<WalletManager>> =
    LazyLock::new(|| Mutex::new(WalletManager::new()));

// Wallet context that holds all wallet-related components
pub struct WalletContext {
    pub wallet: Wallet,
    pub onchain_wallet: OnchainWallet,
}

// Wallet manager that manages the wallet context lifecycle
pub struct WalletManager {
    context: Option<WalletContext>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self { context: None }
    }

    pub fn is_loaded(&self) -> bool {
        self.context.is_some()
    }

    async fn create_wallet(&mut self, datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
        debug!("Creating wallet in {}", datadir.display());

        let (config, net) = merge_config_opts(opts.clone())?;

        try_create_wallet(datadir, net, config.clone(), Some(opts.mnemonic.clone())).await?;

        Ok(())
    }

    async fn load_wallet(
        &mut self,
        datadir: &Path,
        mnemonic: Mnemonic,
        config: Config,
    ) -> anyhow::Result<()> {
        if self.context.is_some() {
            bail!("Wallet is already loaded. Please close it first.");
        }

        debug!("Loading wallet in {}", datadir.display());

        if !datadir.exists() {
            bail!("Datadir does not exist. Please create a new wallet first.");
        }

        info!("Attempting to open wallet...");
        let (wallet, onchain_wallet) = self.open_wallet(datadir, mnemonic, config).await?;

        self.context = Some(WalletContext {
            wallet: wallet,
            onchain_wallet,
        });

        Ok(())
    }

    pub fn close_wallet(&mut self) -> anyhow::Result<()> {
        if self.context.is_none() {
            bail!("No wallet is currently loaded.");
        }
        self.context = None;
        info!("Wallet closed successfully.");
        Ok(())
    }

    pub async fn get_config(&self) -> anyhow::Result<Config> {
        match &self.context {
            Some(ctx) => Ok(ctx.wallet.config().clone()),
            None => bail!("Wallet not loaded"),
        }
    }

    pub fn with_context<T, F>(&mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut WalletContext) -> anyhow::Result<T>,
    {
        match &mut self.context {
            Some(ctx) => f(ctx),
            None => bail!("Wallet not loaded"),
        }
    }

    pub fn with_context_ref<T, F>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&WalletContext) -> anyhow::Result<T>,
    {
        match &self.context {
            Some(ctx) => f(ctx),
            None => bail!("Wallet not loaded"),
        }
    }

    pub async fn with_context_async<'a, T, F, Fut>(&'a mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&'a mut WalletContext) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        match &mut self.context {
            Some(ctx) => f(ctx).await,
            None => bail!("Wallet not loaded"),
        }
    }

    pub async fn with_context_ref_async<T, F, Fut>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&WalletContext) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        match &self.context {
            Some(ctx) => f(ctx).await,
            None => bail!("Wallet not loaded"),
        }
    }

    async fn open_wallet(
        &self,
        datadir: &Path,
        mnemonic: Mnemonic,
        config: Config,
    ) -> anyhow::Result<(Wallet, OnchainWallet)> {
        debug!("Opening bark wallet in {}", datadir.display());

        let db = Arc::new(SqliteClient::open(datadir.join(DB_FILE))?);
        let properties = db
            .read_properties()?
            .context("Failed to read properties from db for opening wallet")?;

        let onchain_wallet =
            OnchainWallet::load_or_create(properties.network, mnemonic.to_seed(""), db.clone())?;
        let wallet =
            Wallet::open_with_onchain(&mnemonic, db.clone(), &onchain_wallet, config).await?;

        Ok((wallet, onchain_wallet))
    }
}

// function to explicitly initialize the logger.
// This should be called once from your FFI entry point.
pub fn init_logger() {
    LOGGER_INIT.call_once(|| {
        logger::Logger::new();
    });
}

pub fn create_mnemonic() -> anyhow::Result<String> {
    info!("Attempting to create a new mnemonic using cxx bridge...");
    let mnemonic = Mnemonic::generate(12).context("failed to generate mnemonic")?;
    info!("Successfully created a new mnemonic using cxx bridge.");
    Ok(mnemonic.to_string())
}

pub async fn create_wallet(datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.create_wallet(datadir, opts).await
}

pub async fn load_wallet(datadir: &Path, mnemonic: Mnemonic, config: Config) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.load_wallet(datadir, mnemonic, config).await
}

pub async fn close_wallet() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.close_wallet()
}

pub async fn is_wallet_loaded() -> bool {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.is_loaded()
}

pub async fn balance() -> anyhow::Result<bark::Balance> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| Ok(ctx.wallet.balance()?))
}

pub async fn get_ark_info() -> anyhow::Result<ArkInfo> {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context_ref(|ctx| {
        let info = ctx.wallet.ark_info();
        if let Some(info) = info {
            Ok(info.clone())
        } else {
            bail!("Failed to get ark info")
        }
    })
}

pub async fn derive_store_next_keypair() -> anyhow::Result<Keypair> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        ctx.wallet
            .derive_store_next_keypair()
            .map(|(keypair, _)| keypair)
    })
}

pub async fn peak_keypair(index: u32) -> anyhow::Result<Keypair> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        Ok(ctx
            .wallet
            .peak_keypair(index)
            .context("Failed to peak keypair")?)
    })
}

pub async fn new_address() -> anyhow::Result<bark::ark::Address> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        Ok(ctx
            .wallet
            .new_address()
            .context("Failed to create new address")?)
    })
}

pub async fn sign_message(
    message: &str,
    index: u32,
) -> anyhow::Result<bark::ark::bitcoin::secp256k1::ecdsa::Signature> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        let wallet = &ctx.wallet;
        let keypair = wallet
            .peak_keypair(index)
            .context("Failed to peak keypair")?;
        let hash = bark::ark::bitcoin::sign_message::signed_msg_hash(message);
        let secp = bark::ark::bitcoin::secp256k1::Secp256k1::new();
        let msg = bark::ark::bitcoin::secp256k1::Message::from_digest_slice(&hash[..]).unwrap();
        let ecdsa_sig = secp.sign_ecdsa(&msg, &keypair.secret_key());

        Ok(ecdsa_sig)
    })
}

pub async fn sign_messsage_with_mnemonic(
    message: &str,
    mnemonic: Mnemonic,
    network: Network,
    index: u32,
) -> anyhow::Result<bark::ark::bitcoin::secp256k1::ecdsa::Signature> {
    let secp = bark::ark::bitcoin::secp256k1::Secp256k1::new();
    let keypair = bip32::Xpriv::new_master(network, &mnemonic.to_seed(""))?
        .derive_priv(&secp, &[ARK_PURPOSE_INDEX.into()])?
        .derive_priv(&secp, &[index.into()])?
        .to_keypair(&secp);

    let hash = bark::ark::bitcoin::sign_message::signed_msg_hash(message);
    let msg = bark::ark::bitcoin::secp256k1::Message::from_digest_slice(&hash[..]).unwrap();
    let ecdsa_sig = secp.sign_ecdsa(&msg, &keypair.secret_key());

    Ok(ecdsa_sig)
}

pub async fn derive_keypair_from_mnemonic(
    mnemonic: Mnemonic,
    network: Network,
    index: u32,
) -> anyhow::Result<Keypair> {
    let secp = bark::ark::bitcoin::secp256k1::Secp256k1::new();
    let keypair = bip32::Xpriv::new_master(network, &mnemonic.to_seed(""))?
        .derive_priv(&secp, &[ARK_PURPOSE_INDEX.into()])?
        .derive_priv(&secp, &[index.into()])?
        .to_keypair(&secp);
    Ok(keypair)
}

pub async fn verify_message(
    message: &str,
    signature: bark::ark::bitcoin::secp256k1::ecdsa::Signature,
    public_key: &bark::ark::bitcoin::secp256k1::PublicKey,
) -> anyhow::Result<bool> {
    let hash = bark::ark::bitcoin::sign_message::signed_msg_hash(message);
    let secp = bark::ark::bitcoin::secp256k1::Secp256k1::new();
    let msg = bark::ark::bitcoin::secp256k1::Message::from_digest_slice(&hash[..]).unwrap();
    Ok(secp.verify_ecdsa(&msg, &signature, &public_key).is_ok())
}

pub async fn bolt11_invoice(amount: u64) -> anyhow::Result<Bolt11Invoice> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let invoice = ctx
                .wallet
                .bolt11_invoice(Amount::from_sat(amount))
                .await
                .context("Failed to create bolt11_invoice")?;
            Ok(invoice)
        })
        .await
}

pub async fn lightning_receive_status(
    payment: PaymentHash,
) -> anyhow::Result<Option<LightningReceive>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        let status = ctx
            .wallet
            .lightning_receive_status(payment)
            .context("Failed to get lightning receive status")?;
        Ok(status)
    })
}

pub async fn check_and_claim_ln_receive(
    payment_hash: PaymentHash,
    wait: bool,
) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let _ = ctx
                .wallet
                .check_and_claim_ln_receive(payment_hash, wait)
                .await
                .context("Failed to claim bolt11 payment")?;
            Ok(())
        })
        .await
}

pub async fn check_and_claim_all_open_ln_receives(wait: bool) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let _ = ctx
                .wallet
                .check_and_claim_all_open_ln_receives(wait)
                .await
                .context("Failed to claim all open invoices")?;
            Ok(())
        })
        .await
}

pub async fn sync_pending_boards() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let _ = ctx
                .wallet
                .sync_pending_boards()
                .await
                .context("Failed to sync pending boards")?;
            Ok(())
        })
        .await
}

pub async fn maintenance() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .maintenance()
                .await
                .context("Failed to perform wallet maintenance")?;
            Ok(())
        })
        .await
}

pub async fn maintenance_with_onchain() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .maintenance_with_onchain(&mut ctx.onchain_wallet)
                .await
                .context("Failed to perform wallet maintenance with onchain")?;
            Ok(())
        })
        .await
}

pub async fn maintenance_refresh() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .maintenance_refresh()
                .await
                .context("Failed to perform vtxo maintenance refresh")?;
            Ok(())
        })
        .await
}

pub async fn sync() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet.sync().await;
            Ok(())
        })
        .await
}

pub async fn movements() -> anyhow::Result<Vec<Movement>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| Ok(ctx.wallet.movements()?))
}

pub async fn vtxos() -> anyhow::Result<Vec<WalletVtxo>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| Ok(ctx.wallet.vtxos()?))
}

pub async fn get_expiring_vtxos(threshold: BlockHeight) -> anyhow::Result<Vec<WalletVtxo>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;

    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .get_expiring_vtxos(threshold)
                .await
                .context("Failed to get expiring vtxos")
        })
        .await
}

pub async fn refresh_vtxos(vtxos: Vec<Vtxo>) -> anyhow::Result<Option<RoundId>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .refresh_vtxos(vtxos)
                .await
                .context("Failed to refresh vtxos")
        })
        .await
}

/// Returns the block height at which the first VTXO will expire
pub async fn get_first_expiring_vtxo_blockheight() -> anyhow::Result<Option<BlockHeight>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        ctx.wallet
            .get_first_expiring_vtxo_blockheight()
            .context("Failed to get first expiring vtxo blockheight")
    })
}

/// Returns the next block height at which we have a VTXO that we
/// want to refresh
pub async fn get_next_required_refresh_blockheight() -> anyhow::Result<Option<BlockHeight>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        ctx.wallet
            .get_next_required_refresh_blockheight()
            .context("Failed to get next required refresh blockheight")
    })
}

pub async fn board_amount(amount: Amount) -> anyhow::Result<Board> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .board_amount(&mut ctx.onchain_wallet, amount)
                .await
        })
        .await
}

pub async fn board_all() -> anyhow::Result<Board> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async { ctx.wallet.board_all(&mut ctx.onchain_wallet).await })
        .await
}

pub async fn sync_past_rounds() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .sync_past_rounds()
                .await
                .context("Failed to sync rounds")?;
            Ok(())
        })
        .await
}

pub async fn validate_arkoor_address(address: bark::ark::Address) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        ctx.wallet
            .validate_arkoor_address(&address)
            .context("Failed to validate address")?;
        Ok(())
    })
}

pub async fn send_arkoor_payment(
    destination: bark::ark::Address,
    amount_sat: Amount,
) -> anyhow::Result<Vec<Vtxo>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            info!(
                "Attempting to send OOR payment of {} to pubkey {:?}",
                amount_sat, destination
            );
            let oor_result = ctx
                .wallet
                .send_arkoor_payment(&destination, amount_sat)
                .await?;
            Ok(oor_result)
        })
        .await
}

pub async fn pay_lightning_invoice(
    destination: lightning::Invoice,
    amount_sat: Option<Amount>,
) -> anyhow::Result<Preimage> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .pay_lightning_invoice(destination, amount_sat)
                .await
        })
        .await
}

pub async fn pay_lightning_offer(
    offer: Offer,
    amount: Option<Amount>,
) -> anyhow::Result<(Bolt12Invoice, Preimage)> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async { ctx.wallet.pay_lightning_offer(offer, amount).await })
        .await
}

pub async fn send_round_onchain_payment(addr: Address, amount: Amount) -> anyhow::Result<Offboard> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            Ok(ctx.wallet.send_round_onchain_payment(addr, amount).await?)
        })
        .await
}

pub async fn pay_lightning_address(
    addr: &str,
    amount: Amount,
    comment: Option<&str>,
) -> anyhow::Result<(Bolt11Invoice, Preimage)> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let lightning_address = LightningAddress::from_str(addr)
                .with_context(|| format!("Invalid Lightning Address format: '{}'", addr))?;

            ctx.wallet
                .pay_lightning_address(&lightning_address, amount, comment)
                .await
        })
        .await
}

pub async fn offboard_specific(
    vtxo_ids: Vec<VtxoId>,
    address: Address,
) -> anyhow::Result<Offboard> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async { ctx.wallet.offboard_vtxos(vtxo_ids, address).await })
        .await
}

pub async fn offboard_all(address: Address) -> anyhow::Result<Offboard> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async { ctx.wallet.offboard_all(address).await })
        .await
}

pub async fn sync_exits() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .sync_exits(&mut ctx.onchain_wallet)
                .await
                .context("Failed to sync exits")?;
            Ok(())
        })
        .await
}
