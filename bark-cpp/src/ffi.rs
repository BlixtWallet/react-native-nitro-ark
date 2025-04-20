use crate::utils::ConfigOpts;

use super::*;
use bark::ark::{bitcoin, VtxoId};
use bip39::Mnemonic;
use logger::log::error;
use logger::Logger;
use std::ffi::{c_char, CStr, CString};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{ptr, slice};

#[repr(C)]
pub struct BarkError {
    message: *mut c_char,
}

impl BarkError {
    fn new(msg: &str) -> Self {
        debug!("Creating BarkError: {}", msg);
        let message = CString::new(msg).unwrap_or_default().into_raw();
        BarkError { message }
    }
}

#[repr(C)]
pub struct BarkConfigOpts {
    asp: *const c_char,
    esplora: *const c_char,
    bitcoind: *const c_char,
    bitcoind_cookie: *const c_char,
    bitcoind_user: *const c_char,
    bitcoind_pass: *const c_char,
}

#[repr(C)]
pub struct BarkCreateOpts {
    force: bool,
    regtest: bool,
    signet: bool,
    bitcoin: bool,
    mnemonic: *const c_char,
    birthday_height: u64,
    config: BarkConfigOpts,
}

#[repr(C)]
pub struct BarkBalance {
    onchain: u64,
    offchain: u64,
    pending_exit: u64,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
#[repr(C)]
pub enum BarkRefreshModeType {
    DefaultThreshold,
    ThresholdBlocks,
    ThresholdHours,
    Counterparty,
    All,
    Specific,
}

// Structure to pass refresh parameters from C
#[repr(C)]
pub struct BarkRefreshOpts {
    mode_type: BarkRefreshModeType,
    // Value used for ThresholdBlocks/ThresholdHours (or ignored)
    threshold_value: u32,
    // Array of VtxoId strings, only used if mode_type is Specific
    specific_vtxo_ids: *const *const c_char,
    num_specific_vtxo_ids: usize,
}

// Helper to convert C opts to Rust RefreshMode
fn convert_refresh_opts(opts: &BarkRefreshOpts) -> anyhow::Result<RefreshMode> {
    match opts.mode_type {
        BarkRefreshModeType::DefaultThreshold => Ok(RefreshMode::DefaultThreshold),
        BarkRefreshModeType::ThresholdBlocks => {
            Ok(RefreshMode::ThresholdBlocks(opts.threshold_value))
        }
        BarkRefreshModeType::ThresholdHours => {
            Ok(RefreshMode::ThresholdHours(opts.threshold_value))
        }
        BarkRefreshModeType::Counterparty => Ok(RefreshMode::Counterparty),
        BarkRefreshModeType::All => Ok(RefreshMode::All),
        BarkRefreshModeType::Specific => {
            if opts.specific_vtxo_ids.is_null() {
                bail!("specific_vtxo_ids pointer is null for Specific refresh mode");
            }
            if opts.num_specific_vtxo_ids == 0 {
                // Allow zero IDs to be passed, `refresh_vtxos_internal` handles this.
                debug!("num_specific_vtxo_ids is 0 for Specific refresh mode.");
                Ok(RefreshMode::Specific(Vec::new()))
            } else {
                let mut vtxo_ids = Vec::with_capacity(opts.num_specific_vtxo_ids);
                unsafe {
                    let id_slice =
                        slice::from_raw_parts(opts.specific_vtxo_ids, opts.num_specific_vtxo_ids);
                    for (i, &c_str_ptr) in id_slice.iter().enumerate() {
                        if c_str_ptr.is_null() {
                            bail!("Specific VTXO ID at index {} is null", i);
                        }
                        let id_str = CStr::from_ptr(c_str_ptr).to_str().with_context(|| {
                            format!("Specific VTXO ID at index {} is not valid UTF-8", i)
                        })?;
                        let vtxo_id = VtxoId::from_str(id_str).with_context(|| {
                            format!(
                                "Specific VTXO ID '{}' at index {} is not a valid VtxoId",
                                id_str, i
                            )
                        })?;
                        vtxo_ids.push(vtxo_id);
                    }
                }
                Ok(RefreshMode::Specific(vtxo_ids))
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn bark_free_error(error: *mut BarkError) {
    if !error.is_null() {
        debug!("Freeing BarkError");
        unsafe {
            let err = Box::from_raw(error);
            if !err.message.is_null() {
                let _ = CString::from_raw(err.message);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn bark_error_message(error: *const BarkError) -> *const c_char {
    if error.is_null() {
        warn!("Attempted to get message from null error");
        return ptr::null();
    }
    unsafe { (*error).message }
}

fn to_rust_config_opts(c_opts: &BarkConfigOpts) -> ConfigOpts {
    debug!("Converting C config opts to Rust");

    let asp = c_string_to_option(c_opts.asp);
    let esplora = c_string_to_option(c_opts.esplora);
    let bitcoind = c_string_to_option(c_opts.bitcoind);
    let bitcoind_cookie = c_string_to_option(c_opts.bitcoind_cookie);
    let bitcoind_user = c_string_to_option(c_opts.bitcoind_user);
    let bitcoind_pass = c_string_to_option(c_opts.bitcoind_pass);

    // Log configuration (without sensitive data)
    debug!("Config - ASP: {}", asp.is_some());
    debug!("Config - Esplora: {}", esplora.is_some());
    debug!("Config - Bitcoind: {}", bitcoind.is_some());

    ConfigOpts {
        asp,
        esplora,
        bitcoind,
        bitcoind_cookie,
        bitcoind_user,
        bitcoind_pass,
    }
}

fn c_string_to_option(s: *const c_char) -> Option<String> {
    if s.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(s).to_str() {
                Ok(str) => {
                    let result = if !str.is_empty() {
                        Some(String::from(str))
                    } else {
                        None
                    };
                    result
                }
                Err(e) => {
                    warn!("Failed to convert C string: {}", e);
                    None
                }
            }
        }
    }
}

fn to_rust_create_opts(c_opts: &BarkCreateOpts) -> anyhow::Result<CreateOpts> {
    debug!("Converting C create opts to Rust");
    debug!(
        "Create opts - Force: {}, Regtest: {}, Signet: {}, Bitcoin: {}",
        c_opts.force, c_opts.regtest, c_opts.signet, c_opts.bitcoin
    );
    debug!("Create opts - Birthday height: {}", c_opts.birthday_height);

    let mnemonic = if c_opts.mnemonic.is_null() {
        debug!("No mnemonic provided");
        bail!("No mnemonic provided");
    } else {
        let mnemonic_str = unsafe { CStr::from_ptr(c_opts.mnemonic).to_str()? };
        if !mnemonic_str.is_empty() {
            debug!("Converting provided mnemonic");
            Mnemonic::from_str(mnemonic_str)?
        } else {
            debug!("Empty mnemonic string provided");
            bail!("Empty mnemonic string provided");
        }
    };

    let birthday_height = if c_opts.birthday_height > 0 {
        Some(c_opts.birthday_height)
    } else {
        None
    };

    Ok(CreateOpts {
        force: c_opts.force,
        regtest: c_opts.regtest,
        signet: c_opts.signet,
        bitcoin: c_opts.bitcoin,
        mnemonic,
        birthday_height,
        config: to_rust_config_opts(&c_opts.config),
    })
}

/// Create a new mnemonic
///
/// @return The mnemonic string as a C string, or NULL on error
#[no_mangle]
pub extern "C" fn bark_create_mnemonic() -> *mut c_char {
    let _logger = Logger::new();
    debug!("bark_create_mnemonic called");

    let mnemonic = match create_mnemonic() {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to create mnemonic: {}", e);
            return ptr::null_mut();
        }
    };

    match CString::new(mnemonic) {
        Ok(c_string) => c_string.into_raw(),
        Err(e) => {
            error!("Failed to convert mnemonic to CString: {}", e);
            ptr::null_mut()
        }
    }
}

/// Create a new wallet at the specified directory
///
/// @param datadir Path to the data directory
/// @param opts Creation options
/// @return Error pointer or NULL on success
#[no_mangle]
pub extern "C" fn bark_create_wallet(
    datadir: *const c_char,
    opts: BarkCreateOpts,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_create_wallet called datadir={:?}, opts: force={}, regtest={}, signet={}, bitcoin={}, birthday_height={}, asp={:?}, esplora={:?}",
    datadir,
    opts.force,
    opts.regtest,
    opts.signet,
    opts.bitcoin,
    opts.birthday_height,
    if opts.config.asp.is_null() { "null" } else { unsafe { CStr::from_ptr(opts.config.asp).to_str().unwrap_or("invalid") } },
    if opts.config.esplora.is_null() { "null" } else { unsafe { CStr::from_ptr(opts.config.esplora).to_str().unwrap_or("invalid") } }
);
    if datadir.is_null() {
        error!("Data directory pointer is null");
        return Box::into_raw(Box::new(BarkError::new("datadir is null")));
    }

    let datadir_str = match c_string_to_string(datadir) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert datadir string: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    let create_opts = match to_rust_create_opts(&opts) {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to convert create options: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    // Create a new runtime for the async function
    debug!("Creating tokio runtime");
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    // Run the async function
    debug!("Running create_wallet async function");
    let result = runtime
        .block_on(async { create_wallet(Path::new(datadir_str.as_str()), create_opts).await });

    match result {
        Ok(_) => {
            debug!("Wallet created successfully");
            ptr::null_mut()
        }
        Err(e) => {
            error!("Failed to create wallet: {}", e);
            Box::into_raw(Box::new(BarkError::new(&e.to_string())))
        }
    }
}

/// Get offchain and onchain balances
///
/// @param datadir Path to the data directory
/// @param no_sync Whether to skip syncing the wallet
/// @param balance_out Pointer to a BarkBalance struct where the result will be stored
/// @return Error pointer or NULL on success
#[no_mangle]
pub extern "C" fn bark_get_balance(
    datadir: *const c_char,
    no_sync: bool,
    mnemonic: *const c_char,
    balance_out: *mut BarkBalance,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_get_balance called, no_sync: {}", no_sync);

    let datadir_str = match c_string_to_string(datadir) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert datadir string: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    let mnemonic_str = match c_string_to_string(mnemonic) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to convert mnemonic string: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    if balance_out.is_null() {
        error!("Balance output pointer is null");
        return Box::into_raw(Box::new(BarkError::new("balance_out is null")));
    }

    // Create a new runtime for the async function
    debug!("Creating tokio runtime");
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&e.to_string())));
        }
    };

    // Run the async function
    debug!("Running get_balance async function");

    let mnemonic = Mnemonic::from_str(mnemonic_str.as_str()).unwrap();
    let result = runtime
        .block_on(async { get_balance(Path::new(datadir_str.as_str()), no_sync, mnemonic).await });

    match result {
        Ok(balance) => {
            // Store the result in the output parameter
            unsafe {
                (*balance_out).onchain = balance.onchain;
                (*balance_out).offchain = balance.offchain;
                (*balance_out).pending_exit = balance.pending_exit;
            }
            debug!(
                "Balance retrieved successfully: onchain={}, offchain={}, pending_exit={}",
                balance.onchain, balance.offchain, balance.pending_exit
            );
            ptr::null_mut()
        }
        Err(e) => {
            error!("Failed to get balance: {}", e);
            Box::into_raw(Box::new(BarkError::new(&e.to_string())))
        }
    }
}

/// Get an onchain address.
///
/// The returned address string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param address_out Pointer to a `*mut c_char` where the address string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_get_onchain_address(
    datadir: *const c_char,
    mnemonic: *const c_char,
    address_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_get_onchain_address called");

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || address_out.is_null() {
        error!("Null pointer passed to bark_get_onchain_address (datadir={}, mnemonic={}, address_out={})",
             datadir.is_null(), mnemonic.is_null(), address_out.is_null());
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    // Initialize output pointer to null
    unsafe {
        *address_out = ptr::null_mut();
    }

    // --- Conversions ---
    let datadir_str = match c_string_to_string(datadir) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!("Invalid datadir: {}", e))))
        }
    };
    let datadir_path = Path::new(&datadir_str);

    let mnemonic_str = match c_string_to_string(mnemonic) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid mnemonic: {}",
                e
            ))))
        }
    };
    let rust_mnemonic = match Mnemonic::from_str(&mnemonic_str) {
        Ok(m) => m,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to parse mnemonic: {}",
                e
            ))))
        }
    };

    // --- Runtime and Async Execution ---
    debug!("Creating tokio runtime for get_onchain_address");
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    debug!("Running get_onchain_address async function");
    let result = runtime.block_on(async { get_onchain_address(datadir_path, rust_mnemonic).await });

    // --- Result Handling ---
    match result {
        Ok(address) => {
            debug!("Address retrieved successfully: {}", address);
            let address_string = address.to_string();
            match CString::new(address_string) {
                Ok(c_string) => {
                    unsafe {
                        *address_out = c_string.into_raw();
                    }
                    debug!("Successfully prepared address C string for return.");
                    ptr::null_mut() // Success
                }
                Err(e) => {
                    error!("Failed to create CString for address: {}", e);
                    Box::into_raw(Box::new(BarkError::new(
                        "Failed to convert address to C string",
                    )))
                }
            }
        }
        Err(e) => {
            error!("Failed to get onchain address: {}", e);
            error!("Get Address Error Details: {:?}", e);
            Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to get address: {}",
                e
            ))))
        }
    }
}

