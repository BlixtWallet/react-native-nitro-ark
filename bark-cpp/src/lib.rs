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
use bark::json::cli::ExitStatus;
use bark::json::cli::Refresh;
use bark::json::VtxoInfo;
use bark::vtxo_selection::VtxoFilter;
use bark::Config;
use bark::SqliteClient;
use bark::UtxoInfo;
use bark::Wallet;
mod ffi;
mod utils;
use bip39::Mnemonic;
use logger::log::{debug, info, warn};
use std::fs;
use std::path::Path;
use utils::try_create_wallet;
use utils::DB_FILE;

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

/// Send all funds using the onchain wallet to a specific address
pub async fn drain_onchain(
    datadir: &Path,
    mnemonic: Mnemonic,
    destination_str: &str, // Take string for validation
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for drain_onchain")?;

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
    datadir: &Path,
    mnemonic: Mnemonic,
    outputs: Vec<(Address, Amount)>, // Pass validated addresses and amounts
    no_sync: bool,
) -> anyhow::Result<Txid> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for send_many_onchain")?;

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
    // Note: The CLI asks for confirmation here, we skip that in the library function.
    let txid = w
        .onchain
        .send_many(outputs)
        .await
        .context("failed to send transaction with multiple outputs")?;

    info!("Onchain send-many successful, TxID: {}", txid);
    Ok(txid)
}

