use anyhow;
use anyhow::bail;
use bark;
use bark::ark::bitcoin::address;
use bark::ark::bitcoin::hex::DisplayHex;
use bark::ark::bitcoin::Address;
use bark::ark::bitcoin::Amount;
use bark::ark::bitcoin::Network;
use bark::ark::bitcoin::Txid;
use bark::ark::ArkInfo;
use bark::ark::VtxoId;
use bark::json::cli::ExitProgressResponse;
use bark::json::cli::Refresh;
use bark::json::VtxoInfo;
use bark::lightning_invoice::Bolt11Invoice;
use bark::vtxo_selection::VtxoFilter;
use bark::Config;
use bark::SqliteClient;
use bark::UtxoInfo;
use bark::Wallet;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
mod cxx;
mod utils;

use bip39::Mnemonic;
use logger::log::{debug, info, warn};
use std::path::Path;
use std::sync::Once;
use utils::try_create_wallet;
use utils::DB_FILE;

pub use utils::*;

use std::str::FromStr;

use anyhow::Context;

// Use a static Once to ensure the logger is initialized only once.
static LOGGER_INIT: Once = Once::new();
static GLOBAL_WALLET: Lazy<Mutex<Option<Wallet>>> = Lazy::new(|| Mutex::new(None));
pub static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

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
    let mnemonic = bip39::Mnemonic::generate(12).context("failed to generate mnemonic")?;
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
        try_create_wallet(
            datadir,
            net,
            config,
            opts.mnemonic.clone(),
            opts.birthday_height,
        )
        .await?;
    }

    info!("Attempting to open wallet...");
    let wallet = open_wallet(datadir, opts.mnemonic).await?;
    *wallet_guard = Some(wallet);

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

pub struct Balance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64,
}

/// Get offchain and onchain balances
pub async fn get_balance(no_sync: bool) -> anyhow::Result<Balance> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing wallet...");
        if let Err(e) = w.sync().await {
            warn!("Sync error: {}", e)
        }
    }

    let onchain = w.onchain.balance().to_sat();
    let offchain = w.offchain_balance()?.to_sat();
    let pending_exit = w.exit.pending_total().await?.to_sat();

    let balances = Balance {
        onchain,
        offchain,
        pending_exit,
    };
    Ok(balances)
}

pub async fn open_wallet(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<Wallet> {
    debug!("Opening bark wallet in {}", datadir.display());

    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::open(&mnemonic, db).await
}

pub async fn get_ark_info() -> anyhow::Result<ArkInfo> {
    let wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_ref().context("Wallet not loaded")?;

    let info = w.ark_info();

    if let Some(info) = info {
        Ok(info.clone())
    } else {
        bail!("Failed to get ark info")
    }
}

/// Get an onchain address from the wallet
pub async fn get_onchain_address() -> anyhow::Result<Address> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    // Wallet::address() returns Result<Address, Error>
    let address = w
        .onchain
        .address()
        .context("Wallet failed to generate address")?;

    Ok(address)
}

/// Send funds using the onchain wallet
pub async fn send_onchain(
    destination_str: &str,
    amount: Amount,
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

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

/// Send all funds using the onchain wallet to a specific address
pub async fn drain_onchain(
    destination_str: &str, // Take string for validation
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let net = w.properties()?.network;

    // Validate address
    let address_unchecked = Address::<address::NetworkUnchecked>::from_str(destination_str)
        .with_context(|| format!("invalid destination address format: '{}'", destination_str))?;
    let destination_address = address_unchecked.require_network(net).with_context(|| {
        format!(
            "address '{}' is not valid for configured network {}",
            destination_str, net
        )
    })?;

    if !no_sync {
        info!("Syncing onchain wallet before draining...");
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during drain: {}", e);
            // Consider if this should be a hard error or warning
        }
    }

    info!("Draining onchain wallet to address {}", destination_address);
    let txid = w
        .onchain
        .drain(destination_address.clone())
        .await
        .with_context(|| format!("failed to drain wallet to {}", destination_address))?;

    info!("Onchain drain successful, TxID: {}", txid);
    Ok(txid)
}