/// Send funds using the onchain wallet.
///
/// The returned transaction ID string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param amount_sat The amount to send in satoshis
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_send_onchain(
    datadir: *const c_char,
    mnemonic: *const c_char,
    destination: *const c_char,
    amount_sat: u64,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_send_onchain called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || destination.is_null() || txid_out.is_null() {
        error!("Null pointer passed to bark_send_onchain (datadir={}, mnemonic={}, destination={}, txid_out={})",
             datadir.is_null(), mnemonic.is_null(), destination.is_null(), txid_out.is_null());
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    // Initialize output pointer to null
    unsafe {
        *txid_out = ptr::null_mut();
    }

    // --- Conversions ---
    let datadir_str = match c_string_to_string(datadir) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!("Invalid datadir: {}", e))))
        }
    };
    let datadir_path = Path::new(&datadir_str);

    let mnemonic_str = match c_string_to_string(mnemonic) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid mnemonic: {}",
                e
            ))))
        }
    };
    let rust_mnemonic = match Mnemonic::from_str(&mnemonic_str) {
        Ok(m) => m,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to parse mnemonic: {}",
                e
            ))))
        }
    };

    let destination_str = match c_string_to_string(destination) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid destination address: {}",
                e
            ))))
        }
    };
    debug!("Destination address string: {}", destination_str);

    let amount = Amount::from_sat(amount_sat);
    debug!("Amount: {}", amount);

    // --- Runtime and Async Execution ---
    debug!("Creating tokio runtime for send_onchain");
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    debug!("Running send_onchain async function");
    // Pass destination_str, validation happens inside send_onchain
    let result = runtime.block_on(async {
        send_onchain(
            datadir_path,
            rust_mnemonic,
            &destination_str,
            amount,
            no_sync,
        )
        .await
    });

    // --- Result Handling ---
    match result {
        Ok(txid) => {
            debug!("Send successful, TxID: {}", txid);
            let txid_string = txid.to_string();
            match CString::new(txid_string) {
                Ok(c_string) => {
                    unsafe {
                        *txid_out = c_string.into_raw();
                    }
                    debug!("Successfully prepared txid C string for return.");
                    ptr::null_mut() // Success
                }
                Err(e) => {
                    error!("Failed to create CString for txid: {}", e);
                    Box::into_raw(Box::new(BarkError::new(
                        "Failed to convert txid to C string",
                    )))
                }
            }
        }
        Err(e) => {
            error!("Failed to send onchain: {}", e);
            error!("Send Onchain Error Details: {:?}", e);
            // Provide more context in the error message if possible
            Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to send onchain: {}",
                e
            ))))
        }
    }
}

