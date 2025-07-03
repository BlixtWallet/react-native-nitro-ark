use crate::ffi_utils::{
    c_string_to_string, handle_string_result, handle_txid_result, to_rust_create_opts,
};

use super::*;
use bark::ark::bitcoin;
use logger::log::{debug, error, warn};
use once_cell::sync::Lazy;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::str::FromStr;
use std::{ptr, slice};
use tokio::runtime::Runtime;

pub static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

/// Initializes the logger for the library.
/// This should be called once when the library is loaded by the C/C++ application,
/// before any other library functions are used.
#[no_mangle]
pub extern "C" fn bark_init_logger() {
    // This calls the init_logger function from lib.rs,
    // which in turn ensures the static LOGGER is accessed and initialized.
    crate::init_logger();
}

#[repr(C)]
pub struct BarkError {
    message: *mut c_char,
}

impl BarkError {
    pub fn new(msg: &str) -> Self {
        debug!("Creating BarkError: {}", msg);
        let message = CString::new(msg).unwrap_or_default().into_raw();
        BarkError { message }
    }
}

#[repr(C)]
pub struct BarkConfigOpts {
    pub asp: *const c_char,
    pub esplora: *const c_char,
    pub bitcoind: *const c_char,
    pub bitcoind_cookie: *const c_char,
    pub bitcoind_user: *const c_char,
    pub bitcoind_pass: *const c_char,
    pub vtxo_refresh_expiry_threshold: u32,
    pub fallback_fee_rate: *const u64,
}

#[repr(C)]
pub struct BarkCreateOpts {
    pub regtest: bool,
    pub signet: bool,
    pub bitcoin: bool,
    pub mnemonic: *const c_char,
    pub birthday_height: u32,
    pub config: BarkConfigOpts,
}

#[repr(C)]
pub struct BarkBalance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64,
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
    pub mode_type: BarkRefreshModeType,
    // Value used for ThresholdBlocks/ThresholdHours (or ignored)
    pub threshold_value: u32,
    // Array of VtxoId strings, only used if mode_type is Specific
    pub specific_vtxo_ids: *const *const c_char,
    pub num_specific_vtxo_ids: usize,
}