/// Get the list of UTXOs from the onchain wallet as a JSON string
pub async fn get_onchain_utxos(
    datadir: &Path,
    mnemonic: Mnemonic,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for get_onchain_utxos")?;

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

    // Get UTXOs from the wallet. `w.onchain.utxos()` returns Vec<bdk::UtxoInfo>
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
pub async fn get_vtxo_pubkey(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<String> {
    // This might not need to be async if opening the wallet doesn't require async ops
    // But open_wallet is async, so we keep it async.
    let w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for get_vtxo_pubkey")?;

    let pubkey = w.oor_pubkey();
    let pubkey_hex = pubkey.to_string(); // PublicKey's Display impl is hex

    debug!("Retrieved VTXO Pubkey: {}", pubkey_hex);
    Ok(pubkey_hex)
}

/// Get the list of VTXOs from the wallet as a JSON string
pub async fn get_vtxos(
    datadir: &Path,
    mnemonic: Mnemonic,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(&datadir, mnemonic)
        .await
        .context("error opening wallet for get_vtxos")?;

    if !no_sync {
        info!("Syncing wallet before getting VTXOs...");
        // Use maintenance sync as VTXOs depend on both onchain and offchain state
        if let Err(e) = w.maintenance().await {
            warn!("Wallet maintenance sync error during get_vtxos: {}", e);
        }
    }

    // Get VTXOs from the wallet. `w.vtxos()` returns Result<Vec<Vtxo>, Error>
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
pub async fn refresh_vtxos(
    datadir: &Path,
    mnemonic: Mnemonic,
    mode: RefreshMode,
    no_sync: bool,
) -> anyhow::Result<String> {
    let round_id_opt = refresh_vtxos_internal(datadir, mnemonic, mode, no_sync).await?;

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
pub async fn board_amount(
    datadir: &Path,
    mnemonic: Mnemonic,
    amount: Amount,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for board_amount")?;

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

pub async fn board_all(
    datadir: &Path,
    mnemonic: Mnemonic,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for board_all")?;

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
    datadir: &Path,
    mnemonic: Mnemonic,
    destination_str: &str,
    amount_sat: Option<u64>, // Amount provided by user (None if not provided)
    comment: Option<String>,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for send_payment")?;

    let destination = parse_send_destination(destination_str)?;

    // Convert optional amount_sat to Option<Amount>
    let user_amount_opt: Option<Amount> = amount_sat.map(Amount::from_sat);

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
            // Assuming send_oor_payment returns Result<OorPayResult, Error> or similar
            let _oor_result = w.send_oor_payment(pk, amount).await?; // Use result if it contains info

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
                (Some(user), _) => user, // User provided, and matches invoice or invoice had none (checked later)
                (None, Some(inv)) => inv, // User didn't provide, use invoice amount
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
            // Assuming send_bolt11_payment returns Result<Bolt11PayResult, Error> containing preimage
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
            // Assuming send_lnaddr returns Result<LnAddrPayResult, Error> containing invoice and preimage
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
    datadir: &Path,
    mnemonic: Mnemonic,
    destination_str: &str,
    amount: Amount,
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for send_round_onchain")?;

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
    datadir: &Path,
    mnemonic: Mnemonic,
    vtxo_ids: Vec<VtxoId>,
    destination_address_str: Option<String>, // Optional destination address string
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for offboard_specific")?;

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
    // Assuming w.offboard_vtxos returns Result<OffboardResult, Error>
    // OffboardResult should be directly serializable or easily convertible to bark_cli_json::Offboard
    let offboard_result = w.offboard_vtxos(vtxo_ids, destination_address_opt).await?;

    let json_string = serde_json::to_string_pretty(&offboard_result)
        .context("Failed to serialize offboard status to JSON")?;

    Ok(json_string)
}

/// Offboard all VTXOs. Returns JSON result.
pub async fn offboard_all(
    datadir: &Path,
    mnemonic: Mnemonic,
    destination_address_str: Option<String>, // Optional destination address string
    no_sync: bool,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for offboard_all")?;

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
    // Assuming w.offboard_all returns Result<OffboardResult, Error>
    let offboard_result = w.offboard_all(destination_address_opt).await?;

    let json_string = serde_json::to_string_pretty(&offboard_result)
        .context("Failed to serialize offboard status to JSON")?;

    Ok(json_string)
}

// --- Exit Logic ---

/// Start the exit process for specific VTXOs. Returns simple success JSON.
pub async fn exit_start_specific(
    datadir: &Path,
    mnemonic: Mnemonic,
    vtxo_ids: Vec<VtxoId>,
) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for exit_start_specific")?;

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

    // Fetch Vtxo objects - TODO: Ensure w.vtxos_with is usable or adapt
    // The CLI uses a filter on existing VTXOs. Let's replicate that.
    info!("Fetching specific VTXOs for exit...");
    let filter = VtxoFilter::new(&w).include_many(vtxo_ids.clone()); // Clone ids if needed later
    let vtxos_to_exit = w
        .vtxos_with(filter) // Assuming this syncs or uses cached vtxos
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
    // Assuming w.exit.start_exit_for_vtxos takes &Vec<Vtxo>
    w.exit
        .start_exit_for_vtxos(&vtxos_to_exit, &mut w.onchain)
        .await?;

    // Return simple success JSON
    let success_json = serde_json::json!({ "success": true, "type": "start_specific", "vtxo_count": vtxos_to_exit.len() });
    let json_string = serde_json::to_string_pretty(&success_json)?;
    Ok(json_string)
}

/// Start the exit process for the entire wallet. Returns simple success JSON.
pub async fn exit_start_all(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for exit_start_all")?;

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

    // Return simple success JSON
    let success_json = serde_json::json!({ "success": true, "type": "start_all" });
    let json_string = serde_json::to_string_pretty(&success_json)?;
    Ok(json_string)
}

/// Progress the exit process once. Returns JSON status.
pub async fn exit_progress_once(datadir: &Path, mnemonic: Mnemonic) -> anyhow::Result<String> {
    let mut w = open_wallet(datadir, mnemonic)
        .await
        .context("error opening wallet for exit_progress_once")?;

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
    // Assuming w.exit.progress_exit updates state and handles tx broadcasting etc.
    w.exit
        .progress_exit(&mut w.onchain)
        .await
        .context("Error making progress on exit process")?;

    // Check status after progressing
    let pending_exits = w.exit.list_pending_exits().await?; // Assuming this lists ongoing exits
    let done = pending_exits.is_empty();
    let height = w.exit.all_spendable_at_height().await; // Assuming returns Option<BlockHeight> or similar

    info!(
        "Exit progress check: Done={}, Spendable Height={:?}",
        done, height
    );

    let exit_status = ExitStatus { done, height };

    let json_string = serde_json::to_string_pretty(&exit_status)
        .context("Failed to serialize exit status to JSON")?;
    Ok(json_string)
}