/// Send funds to multiple recipients using the onchain wallet
pub async fn send_many_onchain(
    outputs: Vec<(Address, Amount)>, // Pass validated addresses and amounts
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    // Network validation should happen *before* calling this function, during FFI conversion.
    // The Vec<(Address, Amount)> should already contain addresses valid for the wallet's network.

    if !no_sync {
        info!("Syncing onchain wallet before send-many...");
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during send-many: {}", e);
            // Consider if this should be a hard error or warning
        }
    }

    info!("Sending onchain transaction with {} outputs", outputs.len());
    let txid = w
        .onchain
        .send_many(outputs)
        .await
        .context("failed to send transaction with multiple outputs")?;

    info!("Onchain send-many successful, TxID: {}", txid);
    Ok(txid)
}

/// Get the list of UTXOs from the onchain wallet as a JSON string
pub async fn get_onchain_utxos(no_sync: bool) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing onchain wallet before getting UTXOs...");
        // Sync only the onchain part and potentially exits that might create UTXOs
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during get_utxos: {}", e);
        }
        if let Err(e) = w.sync_exits().await {
            // Exits can produce UTXOs
            warn!("Exit sync error during get_utxos: {}", e)
        }
    }

    let utxos = w
        .onchain
        .utxos()
        .into_iter()
        .map(UtxoInfo::from)
        .collect::<Vec<UtxoInfo>>();
    debug!("Retrieved {} UTXOs from bdk wallet.", utxos.len());

    let json_string =
        serde_json::to_string_pretty(&utxos).context("Failed to serialize UTXOs to JSON")?;

    debug!("Serialized UTXOs to JSON string.");
    Ok(json_string)
}

/// Get the VTXO public key (OOR Pubkey) as a hex string
pub async fn get_vtxo_pubkey(index: Option<u32>) -> anyhow::Result<String> {
    let wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_ref().context("Wallet not loaded")?;

    if let Some(index) = index {
        Ok(w.peak_keypair(bark::KeychainKind::External, index)
            .context("Failed to get VTXO pubkey")?
            .public_key()
            .to_string())
    } else {
        Ok(w.derive_store_next_keypair(bark::KeychainKind::External)
            .context("Failed to get VTXO pubkey")?
            .public_key()
            .to_string())
    }
}

/// Get a Bolt 11 invoice
pub async fn bolt11_invoice(amount: u64) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let invoice = w
        .bolt11_invoice(Amount::from_sat(amount))
        .await
        .context("Failed to create bolt11_invoice")?;
    Ok(invoice.to_string())
}

/// Claim a lightning payment
pub async fn claim_bolt11_payment(bolt11: String) -> anyhow::Result<()> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let _ = w
        .claim_bolt11_payment(Bolt11Invoice::from_str(&bolt11)?)
        .await
        .context("Failed to claim bolt11 payment")?;

    Ok(())
}

/// Get the list of VTXOs from the wallet as a JSON string
pub async fn get_vtxos(no_sync: bool) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing wallet before getting VTXOs...");
        // Use maintenance sync as VTXOs depend on both onchain and offchain state
        if let Err(e) = w.maintenance().await {
            warn!("Wallet maintenance sync error during get_vtxos: {}", e);
        }
    }

    let vtxos: Vec<VtxoInfo> = w
        .vtxos()
        .context("Failed to retrieve VTXOs from wallet")?
        .into_iter()
        .map(|v| v.into())
        .collect();

    let json_string = serde_json::to_string(&vtxos)?;

    Ok(json_string)
}

/// Refresh VTXOs based on specified criteria. Returns JSON status.
pub async fn refresh_vtxos(mode: RefreshMode, no_sync: bool) -> anyhow::Result<String> {
    let round_id_opt = refresh_vtxos_internal(mode, no_sync).await?;

    // Convert Option<RoundId> to Option<String> for JSON
    let round_string_opt = round_id_opt.map(|id| id.to_string());

    // Construct CLI JSON response
    let refresh_output = Refresh {
        participate_round: round_string_opt.is_some(),
        round: round_id_opt,
    };

    let json_string = serde_json::to_string_pretty(&refresh_output)
        .context("Failed to serialize refresh status to JSON")?;

    Ok(json_string)
}

/// Board a specific amount from the onchain wallet to Ark. Returns JSON status.
pub async fn board_amount(amount: Amount, no_sync: bool) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing onchain wallet before boarding amount...");
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during board_amount: {}", e);
        }
    }

    info!("Attempting to board amount: {}", amount);
    let board_result = w.board_amount(amount).await?;

    let json_string = serde_json::to_string_pretty(&board_result)
        .context("Failed to serialize board status to JSON")?;

    Ok(json_string)
}

