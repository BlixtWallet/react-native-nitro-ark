#![cfg(test)]

use crate::ffi::*;
use crate::ffi_2::*;
use std::env;
use std::ffi::{c_char, CStr, CString};
use std::fs;
use std::path::PathBuf;
use std::ptr;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        // This is important to get logs from the library during tests.
        bark_init_logger();
    });
}

const VALID_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// Helper to create CString for tests, panics on failure.
fn c_string_for_test(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|e| panic!("Failed to create CString for '{}': {}", s, e))
}

// A test fixture for managing a temporary wallet directory and loaded wallet state.
// It loads a wallet on creation and closes it on drop.
// Tests requiring a loaded wallet should use this.
struct WalletTestFixture {
    temp_dir: PathBuf,
    // Keep CStrings alive for the duration of the test by holding ownership.
    _datadir_c: CString,
    _mnemonic_c: CString,
    _asp_url_c: CString,
    _esplora_url_c: CString,
}

impl WalletTestFixture {
    // This will set up a temporary directory and load a wallet into the GLOBAL_WALLET.
    // It will panic if wallet loading fails, as tests using this fixture depend on it.
    fn new(test_name: &str) -> Self {
        setup(); // Ensure logger is initialized

        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test_wallets");
        let temp_dir = base_dir.join(test_name);

        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let datadir_c = c_string_for_test(temp_dir.to_str().unwrap());
        let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
        // Note: These ports should ideally be unique per test or managed by a mock server framework.
        // For now, we assume they are available or the test is ignored.
        let asp_url_c = c_string_for_test("http://127.0.0.1:12345");
        let esplora_url_c = c_string_for_test("http://127.0.0.1:3002");

        let config_opts = BarkConfigOpts {
            asp: asp_url_c.as_ptr(),
            esplora: esplora_url_c.as_ptr(),
            bitcoind: ptr::null(),
            bitcoind_cookie: ptr::null(),
            bitcoind_user: ptr::null(),
            bitcoind_pass: ptr::null(),
            vtxo_refresh_expiry_threshold: 144,
            fallback_fee_rate: ptr::null(),
        };

        let create_opts = BarkCreateOpts {
            regtest: true,
            signet: false,
            bitcoin: false,
            mnemonic: mnemonic_c.as_ptr(),
            birthday_height: 0,
            config: config_opts,
        };

        let err_ptr = bark_load_wallet(datadir_c.as_ptr(), create_opts);
        if !err_ptr.is_null() {
            unsafe {
                let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
                bark_free_error(err_ptr);
                // We panic here because subsequent tests will fail if the wallet isn't loaded.
                // The `#[ignore]` attribute should be used for tests requiring a live server.
                panic!("Wallet loading failed in test setup for '{}': {}. Ensure mock servers or a regtest environment is running.", test_name, msg);
            }
        }

        WalletTestFixture {
            temp_dir,
            _datadir_c: datadir_c,
            _mnemonic_c: mnemonic_c,
            _asp_url_c: asp_url_c,
            _esplora_url_c: esplora_url_c,
        }
    }
}

impl Drop for WalletTestFixture {
    fn drop(&mut self) {
        // Close the wallet
        let err_ptr = bark_close_wallet();
        if !err_ptr.is_null() {
            unsafe {
                let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
                // Using eprintln instead of panic in drop is generally safer.
                eprintln!(
                    "Warning: bark_close_wallet failed during test teardown for '{}': {}",
                    self.temp_dir.display(),
                    msg
                );
                bark_free_error(err_ptr);
            }
        }

        // Clean up the directory
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to clean up test dir {}: {}",
                    self.temp_dir.display(),
                    e
                )
            });
        }
    }
}

// --- Basic FFI Tests ---

#[test]
fn test_bark_init_logger_call() {
    setup(); // Just ensures it can be called without panicking.
}

#[test]
fn test_bark_create_and_free_mnemonic() {
    setup();
    let mnemonic_ptr = bark_create_mnemonic();
    assert!(
        !mnemonic_ptr.is_null(),
        "bark_create_mnemonic should return a valid pointer"
    );

    unsafe {
        let mnemonic_c_str = CStr::from_ptr(mnemonic_ptr);
        let mnemonic_rust_str = mnemonic_c_str
            .to_str()
            .expect("Mnemonic is not valid UTF-8");
        assert_eq!(
            mnemonic_rust_str.split_whitespace().count(),
            12,
            "Mnemonic should have 12 words"
        );
        bark_free_string(mnemonic_ptr);
    }
}