/// Send all funds from the onchain wallet to a destination address.
///
/// The returned transaction ID string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_drain_onchain(
    datadir: *const c_char,
    mnemonic: *const c_char,
    destination: *const c_char,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_drain_onchain called: no_sync={}", no_sync);

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || destination.is_null() || txid_out.is_null() {
        error!("Null pointer passed to bark_drain_onchain");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *txid_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_str = match c_string_to_string(datadir) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!("Invalid datadir: {}", e))))
        }
    };
    let datadir_path = Path::new(&datadir_str);

    let mnemonic_str = match c_string_to_string(mnemonic) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid mnemonic: {}",
                e
            ))))
        }
    };
    let rust_mnemonic = match Mnemonic::from_str(&mnemonic_str) {
        Ok(m) => m,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to parse mnemonic: {}",
                e
            ))))
        }
    };

    let destination_str = match c_string_to_string(destination) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid destination address: {}",
                e
            ))))
        }
    };
    debug!("Drain destination address string: {}", destination_str);

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        drain_onchain(datadir_path, rust_mnemonic, &destination_str, no_sync).await
    });

    // --- Result Handling ---
    // Use the new helper function
    handle_txid_result(result, txid_out, "drain")
}

/// Send funds to multiple recipients using the onchain wallet.
///
/// The returned transaction ID string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destinations Array of C strings representing destination Bitcoin addresses
/// @param amounts_sat Array of u64 representing amounts in satoshis (must match destinations array length)
/// @param num_outputs The number of outputs (length of the destinations and amounts_sat arrays)
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_send_many_onchain(
    datadir: *const c_char,
    mnemonic: *const c_char,
    destinations: *const *const c_char,
    amounts_sat: *const u64,
    num_outputs: usize,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_send_many_onchain called: num_outputs={}, no_sync={}",
        num_outputs, no_sync
    );

    // --- Input Validation ---
    if datadir.is_null()
        || mnemonic.is_null()
        || destinations.is_null()
        || amounts_sat.is_null()
        || txid_out.is_null()
        || num_outputs == 0
    {
        error!("Null pointer or zero outputs passed to bark_send_many_onchain");
        return Box::into_raw(Box::new(BarkError::new(
            "Null pointer or zero outputs provided",
        )));
    }
    unsafe {
        *txid_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions & Core Logic ---
    // This part needs to be inside the async block or use block_on carefully
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        // Perform conversions that need the wallet (like network checking) inside async
        let datadir_str = c_string_to_string(datadir)?;
        let mnemonic_str = c_string_to_string(mnemonic)?;
        let rust_mnemonic = Mnemonic::from_str(&mnemonic_str)?;

        // Open the wallet just to get the network for validation
        let net = {
            let w = open_wallet(Path::new(&datadir_str), rust_mnemonic.clone())
                .await
                .context("Failed to open wallet to determine network for send_many")?;
            w.properties()?.network
            // Wallet `w` is dropped here
        };

        // Convert C arrays to Rust Vec<(Address, Amount)> *with network validation*
        let outputs_vec = convert_outputs(destinations, amounts_sat, num_outputs, net)?;

        // Call the actual send_many logic (will re-open wallet internally)
        send_many_onchain(Path::new(&datadir_str), rust_mnemonic, outputs_vec, no_sync).await
    });

    // --- Result Handling ---
    // Use the new helper function
    handle_txid_result(result, txid_out, "send_many")
}