pub async fn board_all(no_sync: bool) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing onchain wallet before boarding all...");
        if let Err(e) = w.onchain.sync().await {
            warn!("Onchain sync error during board_all: {}", e);
        }
    }

    info!("Attempting to board all onchain funds...");
    let board_result = w.board_all().await?;

    let json_string = serde_json::to_string_pretty(&board_result)
        .context("Failed to serialize board status to JSON")?;

    Ok(json_string)
}

/// Send a payment based on destination type. Returns JSON status.
pub async fn send_payment(
    destination_str: &str,
    amount_sat: Option<u64>,
    comment: Option<String>,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let destination = parse_send_destination(destination_str)?;

    // Convert optional amount_sat to Option<Amount>
    let user_amount_opt = amount_sat.map(Amount::from_sat);

    // --- Logic per destination type ---
    let result_json = match destination {
        SendDestination::VtxoPubkey(pk) => {
            let amount = user_amount_opt
                .context("Amount (amount_sat) is required when sending to a VTXO pubkey")?;
            if amount <= Amount::ZERO {
                bail!("Amount must be positive");
            }
            if comment.is_some() {
                bail!("Comment is not supported when sending to a VTXO pubkey");
            }

            if !no_sync {
                info!("Syncing wallet before sending OOR payment...");
                // Maintenance sync likely needed for OOR
                if let Err(e) = w.maintenance().await {
                    warn!("Wallet maintenance sync error during send (OOR): {}", e);
                }
            }

            info!(
                "Attempting to send OOR payment of {} to pubkey {}",
                amount, pk
            );
            let _oor_result = w.send_arkoor_payment(pk, amount).await?; // Use result if it contains info

            serde_json::json!({
                "type": "oor",
                "success": true,
                "destination_pubkey": pk.to_string(),
                "amount_sat": amount.to_sat()
            })
        }
        SendDestination::Bolt11(invoice) => {
            // Validate amount:
            // 1. If user provided amount, it MUST match invoice amount if invoice has one.
            // 2. If user didn't provide amount, invoice MUST have one.
            let invoice_amount_opt = invoice
                .amount_milli_satoshis()
                .map(|msat| Amount::from_sat(msat.div_ceil(1000)));
            let final_amount = match (user_amount_opt, invoice_amount_opt) {
                (Some(user), Some(inv)) if user != inv => {
                    bail!(
                        "Provided amount {} does not match invoice amount {}",
                        user,
                        inv
                    );
                }
                (Some(user), _) => user,
                (None, Some(inv)) => inv,
                (None, None) => {
                    bail!("Amount (amount_sat) is required for invoices without an amount");
                }
            };
            if final_amount <= Amount::ZERO {
                bail!("Amount must be positive");
            }
            // Check again if invoice required an amount but user didn't supply one (covered by None, None case above)
            if invoice_amount_opt.is_none() && user_amount_opt.is_none() {
                bail!("Amount (amount_sat) is required for invoices without an amount");
            }

            if comment.is_some() {
                bail!("Comment is not supported when sending to a bolt11 invoice");
            }

            if !no_sync {
                info!("Syncing wallet before sending bolt11 payment...");
                // sync_ark likely needed for paying LN
                if let Err(e) = w.sync_ark().await {
                    warn!("Wallet sync error during send (bolt11): {}", e);
                }
            }

            info!(
                "Attempting to send bolt11 payment of {} for invoice {}",
                final_amount, invoice
            );
            let bolt11_result = w.send_bolt11_payment(&invoice, user_amount_opt).await?; // Pass user_amount_opt

            serde_json::json!({
                "type": "bolt11",
                "success": true,
                "destination_invoice": invoice.to_string(),
                "amount_sat": final_amount.to_sat(), // Use final_amount derived logic
                "preimage": bolt11_result.to_lower_hex_string()
            })
        }
        SendDestination::LnAddress(lnaddr) => {
            let amount = user_amount_opt
                .context("Amount (amount_sat) is required when sending to a lightning address")?;
            if amount <= Amount::ZERO {
                bail!("Amount must be positive");
            }
            // Comment is allowed here

            if !no_sync {
                info!("Syncing wallet before sending to lightning address...");
                // sync_ark likely needed for paying LN
                if let Err(e) = w.sync_ark().await {
                    warn!("Wallet sync error during send (lnaddr): {}", e);
                }
            }

            info!(
                "Attempting to send {} to lightning address {} (comment: {:?})",
                amount, lnaddr, comment
            );
            let (lnaddr_result, _) = w.send_lnaddr(&lnaddr, amount, comment.as_deref()).await?;

            serde_json::json!({
                "type": "ln_address",
                "success": true,
                "destination_address": lnaddr.to_string(),
                "amount_sat": amount.to_sat(),
                "paid_invoice": lnaddr_result.to_string(),
            })
        }
    };

    let json_string = serde_json::to_string_pretty(&result_json)
        .context("Failed to serialize send status to JSON")?;

    Ok(json_string)
}

