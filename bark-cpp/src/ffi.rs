use super::*;
use bip39::Mnemonic;
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
        return ptr::null();
    }
    unsafe { (*error).message }
}

fn to_rust_config_opts(c_opts: &BarkConfigOpts) -> ConfigOpts {
    let asp = c_string_to_option(c_opts.asp);
    let esplora = c_string_to_option(c_opts.esplora);
    let bitcoind = c_string_to_option(c_opts.bitcoind);
    let bitcoind_cookie = c_string_to_option(c_opts.bitcoind_cookie);
    let bitcoind_user = c_string_to_option(c_opts.bitcoind_user);
    let bitcoind_pass = c_string_to_option(c_opts.bitcoind_pass);

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
        unsafe { CStr::from_ptr(s).to_str().ok().map(String::from) }
    }
}

fn to_rust_create_opts(c_opts: &BarkCreateOpts) -> Result<CreateOpts, anyhow::Error> {
    let mnemonic = if c_opts.mnemonic.is_null() {
        None
    } else {
        let mnemonic_str = unsafe { CStr::from_ptr(c_opts.mnemonic).to_str()? };
        if !mnemonic_str.is_empty() {
            Some(Mnemonic::from_str(mnemonic_str)?)
        } else {
            None
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
    if datadir.is_null() {
        return Box::into_raw(Box::new(BarkError::new("datadir is null")));
    }

    let datadir_str = match unsafe { CStr::from_ptr(datadir).to_str() } {
        Ok(s) => s,
        Err(_) => return Box::into_raw(Box::new(BarkError::new("Invalid UTF-8 in datadir path"))),
    };

    let create_opts = match to_rust_create_opts(&opts) {
        Ok(o) => o,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // Create a new runtime for the async function
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // Run the async function
    let result =
        runtime.block_on(async { create_wallet(Path::new(datadir_str), create_opts).await });

    match result {
        Ok(_) => ptr::null_mut(),
        Err(e) => Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
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
    balance_out: *mut BarkBalance,
) -> *mut BarkError {
    if datadir.is_null() {
        return Box::into_raw(Box::new(BarkError::new("datadir is null")));
    }
    
    if balance_out.is_null() {
        return Box::into_raw(Box::new(BarkError::new("balance_out is null")));
    }

    let datadir_str = match unsafe { CStr::from_ptr(datadir).to_str() } {
        Ok(s) => s,
        Err(_) => return Box::into_raw(Box::new(BarkError::new("Invalid UTF-8 in datadir path"))),
    };

    // Create a new runtime for the async function
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // Run the async function
    let result = runtime.block_on(async {
        get_balance(Path::new(datadir_str), no_sync).await
    });

    match result {
        Ok(balance) => {
            // Store the result in the output parameter
            unsafe {
                (*balance_out).onchain = balance.onchain;
                (*balance_out).offchain = balance.offchain;
                (*balance_out).pending_exit = balance.pending_exit;
            }
            ptr::null_mut()
        }
        Err(e) => Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    }
}