// Helper function to convert C arrays to Rust Vec<(Address, Amount)> and validate network
fn convert_outputs(
    destinations: *const *const c_char,
    amounts_sat: *const u64,
    num_outputs: usize,
    net: Network, // Network needed for validation
) -> anyhow::Result<Vec<(Address, Amount)>> {
    debug!(
        "Converting {} C outputs to Rust Vec<(Address, Amount)> for network {}",
        num_outputs, net
    );
    let mut outputs = Vec::with_capacity(num_outputs);

    // Unsafe block to read C arrays
    unsafe {
        // Create slices from the raw pointers
        let dest_slice = slice::from_raw_parts(destinations, num_outputs);
        let amount_slice = slice::from_raw_parts(amounts_sat, num_outputs);

        for i in 0..num_outputs {
            if dest_slice[i].is_null() {
                bail!("Output {} has a null address pointer", i);
            }

            // Convert C string address to Rust string
            let dest_str = CStr::from_ptr(dest_slice[i])
                .to_str()
                .with_context(|| format!("Output {} address is not valid UTF-8", i))?;
            if dest_str.is_empty() {
                bail!("Output {} address string is empty", i);
            }

            // Parse address and validate network
            let addr_unchecked = Address::<bitcoin::address::NetworkUnchecked>::from_str(dest_str)
                .with_context(|| {
                    format!("Output {} address '{}' is invalid format", i, dest_str)
                })?;
            let addr = addr_unchecked.require_network(net).with_context(|| {
                format!(
                    "Output {} address '{}' is not valid for network {}",
                    i, dest_str, net
                )
            })?;

            // Create Amount from satoshis
            let amount = Amount::from_sat(amount_slice[i]);
            if amount <= Amount::ZERO {
                bail!(
                    "Output {} amount must be positive (got {} sats)",
                    i,
                    amount.to_sat()
                );
            }

            debug!(
                "Converted output {}: Address={}, Amount={}",
                i, addr, amount
            );
            outputs.push((addr, amount));
        }
    }

    Ok(outputs)
}