/// Send an onchain payment via an Ark round. Returns JSON status.
pub async fn send_round_onchain(
    destination_str: &str,
    amount: Amount,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let net = w.properties()?.network;

    // Validate address
    let addr_unchecked = Address::<address::NetworkUnchecked>::from_str(destination_str)
        .with_context(|| format!("Invalid destination address format: '{}'", destination_str))?;
    let destination_address = addr_unchecked.require_network(net).with_context(|| {
        format!(
            "Address '{}' is not valid for configured network {}",
            destination_str, net
        )
    })?;

    if amount <= Amount::ZERO {
        bail!("Amount must be positive");
    }

    if !no_sync {
        info!("Syncing wallet before sending round onchain payment...");
        // Maintenance sync likely needed for round participation
        if let Err(e) = w.maintenance().await {
            warn!(
                "Wallet maintenance sync error during send_round_onchain: {}",
                e
            );
        }
    }

    info!(
        "Attempting to send round onchain payment of {} to {}",
        amount, destination_address
    );
    // Assuming send_round_onchain_payment returns Result<(), Error>
    w.send_round_onchain_payment(destination_address.clone(), amount)
        .await?;

    // Construct success JSON
    let result_json = serde_json::json!({
        "type": "round_onchain",
        "success": true,
        "destination_address": destination_address.to_string(),
        "amount_sat": amount.to_sat()
    });

    let json_string = serde_json::to_string_pretty(&result_json)
        .context("Failed to serialize send_round_onchain status to JSON")?;

    Ok(json_string)
}

/// Offboard specific VTXOs. Returns JSON result.
pub async fn offboard_specific(
    vtxo_ids: Vec<VtxoId>,
    destination_address_str: Option<String>, // Optional destination address string
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let net = w.properties()?.network;

    // Validate optional address string
    let destination_address_opt: Option<Address> = match destination_address_str {
        Some(addr_str) => {
            let addr_unchecked = Address::<address::NetworkUnchecked>::from_str(&addr_str)
                .with_context(|| format!("Invalid destination address format: '{}'", addr_str))?;
            let addr = addr_unchecked.require_network(net).with_context(|| {
                format!(
                    "Address '{}' is not valid for configured network {}",
                    addr_str, net
                )
            })?;
            Some(addr)
        }
        None => None,
    };

    if vtxo_ids.is_empty() {
        bail!("At least one VTXO ID must be provided for specific offboarding");
    }

    if !no_sync {
        info!("Syncing wallet before offboarding specific VTXOs...");
        // Maintenance sync might be needed
        if let Err(e) = w.maintenance().await {
            warn!(
                "Wallet maintenance sync error during offboard_specific: {}",
                e
            );
        }
    }

    info!(
        "Attempting to offboard {} specific VTXOs to {:?}",
        vtxo_ids.len(),
        destination_address_opt
    );
    let offboard_result = w.offboard_vtxos(vtxo_ids, destination_address_opt).await?;

    let json_string = serde_json::to_string_pretty(&offboard_result)
        .context("Failed to serialize offboard status to JSON")?;

    Ok(json_string)
}

/// Offboard all VTXOs. Returns JSON result.
pub async fn offboard_all(
    destination_address_str: Option<String>, // Optional destination address string
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    let net = w.properties()?.network;

    // Validate optional address string
    let destination_address_opt: Option<Address> = match destination_address_str {
        Some(addr_str) => {
            let addr_unchecked = Address::<address::NetworkUnchecked>::from_str(&addr_str)
                .with_context(|| format!("Invalid destination address format: '{}'", addr_str))?;
            let addr = addr_unchecked.require_network(net).with_context(|| {
                format!(
                    "Address '{}' is not valid for configured network {}",
                    addr_str, net
                )
            })?;
            Some(addr)
        }
        None => None,
    };

    if !no_sync {
        info!("Syncing wallet before offboarding all VTXOs...");
        // sync_ark might be needed to find all VTXOs correctly
        if let Err(e) = w.sync_ark().await {
            warn!("Wallet sync_ark error during offboard_all: {}", e);
        }
    }

    info!(
        "Attempting to offboard all VTXOs to {:?}",
        destination_address_opt
    );
    let offboard_result = w.offboard_all(destination_address_opt).await?;

    let json_string = serde_json::to_string_pretty(&offboard_result)
        .context("Failed to serialize offboard status to JSON")?;

    Ok(json_string)
}