#[no_mangle]
pub extern "C" fn bark_free_error(error: *mut BarkError) {
    if !error.is_null() {
        debug!("Freeing BarkError");
        unsafe {
            let err = Box::from_raw(error);
            if !err.message.is_null() {
                // Free the message string using the new function
                bark_free_string(err.message);
            }
            // Box goes out of scope and frees the BarkError struct itself
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

/// Frees a C string allocated by a bark-cpp function.
#[no_mangle]
pub extern "C" fn bark_free_string(s: *mut c_char) {
    if !s.is_null() {
        debug!("Freeing C string pointer: {:?}", s);
        unsafe {
            // Reconstruct the CString from the raw pointer. This takes ownership back.
            let _ = CString::from_raw(s);
            // When `_` goes out of scope here, the CString is dropped,
            // and its memory is deallocated by Rust's allocator.
        }
        debug!("Freed C string");
    } else {
        debug!("Called bark_free_string with a null pointer, doing nothing.");
    }
}

/// Create a new mnemonic
#[no_mangle]
pub extern "C" fn bark_create_mnemonic() -> *mut c_char {
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

/// Load an existing wallet or create a new one at the specified directory
#[no_mangle]
pub extern "C" fn bark_load_wallet(datadir: *const c_char, opts: BarkCreateOpts) -> *mut BarkError {
    debug!("bark_load_wallet called");
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

    // Run the async function
    debug!("Running load_wallet async function");
    let result = TOKIO_RUNTIME
        .block_on(async { load_wallet(Path::new(datadir_str.as_str()), create_opts).await });

    match result {
        Ok(_) => {
            debug!("Wallet loaded successfully");
            ptr::null_mut()
        }
        Err(e) => {
            error!("Failed to load wallet: {}", e);
            Box::into_raw(Box::new(BarkError::new(&e.to_string())))
        }
    }
}

/// Close the currently loaded wallet
#[no_mangle]
pub extern "C" fn bark_close_wallet() -> *mut BarkError {
    debug!("bark_close_wallet called");

    let result = TOKIO_RUNTIME.block_on(async { close_wallet().await });

    match result {
        Ok(_) => {
            debug!("Wallet closed successfully");
            ptr::null_mut()
        }
        Err(e) => {
            error!("Failed to close wallet: {}", e);
            Box::into_raw(Box::new(BarkError::new(&e.to_string())))
        }
    }
}

/// Get offchain and onchain balances
#[no_mangle]
pub extern "C" fn bark_get_balance(no_sync: bool, balance_out: *mut BarkBalance) -> *mut BarkError {
    debug!("bark_get_balance called, no_sync: {}", no_sync);

    if balance_out.is_null() {
        error!("Balance output pointer is null");
        return Box::into_raw(Box::new(BarkError::new("balance_out is null")));
    }

    // Run the async function
    debug!("Running get_balance async function");

    let result = TOKIO_RUNTIME.block_on(async { get_balance(no_sync).await });

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
#[no_mangle]
pub extern "C" fn bark_get_onchain_address(address_out: *mut *mut c_char) -> *mut BarkError {
    debug!("bark_get_onchain_address called");

    // --- Input Validation ---
    if address_out.is_null() {
        error!(
            "Null pointer passed to bark_get_onchain_address (address_out={})",
            address_out.is_null()
        );
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    // Initialize output pointer to null
    unsafe {
        *address_out = ptr::null_mut();
    }

    // --- Runtime and Async Execution ---
    debug!("Running get_onchain_address async function");
    let result = TOKIO_RUNTIME.block_on(async { get_onchain_address().await });

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
#[no_mangle]
pub extern "C" fn bark_send_onchain(
    destination: *const c_char,
    amount_sat: u64,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_send_onchain called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if destination.is_null() || txid_out.is_null() {
        error!(
            "Null pointer passed to bark_send_onchain (destination={}, txid_out={})",
            destination.is_null(),
            txid_out.is_null()
        );
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    // Initialize output pointer to null
    unsafe {
        *txid_out = ptr::null_mut();
    }

    // --- Conversions ---
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
    debug!("Running send_onchain async function");
    // Pass destination_str, validation happens inside send_onchain
    let result =
        TOKIO_RUNTIME.block_on(async { send_onchain(&destination_str, amount, no_sync).await });

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
#[no_mangle]
pub extern "C" fn bark_drain_onchain(
    destination: *const c_char,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_drain_onchain called: no_sync={}", no_sync);

    // --- Input Validation ---
    if destination.is_null() || txid_out.is_null() {
        error!("Null pointer passed to bark_drain_onchain");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *txid_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
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
    let result = TOKIO_RUNTIME.block_on(async { drain_onchain(&destination_str, no_sync).await });

    // --- Result Handling ---
    // Use the new helper function
    handle_txid_result(result, txid_out, "drain")
}

/// Send funds to multiple recipients using the onchain wallet.
#[no_mangle]
pub extern "C" fn bark_send_many_onchain(
    destinations: *const *const c_char,
    amounts_sat: *const u64,
    num_outputs: usize,
    no_sync: bool,
    txid_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_send_many_onchain called: num_outputs={}, no_sync={}",
        num_outputs, no_sync
    );

    // --- Input Validation ---
    if destinations.is_null() || amounts_sat.is_null() || txid_out.is_null() || num_outputs == 0 {
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
    let result = TOKIO_RUNTIME.block_on(async {
        // Open the wallet just to get the network for validation
        let net = {
            let mut wallet_guard = GLOBAL_WALLET.lock().await;
            let w = wallet_guard.as_mut().context("Wallet not loaded")?;
            w.properties()?.network
            // Wallet `w` is dropped here
        };

        // Convert C arrays to Rust Vec<(Address, Amount)> *with network validation*
        let outputs_vec = convert_outputs(destinations, amounts_sat, num_outputs, net)?;

        // Call the actual send_many logic (will re-open wallet internally)
        send_many_onchain(outputs_vec, no_sync).await
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

/// Get the list of onchain UTXOs as a JSON string.
#[no_mangle]
pub extern "C" fn bark_get_onchain_utxos(
    no_sync: bool,
    utxos_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_get_onchain_utxos called: no_sync={}", no_sync);

    // --- Input Validation ---
    if utxos_json_out.is_null() {
        error!("Null pointer passed to bark_get_onchain_utxos");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *utxos_json_out = ptr::null_mut();
    } // Initialize output

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { get_onchain_utxos(no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, utxos_json_out, "get_onchain_utxos")
}