// Helper to handle Txid result and C string conversion for FFI functions
fn handle_txid_result(
    result: anyhow::Result<Txid>,
    txid_out: *mut *mut c_char,
    operation: &str, // e.g., "send", "drain", "send_many"
) -> *mut BarkError {
    match result {
        Ok(txid) => {
            debug!("Onchain {} successful, TxID: {}", operation, txid);
            let txid_string = txid.to_string();
            match CString::new(txid_string) {
                Ok(c_string) => {
                    unsafe {
                        // Transfer ownership of the CString's buffer to C
                        *txid_out = c_string.into_raw();
                    }
                    debug!("Successfully prepared txid C string for return.");
                    ptr::null_mut() // Success
                }
                Err(e) => {
                    error!("Failed to create CString for {} txid: {}", operation, e);
                    Box::into_raw(Box::new(BarkError::new(&format!(
                        "Failed to convert {} txid to C string",
                        operation
                    ))))
                }
            }
        }
        Err(e) => {
            error!("Failed to {}: {}", operation, e);
            // Log the detailed error chain if possible
            error!("{} Error Details: {:?}", operation, e);
            Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to {}: {}",
                operation, e
            ))))
        }
    }
}

fn handle_string_result(
    result: anyhow::Result<String>,
    string_out: *mut *mut c_char,
    operation: &str, // e.g., "get_utxos", "get_pubkey"
) -> *mut BarkError {
    match result {
        Ok(value_string) => {
            if value_string.is_empty() {
                debug!("{} operation returned an empty string.", operation);
                // Decide if empty string is valid or an error case depending on the operation
                // For JSON/Pubkey, let's return it as success.
            } else {
                debug!(
                    "{} successful, String length: {}",
                    operation,
                    value_string.len()
                );
            }

            match CString::new(value_string) {
                Ok(c_string) => {
                    unsafe {
                        *string_out = c_string.into_raw();
                    }
                    debug!("Successfully prepared {} C string for return.", operation);
                    ptr::null_mut() // Success
                }
                Err(e) => {
                    error!("Failed to create CString for {}: {}", operation, e);
                    Box::into_raw(Box::new(BarkError::new(&format!(
                        "Failed to convert {} result to C string",
                        operation
                    ))))
                }
            }
        }
        Err(e) => {
            error!("Failed to {}: {}", operation, e);
            error!("{} Error Details: {:?}", operation, e);
            Box::into_raw(Box::new(BarkError::new(&format!(
                "Failed to {}: {}",
                operation, e
            ))))
        }
    }
}

