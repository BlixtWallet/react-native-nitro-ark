use std::{
    ffi::{c_char, CStr, CString},
    path::PathBuf,
    ptr, slice,
    str::FromStr,
};

use anyhow::{bail, Context};
use bark::ark::{bitcoin::Txid, VtxoId};
use bip39::Mnemonic;
use logger::tracing::{debug, error, warn};

use crate::{
    ffi::{BarkConfigOpts, BarkCreateOpts, BarkError, BarkRefreshModeType, BarkRefreshOpts},
    ConfigOpts, CreateOpts, RefreshMode,
};

// Helper to convert C opts to Rust RefreshMode
pub(crate) fn convert_refresh_opts(opts: &BarkRefreshOpts) -> anyhow::Result<RefreshMode> {
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

pub(crate) fn to_rust_config_opts(c_opts: &BarkConfigOpts) -> ConfigOpts {
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

pub(crate) fn c_string_to_option(s: *const c_char) -> Option<String> {
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

pub(crate) fn to_rust_create_opts(c_opts: &BarkCreateOpts) -> anyhow::Result<CreateOpts> {
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

// Helper to handle Txid result and C string conversion for FFI functions
pub(crate) fn handle_txid_result(
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

pub(crate) fn handle_string_result(
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

// Helper to convert C string to PathBuf
pub fn c_string_to_path(s: *const c_char) -> anyhow::Result<PathBuf> {
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
pub(crate) fn c_string_to_mnemonic(s: *const c_char) -> anyhow::Result<Mnemonic> {
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
pub(crate) fn c_string_to_string(s: *const c_char) -> anyhow::Result<String> {
    if s.is_null() {
        bail!("C string is null");
    }

    let s = unsafe { CStr::from_ptr(s).to_str()? };
    Ok(s.to_string())
}

// Helper to convert C array of C strings to Vec<VtxoId>
pub(crate) fn convert_vtxo_ids(
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
