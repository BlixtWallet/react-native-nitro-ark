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
use bark::onchain::{ChainSource, ChainSourceClient, OnchainWallet};
use bark::persist::BarkPersister;
use bark::Config;
use bark::Offboard;
use bark::SendOnchain;
use bark::SqliteClient;
use bark::Wallet;
use bdk_bitcoind_rpc::bitcoincore_rpc;
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

pub static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

// Global wallet manager instance
static GLOBAL_WALLET_MANAGER: LazyLock<Mutex<WalletManager>> =
    LazyLock::new(|| Mutex::new(WalletManager::new()));

// Wallet context that holds all wallet-related components
pub struct WalletContext {
    pub wallet: Wallet,
    pub onchain_wallet: OnchainWallet,
    pub chain_client: Arc<ChainSourceClient>,
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

    pub async fn load_wallet(&mut self, datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
        if self.context.is_some() {
            bail!("Wallet is already loaded. Please close it first.");
        }

        debug!("Loading wallet in {}", datadir.display());

        let net = match (opts.bitcoin, opts.signet, opts.regtest) {
            (true, false, false) => Network::Bitcoin,
            (false, true, false) => Network::Signet,
            (false, false, true) => Network::Regtest,
            _ => bail!("A network must be specified. Use either --signet, --regtest or --bitcoin"),
        };

        let mut config = Config {
            asp_address: opts
                .config
                .asp
                .clone()
                .context("ASP address missing, use --asp")?,
            ..Default::default()
        };
        opts.config
            .clone()
            .merge_into(&mut config)
            .context("invalid configuration")?;

        // check if dir doesn't exists, then create it
        if !datadir.exists() {
            info!("Wallet directory does not exist, creating it...");
            try_create_wallet(datadir, net, config.clone(), Some(opts.mnemonic.clone())).await?;
        }

        let auth = if let (Some(user), Some(pass)) =
            (config.bitcoind_user.clone(), config.bitcoind_pass.clone())
        {
            bitcoincore_rpc::Auth::UserPass(user, pass)
        } else {
            bitcoincore_rpc::Auth::None
        };

        let chain_source = if let Some(url) = config.esplora_address.clone() {
            ChainSource::Esplora { url }
        } else if let Some(url) = config.bitcoind_address.clone() {
            ChainSource::Bitcoind { url, auth }
        } else {
            bail!("Provide either an esplora or bitcoind url as chain source.");
        };

        let chain_client =
            Arc::new(ChainSourceClient::new(chain_source, net, config.fallback_fee_rate).await?);

        info!("Attempting to open wallet...");
        let (wallet, onchain_wallet) = self.open_wallet(datadir, opts.mnemonic).await?;

        self.context = Some(WalletContext {
            wallet,
            onchain_wallet,
            chain_client,
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
    ) -> anyhow::Result<(Wallet, OnchainWallet)> {
        debug!("Opening bark wallet in {}", datadir.display());

        let db = Arc::new(SqliteClient::open(datadir.join(DB_FILE))?);
        let properties = db
            .read_properties()?
            .context("Failed to read properties from db for opening wallet")?;

        let onchain_wallet =
            OnchainWallet::load_or_create(properties.network, mnemonic.to_seed(""), db.clone())?;
        let wallet = Wallet::open_with_onchain(&mnemonic, db.clone(), &onchain_wallet).await?;

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

pub async fn load_wallet(datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.load_wallet(datadir, opts).await
}

pub async fn close_wallet() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.close_wallet()
}

pub async fn is_wallet_loaded() -> bool {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.is_loaded()
}

pub async fn persist_config(config: Config) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| {
        ctx.wallet.set_config(config);
        ctx.wallet
            .persist_config()
            .context("Failed to persist wallet config to disk")?;
        info!("Wallet configuration updated successfully.");
        Ok(())
    })
}

pub struct Balance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64,
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
    manager.with_context(|ctx| Ok(ctx.wallet.derive_store_next_keypair()?))
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

pub async fn bolt11_invoice(amount: u64) -> anyhow::Result<String> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let invoice = ctx
                .wallet
                .bolt11_invoice(Amount::from_sat(amount))
                .await
                .context("Failed to create bolt11_invoice")?;
            Ok(invoice.to_string())
        })
        .await
}

pub async fn finish_lightning_receive(bolt11: Bolt11Invoice) -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            let _ = ctx
                .wallet
                .finish_lightning_receive(bolt11)
                .await
                .context("Failed to claim bolt11 payment")?;
            Ok(())
        })
        .await
}

pub async fn maintenance() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .maintenance(&mut ctx.onchain_wallet)
                .await
                .context("Failed to perform wallet maintenance")?;
            Ok(())
        })
        .await
}

pub async fn sync() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet.sync().await.context("Failed to sync wallet")?;
            Ok(())
        })
        .await
}

pub async fn get_vtxos() -> anyhow::Result<Vec<Vtxo>> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| Ok(ctx.wallet.vtxos()?))
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

pub async fn board_amount(amount: Amount) -> anyhow::Result<String> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            info!("Attempting to board amount: {}", amount);
            let board_result = ctx
                .wallet
                .board_amount(&mut ctx.onchain_wallet, amount)
                .await?;

            let json_string = serde_json::to_string_pretty(&board_result)
                .context("Failed to serialize board status to JSON")?;

            Ok(json_string)
        })
        .await
}

pub async fn board_all() -> anyhow::Result<String> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            info!("Attempting to board all onchain funds...");
            let board_result = ctx.wallet.board_all(&mut ctx.onchain_wallet).await?;

            let json_string = serde_json::to_string_pretty(&board_result)
                .context("Failed to serialize board status to JSON")?;

            Ok(json_string)
        })
        .await
}

pub async fn sync_rounds() -> anyhow::Result<()> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .sync_rounds()
                .await
                .context("Failed to sync rounds")?;
            Ok(())
        })
        .await
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

pub async fn send_lightning_payment(
    destination: Bolt11Invoice,
    amount_sat: Option<Amount>,
) -> anyhow::Result<Preimage> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.wallet
                .send_lightning_payment(&destination, amount_sat)
                .await
        })
        .await
}

pub async fn send_round_onchain_payment(
    addr: Address,
    amount: Amount,
) -> anyhow::Result<SendOnchain> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            Ok(ctx.wallet.send_round_onchain_payment(addr, amount).await?)
        })
        .await
}

pub async fn send_lnaddr(
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
                .send_lnaddr(&lightning_address, amount, comment)
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