/// Get the list of onchain UTXOs as a JSON string.
///
/// The returned JSON string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the wallet before fetching
/// @param utxos_json_out Pointer to a `*mut c_char` where the JSON string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_get_onchain_utxos(
    datadir: *const c_char,
    mnemonic: *const c_char,
    no_sync: bool,
    utxos_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_get_onchain_utxos called: no_sync={}", no_sync);

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || utxos_json_out.is_null() {
        error!("Null pointer passed to bark_get_onchain_utxos");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *utxos_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result =
        runtime.block_on(async { get_onchain_utxos(&datadir_path, rust_mnemonic, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, utxos_json_out, "get_onchain_utxos")
}

/// Get the wallet's VTXO public key (hex string).
///
/// The returned public key string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param pubkey_hex_out Pointer to a `*mut c_char` where the hex string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_get_vtxo_pubkey(
    datadir: *const c_char,
    mnemonic: *const c_char,
    pubkey_hex_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_get_vtxo_pubkey called");

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || pubkey_hex_out.is_null() {
        error!("Null pointer passed to bark_get_vtxo_pubkey");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *pubkey_hex_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    // `get_vtxo_pubkey` is async because `open_wallet` is async
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async { get_vtxo_pubkey(&datadir_path, rust_mnemonic).await });

    // --- Result Handling ---
    handle_string_result(result, pubkey_hex_out, "get_vtxo_pubkey")
}

/// Get the list of VTXOs as a JSON string.
///
/// The returned JSON string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the wallet before fetching
/// @param vtxos_json_out Pointer to a `*mut c_char` where the JSON string pointer will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_get_vtxos(
    datadir: *const c_char,
    mnemonic: *const c_char,
    no_sync: bool,
    vtxos_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_get_vtxos called: no_sync={}", no_sync);

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || vtxos_json_out.is_null() {
        error!("Null pointer passed to bark_get_vtxos");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *vtxos_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async { get_vtxos(&datadir_path, rust_mnemonic, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, vtxos_json_out, "get_vtxos")
}

/// Refresh VTXOs based on specified criteria.
///
/// The returned JSON status string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param refresh_opts Options specifying which VTXOs to refresh
/// @param no_sync Whether to skip syncing the wallet before refreshing
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_refresh_vtxos(
    datadir: *const c_char,
    mnemonic: *const c_char,
    refresh_opts: BarkRefreshOpts, // Pass struct by value
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_refresh_vtxos called: mode={:?}, threshold={}, num_specific={}, no_sync={}",
        refresh_opts.mode_type,
        refresh_opts.threshold_value,
        refresh_opts.num_specific_vtxo_ids,
        no_sync
    ); // Use Debug on enum if derived

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_refresh_vtxos");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    if refresh_opts.mode_type == BarkRefreshModeType::Specific
        && refresh_opts.num_specific_vtxo_ids > 0
        && refresh_opts.specific_vtxo_ids.is_null()
    {
        error!("Specific mode selected but specific_vtxo_ids pointer is null");
        return Box::into_raw(Box::new(BarkError::new(
            "Null specific_vtxo_ids pointer for Specific mode",
        )));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mode = match convert_refresh_opts(&refresh_opts) {
        Ok(mode) => mode,
        Err(e) => {
            error!("Failed to convert refresh options: {}", e);
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid refresh options: {}",
                e
            ))));
        }
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime
        .block_on(async { refresh_vtxos(&datadir_path, rust_mnemonic, rust_mode, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "refresh_vtxos")
}

// --- Board FFI ---

/// Board a specific amount from the onchain wallet into Ark.
///
/// The returned JSON status string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param amount_sat The amount in satoshis to board
/// @param no_sync Whether to skip syncing the onchain wallet before boarding
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_board_amount(
    datadir: *const c_char,
    mnemonic: *const c_char,
    amount_sat: u64,
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_board_amount called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_board_amount");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    if amount_sat == 0 {
        error!("Board amount cannot be zero");
        return Box::into_raw(Box::new(BarkError::new("Board amount cannot be zero")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let amount = Amount::from_sat(amount_sat);

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime
        .block_on(async { board_amount(&datadir_path, rust_mnemonic, amount, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "board_amount")
}

/// Board all available funds from the onchain wallet into Ark.
///
/// The returned JSON status string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the onchain wallet before boarding
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_board_all(
    datadir: *const c_char,
    mnemonic: *const c_char,
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_board_all called: no_sync={}", no_sync);

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_board_all");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async { board_all(&datadir_path, rust_mnemonic, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "board_all")
}

#[no_mangle]
pub extern "C" fn bark_send(
    datadir: *const c_char,
    mnemonic: *const c_char,
    destination: *const c_char,
    amount_sat: u64,        // Use 0 or ULLONG_MAX to indicate 'not provided by user'
    comment: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    // Use a sentinel value like u64::MAX to clearly indicate user did not provide amount
    const AMOUNT_NOT_PROVIDED: u64 = u64::MAX;
    let amount_provided = amount_sat != AMOUNT_NOT_PROVIDED;
    debug!(
        "bark_send called: amount_sat={}, amount_provided={}, no_sync={}",
        if amount_provided {
            amount_sat.to_string()
        } else {
            "NotProvided".to_string()
        },
        amount_provided,
        no_sync
    );

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || destination.is_null() || status_json_out.is_null()
    {
        error!("Null pointer passed to bark_send");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let destination_str = match c_string_to_string(destination) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid destination: {}",
                e
            ))))
        }
    };
    let rust_amount_opt: Option<u64> = if amount_provided {
        Some(amount_sat)
    } else {
        None
    };
    let rust_comment_opt: Option<String> = c_string_to_option(comment);

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        send_payment(
            &datadir_path,
            rust_mnemonic,
            &destination_str,
            rust_amount_opt,
            rust_comment_opt,
            no_sync,
        )
        .await
    });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "send_payment")
}