/// Start the exit process for specific VTXOs. Returns simple success JSON.
pub async fn start_exit_for_vtxos(vtxo_ids: Vec<VtxoId>) -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if vtxo_ids.is_empty() {
        bail!("At least one VTXO ID must be provided for starting specific exit");
    }

    // Syncing is crucial before starting an exit
    info!("Syncing wallet before starting specific exit...");
    if let Err(err) = w.onchain.sync().await {
        warn!("Failed to perform onchain sync during exit start: {}", err);
    }
    if let Err(err) = w.sync_ark().await {
        warn!("Failed to perform ark sync during exit start: {}", err);
    }

    info!("Fetching specific VTXOs for exit...");
    let filter = VtxoFilter::new(&w).include_many(vtxo_ids.clone()); // Clone ids if needed later
    let vtxos_to_exit = w
        .vtxos_with(filter)
        .context("Error finding specified vtxos for exit")?;

    if vtxos_to_exit.len() != vtxo_ids.len() {
        warn!("Could not find all specified VTXO IDs. Found {} out of {}. Proceeding with found VTXOs.", vtxos_to_exit.len(), vtxo_ids.len());
        if vtxos_to_exit.is_empty() {
            bail!("None of the specified VTXOs were found.");
        }
    }

    info!(
        "Starting exit process for {} specific VTXOs...",
        vtxos_to_exit.len()
    );
    w.exit
        .start_exit_for_vtxos(&vtxos_to_exit, &mut w.onchain)
        .await?;

    // Return simple success JSON
    let success_json = serde_json::json!({ "success": true, "type": "start_specific", "vtxo_count": vtxos_to_exit.len() });
    let json_string = serde_json::to_string_pretty(&success_json)?;
    Ok(json_string)
}

/// Start the exit process for the entire wallet. Returns simple success JSON.
/// This function starts the exit process for all vtxos in the wallet.
/// It returns a JSON object indicating success.
pub async fn start_exit_for_entire_wallet() -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    // Syncing is crucial
    info!("Syncing wallet before starting exit for all VTXOs...");
    if let Err(err) = w.onchain.sync().await {
        warn!(
            "Failed to perform onchain sync during exit start all: {}",
            err
        );
    }
    if let Err(err) = w.sync_ark().await {
        warn!("Failed to perform ark sync during exit start all: {}", err);
    }

    info!("Starting exit process for entire wallet...");
    w.exit.start_exit_for_entire_wallet(&mut w.onchain).await?;

    let success_json = serde_json::json!({ "success": true, "type": "start_all" });
    let json_string = serde_json::to_string_pretty(&success_json)?;
    Ok(json_string)
}

/// This function processes the exit queue for the wallet.
/// It returns a JSON object with the exit status, including whether the process is
/// done, the spendable height for exits, and any new exit transactions.
pub async fn exit_progress_once() -> anyhow::Result<String> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    // Sync before progressing - crucial for exit state
    info!("Syncing wallet before progressing exit...");
    if let Err(error) = w.onchain.sync().await {
        warn!(
            "Failed to perform onchain sync during exit progress: {}",
            error
        )
    }
    if let Err(error) = w.sync_exits().await {
        // sync_exits is important here
        warn!("Failed to sync exits during exit progress: {}", error);
    }
    info!("Wallet sync completed for exit progress");

    info!("Attempting to progress exit process...");
    let result = w
        .exit
        .progress_exit(&mut w.onchain)
        .await
        .context("Error making progress on exit process")?;

    // Check status after progressing
    let has_pending_exits = w.exit.has_pending_exits();
    let spendable_height = w.exit.all_spendable_at_height().await;

    info!(
        "Exit progress check: Done={}, Spendable Height={:?}",
        has_pending_exits, spendable_height
    );

    let exits = result.unwrap_or_default();

    let json_string = serde_json::to_string_pretty(&ExitProgressResponse {
        done: !has_pending_exits,
        spendable_height,
        exits,
    })
    .context("Failed to serialize exit status to JSON")?;
    Ok(json_string)
}

#[cfg(test)]
mod tests;
