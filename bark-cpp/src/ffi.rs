use crate::utils::ConfigOpts;

use super::*;
use bip39::Mnemonic;
use logger::log::error;
use logger::Logger;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::str::FromStr;

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

// Extract string from C string
fn c_string_to_string(s: *const c_char) -> anyhow::Result<String> {
    if s.is_null() {
        bail!("C string is null");
    }

    let s = unsafe { CStr::from_ptr(s).to_str()? };
    Ok(s.to_string())
}