#[test]
fn test_bark_free_string_with_null() {
    setup();
    bark_free_string(ptr::null_mut()); // Should not panic.
}

#[test]
fn test_bark_error_handling_functions() {
    setup();
    // Test bark_error_message with null
    assert!(
        bark_error_message(ptr::null()).is_null(),
        "bark_error_message with null should return null"
    );

    // Test bark_free_error with null
    bark_free_error(ptr::null_mut()); // Should not panic.

    // Create a real error to test message and free
    let error_message = "This is a test error";
    let bark_error = BarkError::new(error_message);
    let error_ptr = Box::into_raw(Box::new(bark_error));

    unsafe {
        let returned_message_ptr = bark_error_message(error_ptr);
        assert!(
            !returned_message_ptr.is_null(),
            "bark_error_message should return the message pointer"
        );
        let returned_message = CStr::from_ptr(returned_message_ptr).to_str().unwrap();
        assert_eq!(
            returned_message, error_message,
            "The returned error message should match the original"
        );

        bark_free_error(error_ptr);
    }
}

// --- Wallet Loading and Closing Tests ---

#[test]
fn test_bark_load_wallet_null_datadir() {
    setup();
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let asp_url_c = c_string_for_test("http://127.0.0.1:12345");
    let esplora_url_c = c_string_for_test("http://127.0.0.1:3002");

    let config_opts = BarkConfigOpts {
        asp: asp_url_c.as_ptr(),
        esplora: esplora_url_c.as_ptr(),
        bitcoind: ptr::null(),
        bitcoind_cookie: ptr::null(),
        bitcoind_user: ptr::null(),
        bitcoind_pass: ptr::null(),
        vtxo_refresh_expiry_threshold: 144,
        fallback_fee_rate: ptr::null(),
    };
    let create_opts = BarkCreateOpts {
        regtest: true,
        signet: false,
        bitcoin: false,
        mnemonic: mnemonic_c.as_ptr(),
        birthday_height: 0,
        config: config_opts,
    };

    let err_ptr = bark_load_wallet(ptr::null(), create_opts);
    assert!(
        !err_ptr.is_null(),
        "bark_load_wallet should fail with null datadir"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("datadir is null"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_bark_load_wallet_no_network() {
    setup();
    let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test_wallet_no_network");
    fs::create_dir_all(&temp_dir).unwrap();
    let datadir_c = c_string_for_test(temp_dir.to_str().unwrap());
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let asp_url_c = c_string_for_test("http://127.0.0.1:12345");
    let esplora_url_c = c_string_for_test("http://127.0.0.1:3002");

    let config_opts = BarkConfigOpts {
        asp: asp_url_c.as_ptr(),
        esplora: esplora_url_c.as_ptr(),
        bitcoind: ptr::null(),
        bitcoind_cookie: ptr::null(),
        bitcoind_user: ptr::null(),
        bitcoind_pass: ptr::null(),
        vtxo_refresh_expiry_threshold: 144,
        fallback_fee_rate: ptr::null(),
    };
    // No network flag is set to true
    let create_opts = BarkCreateOpts {
        regtest: false,
        signet: false,
        bitcoin: false,
        mnemonic: mnemonic_c.as_ptr(),
        birthday_height: 0,
        config: config_opts,
    };

    let err_ptr = bark_load_wallet(datadir_c.as_ptr(), create_opts);
    assert!(
        !err_ptr.is_null(),
        "bark_load_wallet should fail with no network specified"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("A network must be specified"));
        bark_free_error(err_ptr);
    }
    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_bark_close_wallet_when_none_loaded() {
    setup();
    // Ensure no wallet is loaded by not calling the fixture and checking state before test.
    let err_ptr = bark_close_wallet();
    assert!(
        !err_ptr.is_null(),
        "bark_close_wallet should fail if no wallet is loaded"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("No wallet is currently loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_load_and_close_wallet_success() {
    // The fixture's new() and drop() methods test the success case.
    // This test just ensures it runs without panicking.
    let _fixture = WalletTestFixture::new("load_and_close_success");
    // Wallet is loaded in new() and closed in drop()
}

// --- Wallet Functionality Tests ---

#[test]
fn test_get_balance_no_wallet_loaded() {
    setup();
    let mut balance_out = BarkBalance {
        onchain: 0,
        offchain: 0,
        pending_exit: 0,
    };
    let err_ptr = bark_get_balance(true, &mut balance_out);
    assert!(
        !err_ptr.is_null(),
        "bark_get_balance should fail if no wallet is loaded"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_balance_null_output_pointer() {
    // We need a wallet loaded for this check to be reached.
    let _fixture = WalletTestFixture::new("get_balance_null_output");
    let err_ptr = bark_get_balance(true, ptr::null_mut());
    assert!(
        !err_ptr.is_null(),
        "bark_get_balance should fail with null output pointer"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("balance_out is null"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_balance_success() {
    let _fixture = WalletTestFixture::new("get_balance_success");

    let mut balance_out = BarkBalance {
        onchain: 0,
        offchain: 0,
        pending_exit: 0,
    };
    let err_ptr = bark_get_balance(true, &mut balance_out); // no_sync = true

    if !err_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
            bark_free_error(err_ptr);
            panic!("bark_get_balance failed: {}", msg);
        }
    }

    // For a new wallet, balances should be 0.
    assert_eq!(balance_out.onchain, 0);
    assert_eq!(balance_out.offchain, 0);
    assert_eq!(balance_out.pending_exit, 0);
}

#[test]
fn test_get_onchain_address_no_wallet_loaded() {
    setup();
    let mut address_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_onchain_address(&mut address_out);
    assert!(
        !err_ptr.is_null(),
        "bark_get_onchain_address should fail if no wallet is loaded"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
    assert!(address_out.is_null());
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_onchain_address_success() {
    let _fixture = WalletTestFixture::new("get_onchain_address_success");

    let mut address_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_onchain_address(&mut address_out);

    if !err_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
            bark_free_error(err_ptr);
            panic!("bark_get_onchain_address failed: {}", msg);
        }
    }

    assert!(
        !address_out.is_null(),
        "Address output should not be null on success"
    );
    unsafe {
        let address_str = CStr::from_ptr(address_out).to_str().unwrap();
        // For regtest, addresses start with "bcrt1".
        assert!(
            address_str.starts_with("bcrt1"),
            "Address should be a regtest address, but was {}",
            address_str
        );
        bark_free_string(address_out);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_onchain_address_error_null_address_out() {
    let _fixture = WalletTestFixture::new("get_onchain_address_null_output");
    let err_ptr = bark_get_onchain_address(ptr::null_mut());
    assert!(
        !err_ptr.is_null(),
        "bark_get_onchain_address should fail with null output pointer"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer argument provided"));
        bark_free_error(err_ptr);
    }
}

// --- Onchain Send Tests ---

#[test]
fn test_send_onchain_no_wallet_loaded() {
    setup();
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut txid_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_send_onchain(destination_c.as_ptr(), 1000, true, &mut txid_out);
    assert!(
        !err_ptr.is_null(),
        "bark_send_onchain should fail if no wallet is loaded"
    );
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
    assert!(txid_out.is_null());
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_send_onchain_null_pointers() {
    let _fixture = WalletTestFixture::new("send_onchain_null_pointers");
    let mut txid_out: *mut c_char = ptr::null_mut();
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    // Null destination
    let err_ptr = bark_send_onchain(ptr::null(), 1000, true, &mut txid_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer argument provided"));
        bark_free_error(err_ptr);
    }

    // Null txid_out
    let err_ptr = bark_send_onchain(destination_c.as_ptr(), 1000, true, ptr::null_mut());
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer argument provided"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a funded regtest wallet"]
fn test_send_onchain_success() {
    let _fixture = WalletTestFixture::new("send_onchain_success");
    // To make this test pass, you would need to:
    // 1. Fund the wallet's onchain address.
    // 2. Provide a valid destination address.
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut txid_out: *mut c_char = ptr::null_mut();

    let err_ptr = bark_send_onchain(destination_c.as_ptr(), 5000, false, &mut txid_out);

    if !err_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
            bark_free_error(err_ptr);
            panic!("bark_send_onchain failed: {}", msg);
        }
    }
    assert!(!txid_out.is_null());
    unsafe {
        let txid_str = CStr::from_ptr(txid_out).to_str().unwrap();
        assert!(!txid_str.is_empty());
        bark_free_string(txid_out);
    }
}

// --- Drain and SendMany Tests ---

#[test]
fn test_drain_onchain_no_wallet_loaded() {
    setup();
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut txid_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_drain_onchain(destination_c.as_ptr(), true, &mut txid_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_drain_onchain_null_pointers() {
    let _fixture = WalletTestFixture::new("drain_onchain_null_pointers");
    let mut txid_out: *mut c_char = ptr::null_mut();
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    // Null destination
    let err_ptr = bark_drain_onchain(ptr::null(), true, &mut txid_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer argument provided"));
        bark_free_error(err_ptr);
    }

    // Null txid_out
    let err_ptr = bark_drain_onchain(destination_c.as_ptr(), true, ptr::null_mut());
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer argument provided"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_send_many_onchain_no_wallet_loaded() {
    setup();
    let dest_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let destinations = [dest_c.as_ptr()];
    let amounts = [1000u64];
    let mut txid_out: *mut c_char = ptr::null_mut();

    let err_ptr = bark_send_many_onchain(
        destinations.as_ptr(),
        amounts.as_ptr(),
        1,
        true,
        &mut txid_out,
    );
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_send_many_onchain_null_pointers() {
    let _fixture = WalletTestFixture::new("send_many_onchain_null_pointers");
    let dest_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let destinations = [dest_c.as_ptr()];
    let amounts = [1000u64];
    let mut txid_out: *mut c_char = ptr::null_mut();

    // Null destinations
    let err_ptr = bark_send_many_onchain(ptr::null(), amounts.as_ptr(), 1, true, &mut txid_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer or zero outputs provided"));
        bark_free_error(err_ptr);
    }

    // Null amounts
    let err_ptr =
        bark_send_many_onchain(destinations.as_ptr(), ptr::null(), 1, true, &mut txid_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer or zero outputs provided"));
        bark_free_error(err_ptr);
    }

    // Null txid_out
    let err_ptr = bark_send_many_onchain(
        destinations.as_ptr(),
        amounts.as_ptr(),
        1,
        true,
        ptr::null_mut(),
    );
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer or zero outputs provided"));
        bark_free_error(err_ptr);
    }

    // Zero outputs
    let err_ptr = bark_send_many_onchain(
        destinations.as_ptr(),
        amounts.as_ptr(),
        0,
        true,
        &mut txid_out,
    );
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Null pointer or zero outputs provided"));
        bark_free_error(err_ptr);
    }
}

// --- VTXO and Refresh Tests ---

#[test]
fn test_get_vtxos_no_wallet_loaded() {
    setup();
    let mut vtxos_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_vtxos(true, &mut vtxos_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_vtxos_success() {
    let _fixture = WalletTestFixture::new("get_vtxos_success");
    let mut vtxos_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_vtxos(true, &mut vtxos_json_out);
    assert!(err_ptr.is_null());
    assert!(!vtxos_json_out.is_null());
    unsafe {
        let json_str = CStr::from_ptr(vtxos_json_out).to_str().unwrap();
        assert_eq!(json_str, "[]"); // Expect empty array for new wallet
        bark_free_string(vtxos_json_out);
    }
}

#[test]
fn test_refresh_vtxos_no_wallet_loaded() {
    setup();
    let refresh_opts = BarkRefreshOpts {
        mode_type: BarkRefreshModeType::DefaultThreshold,
        threshold_value: 0,
        specific_vtxo_ids: ptr::null(),
        num_specific_vtxo_ids: 0,
    };
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_refresh_vtxos(refresh_opts, true, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

// --- UTXO and Pubkey Tests ---

#[test]
fn test_get_onchain_utxos_no_wallet_loaded() {
    setup();
    let mut utxos_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_onchain_utxos(true, &mut utxos_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_onchain_utxos_success() {
    let _fixture = WalletTestFixture::new("get_onchain_utxos_success");
    let mut utxos_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_onchain_utxos(true, &mut utxos_json_out);
    assert!(err_ptr.is_null());
    assert!(!utxos_json_out.is_null());
    unsafe {
        let json_str = CStr::from_ptr(utxos_json_out).to_str().unwrap();
        assert!(json_str.starts_with("[]"));
        bark_free_string(utxos_json_out);
    }
}

#[test]
fn test_get_vtxo_pubkey_no_wallet_loaded() {
    setup();
    let mut pubkey_hex_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_get_vtxo_pubkey(ptr::null(), &mut pubkey_hex_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_get_vtxo_pubkey_success() {
    let _fixture = WalletTestFixture::new("get_vtxo_pubkey_success");
    let mut pubkey_hex_out: *mut c_char = ptr::null_mut();
    // Get next available pubkey
    let err_ptr = bark_get_vtxo_pubkey(ptr::null(), &mut pubkey_hex_out);
    assert!(err_ptr.is_null());
    assert!(!pubkey_hex_out.is_null());
    unsafe {
        let pubkey_str = CStr::from_ptr(pubkey_hex_out).to_str().unwrap();
        assert_eq!(pubkey_str.len(), 66); // 33 bytes * 2 hex chars
        bark_free_string(pubkey_hex_out);
    }
}

// --- Boarding Tests ---

#[test]
fn test_board_amount_no_wallet_loaded() {
    setup();
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_board_amount(10000, true, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
#[ignore = "This test requires a running regtest environment with esplora and asp servers"]
fn test_board_amount_zero_amount() {
    let _fixture = WalletTestFixture::new("board_amount_zero_amount");
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_board_amount(0, true, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Board amount cannot be zero"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_board_all_no_wallet_loaded() {
    setup();
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_board_all(true, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

// --- Generic Send Tests ---

#[test]
fn test_send_no_wallet_loaded() {
    setup();
    let dest_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut status_json_out: *mut c_char = ptr::null_mut();
    const AMOUNT_NOT_PROVIDED: u64 = u64::MAX;

    let err_ptr = bark_send(
        dest_c.as_ptr(),
        AMOUNT_NOT_PROVIDED,
        ptr::null(),
        true,
        &mut status_json_out,
    );
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

// --- Offboarding Tests ---

#[test]
fn test_offboard_specific_no_wallet_loaded() {
    setup();
    let vtxo_id_c =
        c_string_for_test("0000000000000000000000000000000000000000000000000000000000000000:0");
    let vtxo_ids = [vtxo_id_c.as_ptr()];
    let mut status_json_out: *mut c_char = ptr::null_mut();

    let err_ptr = bark_offboard_specific(
        vtxo_ids.as_ptr(),
        1,
        ptr::null(),
        true,
        &mut status_json_out,
    );
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_offboard_all_no_wallet_loaded() {
    setup();
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_offboard_all(ptr::null(), true, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

// --- Exit Flow Tests ---

#[test]
fn test_exit_start_specific_no_wallet_loaded() {
    setup();
    let vtxo_id_c =
        c_string_for_test("0000000000000000000000000000000000000000000000000000000000000000:0");
    let vtxo_ids = [vtxo_id_c.as_ptr()];
    let mut status_json_out: *mut c_char = ptr::null_mut();

    let err_ptr = bark_exit_start_specific(vtxo_ids.as_ptr(), 1, &mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_exit_start_all_no_wallet_loaded() {
    setup();
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_exit_start_all(&mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_exit_progress_once_no_wallet_loaded() {
    setup();
    let mut status_json_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_exit_progress_once(&mut status_json_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

// --- BOLT11 Invoice Tests ---

#[test]
fn test_bolt11_invoice_no_wallet_loaded() {
    setup();
    let mut invoice_out: *mut c_char = ptr::null_mut();
    let err_ptr = bark_bolt11_invoice(1000, &mut invoice_out);
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}

#[test]
fn test_claim_bolt11_payment_no_wallet_loaded() {
    setup();
    let bolt11_c = c_string_for_test("lnbcrt100n1pjz3zsp5...");
    let err_ptr = bark_claim_bolt11_payment(bolt11_c.as_ptr());
    assert!(!err_ptr.is_null());
    unsafe {
        let msg = CStr::from_ptr(bark_error_message(err_ptr)).to_string_lossy();
        assert!(msg.contains("Wallet not loaded"));
        bark_free_error(err_ptr);
    }
}