// --- Send Round Onchain FFI ---

/// Send an onchain payment via an Ark round.
///
/// The returned JSON status string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param amount_sat The amount in satoshis to send
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_send_round_onchain(
    datadir: *const c_char,
    mnemonic: *const c_char,
    destination: *const c_char,
    amount_sat: u64,
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_send_round_onchain called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || destination.is_null() || status_json_out.is_null()
    {
        error!("Null pointer passed to bark_send_round_onchain");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    if amount_sat == 0 {
        error!("Send round onchain amount cannot be zero");
        return Box::into_raw(Box::new(BarkError::new("Amount cannot be zero")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let destination_str = match c_string_to_string(destination) {
        Ok(s) => s,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid destination address: {}",
                e
            ))))
        }
    };
    let amount = Amount::from_sat(amount_sat);

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        send_round_onchain(
            &datadir_path,
            rust_mnemonic,
            &destination_str,
            amount,
            no_sync,
        )
        .await
    });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "send_round_onchain")
}

// --- Offboard FFI ---

/// Offboard specific VTXOs to an optional onchain address.
///
/// The returned JSON result string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param specific_vtxo_ids Array of VtxoId strings (cannot be empty)
/// @param num_specific_vtxo_ids Number of VtxoIds in the array
/// @param optional_address Optional destination Bitcoin address (pass NULL if not provided)
/// @param no_sync Whether to skip syncing the wallet
/// @param status_json_out Pointer to a `*mut c_char` where the JSON result string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_offboard_specific(
    datadir: *const c_char,
    mnemonic: *const c_char,
    specific_vtxo_ids: *const *const c_char,
    num_specific_vtxo_ids: usize,
    optional_address: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_offboard_specific called: num_vtxos={}, no_sync={}",
        num_specific_vtxo_ids, no_sync
    );

    // --- Input Validation ---
    if datadir.is_null()
        || mnemonic.is_null()
        || specific_vtxo_ids.is_null()
        || status_json_out.is_null()
    {
        error!("Null pointer passed to bark_offboard_specific (excluding optional_address)");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    if num_specific_vtxo_ids == 0 {
        error!("Must provide at least one VTXO ID for specific offboarding");
        return Box::into_raw(Box::new(BarkError::new("No VTXO IDs provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_address_opt = c_string_to_option(optional_address);

    // Convert VTXO ID strings
    let rust_vtxo_ids = match convert_vtxo_ids(specific_vtxo_ids, num_specific_vtxo_ids) {
        Ok(ids) => ids,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid VTXO IDs: {}",
                e
            ))))
        }
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        offboard_specific(
            &datadir_path,
            rust_mnemonic,
            rust_vtxo_ids,
            rust_address_opt,
            no_sync,
        )
        .await
    });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "offboard_specific")
}

/// Offboard all VTXOs to an optional onchain address.
///
/// The returned JSON result string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param optional_address Optional destination Bitcoin address (pass NULL if not provided)
/// @param no_sync Whether to skip syncing the wallet
/// @param status_json_out Pointer to a `*mut c_char` where the JSON result string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_offboard_all(
    datadir: *const c_char,
    mnemonic: *const c_char,
    optional_address: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_offboard_all called: no_sync={}", no_sync);

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_offboard_all (excluding optional_address)");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_address_opt = c_string_to_option(optional_address);

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async {
        offboard_all(&datadir_path, rust_mnemonic, rust_address_opt, no_sync).await
    });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "offboard_all")
}

// --- Exit FFI ---

