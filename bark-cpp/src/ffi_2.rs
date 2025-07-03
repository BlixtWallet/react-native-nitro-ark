use std::{ffi::c_char, ptr};

use bark::ark::bitcoin::Amount;
use logger::log::{debug, error};

use crate::ffi::*;
use crate::ffi_utils::*;
use crate::*;

/// Get the wallet's VTXO public key (hex string).
#[no_mangle]
pub extern "C" fn bark_get_vtxo_pubkey(
    index: *const u32,
    pubkey_hex_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_get_vtxo_pubkey called");

    // --- Input Validation ---
    if pubkey_hex_out.is_null() {
        error!("Null pointer passed to bark_get_vtxo_pubkey");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *pubkey_hex_out = ptr::null_mut();
    } // Initialize output

    let index_opt = if index.is_null() {
        None
    } else {
        Some(unsafe { *index })
    };

    // --- Runtime and Async Execution ---
    // `get_vtxo_pubkey` is async because `open_wallet` is async
    let result = TOKIO_RUNTIME.block_on(async { get_vtxo_pubkey(index_opt).await });

    // --- Result Handling ---
    handle_string_result(result, pubkey_hex_out, "get_vtxo_pubkey")
}

/// Get the list of VTXOs as a JSON string.
#[no_mangle]
pub extern "C" fn bark_get_vtxos(
    no_sync: bool,
    vtxos_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_get_vtxos called: no_sync={}", no_sync);

    // --- Input Validation ---
    if vtxos_json_out.is_null() {
        error!("Null pointer passed to bark_get_vtxos");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *vtxos_json_out = ptr::null_mut();
    } // Initialize output

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { get_vtxos(no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, vtxos_json_out, "get_vtxos")
}

/// Refresh VTXOs based on specified criteria.
#[no_mangle]
pub extern "C" fn bark_refresh_vtxos(
    refresh_opts: BarkRefreshOpts, // Pass struct by value
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_refresh_vtxos called: mode={:?}, threshold={}, num_specific={}, no_sync={}",
        refresh_opts.mode_type,
        refresh_opts.threshold_value,
        refresh_opts.num_specific_vtxo_ids,
        no_sync
    ); // Use Debug on enum if derived

    // --- Input Validation ---
    if status_json_out.is_null() {
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
    let result = TOKIO_RUNTIME.block_on(async { refresh_vtxos(rust_mode, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "refresh_vtxos")
}

// --- Board FFI ---

/// Board a specific amount from the onchain wallet into Ark.
#[no_mangle]
pub extern "C" fn bark_board_amount(
    amount_sat: u64,
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_board_amount called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if status_json_out.is_null() {
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
    let amount = Amount::from_sat(amount_sat);

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { board_amount(amount, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "board_amount")
}

/// Board all available funds from the onchain wallet into Ark.
#[no_mangle]
pub extern "C" fn bark_board_all(
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_board_all called: no_sync={}", no_sync);

    // --- Input Validation ---
    if status_json_out.is_null() {
        error!("Null pointer passed to bark_board_all");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { board_all(no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "board_all")
}

#[no_mangle]
pub extern "C" fn bark_send(
    destination: *const c_char,
    amount_sat: u64,        // Use 0 or ULLONG_MAX to indicate 'not provided by user'
    comment: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
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
    if destination.is_null() || status_json_out.is_null() {
        error!("Null pointer passed to bark_send");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
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
    let result = TOKIO_RUNTIME.block_on(async {
        send_payment(&destination_str, rust_amount_opt, rust_comment_opt, no_sync).await
    });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "send_payment")
}

// --- Send Round Onchain FFI ---

/// Send an onchain payment via an Ark round.
#[no_mangle]
pub extern "C" fn bark_send_round_onchain(
    destination: *const c_char,
    amount_sat: u64,
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_send_round_onchain called: amount_sat={}, no_sync={}",
        amount_sat, no_sync
    );

    // --- Input Validation ---
    if destination.is_null() || status_json_out.is_null() {
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
    let result = TOKIO_RUNTIME
        .block_on(async { send_round_onchain(&destination_str, amount, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "send_round_onchain")
}

// --- Offboard FFI ---

/// Offboard specific VTXOs to an optional onchain address.
#[no_mangle]
pub extern "C" fn bark_offboard_specific(
    specific_vtxo_ids: *const *const c_char,
    num_specific_vtxo_ids: usize,
    optional_address: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_offboard_specific called: num_vtxos={}, no_sync={}",
        num_specific_vtxo_ids, no_sync
    );

    // --- Input Validation ---
    if specific_vtxo_ids.is_null() || status_json_out.is_null() {
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
    let result = TOKIO_RUNTIME
        .block_on(async { offboard_specific(rust_vtxo_ids, rust_address_opt, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "offboard_specific")
}

/// Offboard all VTXOs to an optional onchain address.
#[no_mangle]
pub extern "C" fn bark_offboard_all(
    optional_address: *const c_char, // Nullable
    no_sync: bool,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!("bark_offboard_all called: no_sync={}", no_sync);

    // --- Input Validation ---
    if status_json_out.is_null() {
        error!("Null pointer passed to bark_offboard_all (excluding optional_address)");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Conversions ---
    let rust_address_opt = c_string_to_option(optional_address);

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { offboard_all(rust_address_opt, no_sync).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "offboard_all")
}

// --- Exit FFI ---

/// Start the exit process for specific VTXOs.
#[no_mangle]
pub extern "C" fn bark_exit_start_specific(
    specific_vtxo_ids: *const *const c_char,
    num_specific_vtxo_ids: usize,
    status_json_out: *mut *mut c_char,
) -> *mut BarkError {
    debug!(
        "bark_exit_start_specific called: num_vtxos={}",
        num_specific_vtxo_ids
    );

    // --- Input Validation ---
    if specific_vtxo_ids.is_null() || status_json_out.is_null() {
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
    let result = TOKIO_RUNTIME.block_on(async { start_exit_for_vtxos(rust_vtxo_ids).await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_start_specific")
}

/// Start the exit process for all VTXOs in the wallet.
#[no_mangle]
pub extern "C" fn bark_exit_start_all(status_json_out: *mut *mut c_char) -> *mut BarkError {
    debug!("bark_exit_start_all called");

    // --- Input Validation ---
    if status_json_out.is_null() {
        error!("Null pointer passed to bark_exit_start_all");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { start_exit_for_entire_wallet().await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_start_all")
}

/// Progress the exit process once and return the current status.
#[no_mangle]
pub extern "C" fn bark_exit_progress_once(status_json_out: *mut *mut c_char) -> *mut BarkError {
    debug!("bark_exit_progress_once called");

    // --- Input Validation ---
    if status_json_out.is_null() {
        error!("Null pointer passed to bark_exit_progress_once");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *status_json_out = ptr::null_mut();
    } // Initialize output

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { exit_progress_once().await });

    // --- Result Handling ---
    handle_string_result(result, status_json_out, "exit_progress_once")
}

/// FFI: Creates a BOLT11 invoice for receiving payments.
#[no_mangle]
pub extern "C" fn bark_bolt11_invoice(
    amount_msat: u64,
    invoice_out: *mut *mut c_char,
) -> *mut BarkError {
    if invoice_out.is_null() {
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }
    unsafe {
        *invoice_out = ptr::null_mut();
    }

    let result = TOKIO_RUNTIME.block_on(async { bolt11_invoice(amount_msat).await });

    handle_string_result(result, invoice_out, "bolt11_invoice")
}

/// FFI: Claims a BOLT11 payment using an invoice.
#[no_mangle]
pub extern "C" fn bark_claim_bolt11_payment(bolt11: *const c_char) -> *mut BarkError {
    debug!("bark_claim_bolt11_payment called");

    // --- Input Validation ---
    if bolt11.is_null() {
        error!("Null pointer passed to bark_claim_bolt11_payment");
        return Box::into_raw(Box::new(BarkError::new("Null pointer argument provided")));
    }

    // --- Conversions ---
    let rust_bolt11 = match c_string_to_string(bolt11) {
        Ok(s) => s,
        Err(e) => return Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    };

    // --- Runtime and Async Execution ---
    let result = TOKIO_RUNTIME.block_on(async { claim_bolt11_payment(rust_bolt11).await });

    // --- Result Handling ---
    match result {
        Ok(_) => ptr::null_mut(),
        Err(e) => Box::into_raw(Box::new(BarkError::new(&e.to_string()))),
    }
}