/// Start the exit process for specific VTXOs.
///
/// The returned JSON success string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param specific_vtxo_ids Array of VtxoId strings (cannot be empty)
/// @param num_specific_vtxo_ids Number of VtxoIds in the array
/// @param status_json_out Pointer to a `*mut c_char` where the JSON success string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_exit_start_specific(
    datadir: *const c_char,
    mnemonic: *const c_char,
    specific_vtxo_ids: *const *const c_char,
    num_specific_vtxo_ids: usize,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!(
        "bark_exit_start_specific called: num_vtxos={}",
        num_specific_vtxo_ids
    );

    // --- Input Validation ---
    if datadir.is_null()
        || mnemonic.is_null()
        || specific_vtxo_ids.is_null()
        || status_json_out.is_null()
    {
        error!("Null pointer passed to bark_exit_start_specific");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    if num_specific_vtxo_ids == 0 {
        error!("Must provide at least one VTXO ID for starting specific exit");
        return Box::into_raw(Box::new(BarkError::new("No VTXO IDs provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    // Convert VTXO ID strings
    let rust_vtxo_ids = match convert_vtxo_ids(specific_vtxo_ids, num_specific_vtxo_ids) {
        Ok(ids) => ids,
        Err(e) => {
            return Box::into_raw(Box::new(BarkError::new(&format!(
                "Invalid VTXO IDs: {}",
                e
            ))))
        }
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime
        .block_on(async { exit_start_specific(&datadir_path, rust_mnemonic, rust_vtxo_ids).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_start_specific")
}

/// Start the exit process for all VTXOs in the wallet.
///
/// The returned JSON success string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param status_json_out Pointer to a `*mut c_char` where the JSON success string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_exit_start_all(
    datadir: *const c_char,
    mnemonic: *const c_char,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_exit_start_all called");

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_exit_start_all");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async { exit_start_all(&datadir_path, rust_mnemonic).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_start_all")
}

/// Progress the exit process once and return the current status.
///
/// The returned JSON status string must be freed by the caller using `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status string will be written.
/// @return Error pointer or NULL on success.
#[no_mangle]
pub extern "C" fn bark_exit_progress_once(
    datadir: *const c_char,
    mnemonic: *const c_char,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    let _logger = Logger::new();
    debug!("bark_exit_progress_once called");

    // --- Input Validation ---
    if datadir.is_null() || mnemonic.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_exit_progress_once");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let datadir_path = match c_string_to_path(datadir) {
        Ok(p) => p,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };
    let rust_mnemonic = match c_string_to_mnemonic(mnemonic) {
        Ok(m) => m,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&format!("Runtime error: {}", e)))),
    };

    let result = runtime.block_on(async { exit_progress_once(&datadir_path, rust_mnemonic).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_progress_once")
}

// --- Add Helper Conversion Functions (if they don't exist) ---

// Helper to convert C string to PathBuf
fn c_string_to_path(s: *const c_char) -> anyhow::Result<PathBuf> {
    if s.is_null() {
        bail!("C path string pointer is null");
    }
    let path_str = unsafe { CStr::from_ptr(s) }
        .to_str()
        .context("Failed to convert C path string to UTF-8")?;
    if path_str.is_empty() {
        bail!("Path string is empty");
    }
    Ok(PathBuf::from(path_str))
}

// Helper to convert C string to Mnemonic
fn c_string_to_mnemonic(s: *const c_char) -> anyhow::Result<Mnemonic> {
    if s.is_null() {
        bail!("C mnemonic string pointer is null");
    }
    let mnemonic_str = unsafe { CStr::from_ptr(s) }
        .to_str()
        .context("Failed to convert C mnemonic string to UTF-8")?;
    if mnemonic_str.is_empty() {
        bail!("Mnemonic string is empty");
    }
    Mnemonic::from_str(mnemonic_str).context("Invalid mnemonic format")
}

// Extract string from C string
fn c_string_to_string(s: *const c_char) -> anyhow::Result<String> {
    if s.is_null() {
        bail!("C string is null");
    }

    let s = unsafe { CStr::from_ptr(s).to_str()? };
    Ok(s.to_string())
}

// Helper to convert C array of C strings to Vec<VtxoId>
fn convert_vtxo_ids(
    ids_array: *const *const c_char,
    num_ids: usize,
) -> anyhow::Result<Vec<VtxoId>> {
    if ids_array.is_null() {
        bail!("VTXO IDs array pointer is null");
    }
    let mut vtxo_ids = Vec::with_capacity(num_ids);
    unsafe {
        let id_slice = slice::from_raw_parts(ids_array, num_ids);
        for (i, &c_str_ptr) in id_slice.iter().enumerate() {
            if c_str_ptr.is_null() {
                bail!("VTXO ID at index {} is null", i);
            }
            let id_str = CStr::from_ptr(c_str_ptr)
                .to_str()
                .with_context(|| format!("VTXO ID at index {} is not valid UTF-8", i))?;
            let vtxo_id = VtxoId::from_str(id_str).with_context(|| {
                format!("VTXO ID '{}' at index {} is not a valid VtxoId", id_str, i)
            })?;
            vtxo_ids.push(vtxo_id);
        }
    }
    Ok(vtxo_ids)
}
