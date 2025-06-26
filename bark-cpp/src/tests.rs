#![cfg(test)]

use crate::ffi::*;
use std::env;
use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;
use std::ptr; // For env!("CARGO_MANIFEST_DIR")
use std::sync::Once;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        bark_init_logger();
    });
}

const VALID_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn c_string_for_test(s: &str) -> CString {
    CString::new(s)
        .unwrap_or_else(|e| panic!("Failed to create CString in test for '{}': {}", s, e))
}

#[test]
fn test_bark_init_logger_call() {
    setup();
}

#[test]
fn test_bark_create_and_free_mnemonic() {
    let mnemonic_ptr = bark_create_mnemonic();
    assert!(!mnemonic_ptr.is_null());
    unsafe {
        let mnemonic_c_str = CStr::from_ptr(mnemonic_ptr);
        let mnemonic_rust_str = mnemonic_c_str
            .to_str()
            .expect("Mnemonic C string is not valid UTF-8.");
        assert!(!mnemonic_rust_str.is_empty());
        let word_count = mnemonic_rust_str.split_whitespace().count();
        assert_eq!(word_count, 12); // Standard BIP-39 mnemonics are 12 or 24 words
        bark_free_string(mnemonic_ptr);
    }
}

#[test]
fn test_bark_create_wallet_error_on_null_datadir() {
    let asp_url_c = c_string_for_test("http://127.0.0.1:12345");
    let esplora_url_c = c_string_for_test("http://127.0.0.1:12346");
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);

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
        force: false,
        regtest: true,
        signet: false,
        bitcoin: false,
        mnemonic: mnemonic_c.as_ptr(),
        birthday_height: 0,
        config: config_opts,
    };

    let error_ptr = bark_create_wallet(ptr::null(), create_opts);
    assert!(!error_ptr.is_null());

    unsafe {
        let error_message_ptr = bark_error_message(error_ptr);
        assert!(!error_message_ptr.is_null());
        let error_message_c_str = CStr::from_ptr(error_message_ptr);
        let error_message_rust_str = error_message_c_str
            .to_str()
            .expect("Error message C string is not valid UTF-8.");
        assert!(error_message_rust_str.contains("datadir is null"));
        bark_free_error(error_ptr);
    }
}

#[test]
fn test_bark_free_error_null_ptr() {
    bark_free_error(ptr::null_mut()); // Should not panic
}

#[test]
fn test_bark_free_string_null_ptr() {
    bark_free_string(ptr::null_mut()); // Should not panic
}

#[test]
fn test_bark_error_message_null_ptr() {
    let msg_ptr = bark_error_message(ptr::null());
    assert!(msg_ptr.is_null()); // Should return null and not panic
}

#[test]
fn test_bark_create_wallet_expects_network_error_without_server() {
    let temp_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test_wallet_network_error_specific"); // Unique name
    if temp_dir_path.exists() {
        fs::remove_dir_all(&temp_dir_path).expect("Failed to remove existing test temp_dir");
    }
    fs::create_dir_all(&temp_dir_path).expect("Failed to create test temp_dir");

    let datadir_c = c_string_for_test(temp_dir_path.to_str().unwrap());
    let asp_url_c = c_string_for_test("http://127.0.0.1:12347"); // Different port
    let esplora_url_c = c_string_for_test("http://127.0.0.1:12348"); // Different port
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);

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
        force: true,
        regtest: true,
        signet: false,
        bitcoin: false,
        mnemonic: mnemonic_c.as_ptr(),
        birthday_height: 0,
        config: config_opts,
    };

    setup(); // Ensure logger is initialized for debug output
    let error_ptr = bark_create_wallet(datadir_c.as_ptr(), create_opts);

    assert!(
        !error_ptr.is_null(),
        "bark_create_wallet should return an error if no server is running."
    );

    unsafe {
        let message_ptr = bark_error_message(error_ptr);
        assert!(
            !message_ptr.is_null(),
            "Error message pointer should not be null for network error."
        );
        let message = CStr::from_ptr(message_ptr).to_string_lossy();

        let is_network_error = message.contains("connect")
            || message.contains("handshake")
            || message.contains("error creating wallet")
            || message.contains("failed to connect")
            || message.contains("Connection refused");

        assert!(
            is_network_error,
            "Expected a network-related error, but got: {}",
            message
        );
        bark_free_error(error_ptr);
    }

    fs::remove_dir_all(&temp_dir_path).expect("Failed to clean up test temp_dir");
}

fn setup_temp_wallet(test_name: &str) -> (PathBuf, CString, CString, *mut BarkError) {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test_wallets");
    let temp_dir_path = base_dir.join(test_name);

    if temp_dir_path.exists() {
        fs::remove_dir_all(&temp_dir_path).unwrap_or_else(|e| {
            panic!(
                "Failed to remove existing test dir {}: {}",
                temp_dir_path.display(),
                e
            )
        });
    }
    fs::create_dir_all(&temp_dir_path).unwrap_or_else(|e| {
        panic!(
            "Failed to create test dir {}: {}",
            temp_dir_path.display(),
            e
        )
    });

    let datadir_c = c_string_for_test(temp_dir_path.to_str().unwrap());
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let asp_url_c = c_string_for_test("http://127.0.0.1:12349"); // Unique port for setup
    let esplora_url_c = c_string_for_test("http://127.0.0.1:12350"); // Unique port for setup

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
        force: true,
        regtest: true,
        signet: false,
        bitcoin: false,
        mnemonic: mnemonic_c.as_ptr(),
        birthday_height: 0,
        config: config_opts,
    };

    setup();
    let error_ptr = bark_create_wallet(datadir_c.as_ptr(), create_opts);
    (temp_dir_path, datadir_c, mnemonic_c, error_ptr)
}

fn cleanup_temp_wallet(temp_dir_path: &PathBuf) {
    if temp_dir_path.exists() {
        fs::remove_dir_all(temp_dir_path).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to clean up test dir {}: {}",
                temp_dir_path.display(),
                e
            )
        });
    }
}

#[test]
#[ignore = "This test requires a running (mock) server or will fail due to network dependency in wallet creation"]
fn test_bark_get_onchain_address_success() {
    let (temp_dir, datadir_c, mnemonic_c, create_wallet_error_ptr) =
        setup_temp_wallet("get_onchain_address_success");

    if !create_wallet_error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy();
            bark_free_error(create_wallet_error_ptr);
            panic!("Wallet creation failed in setup for ignored test_bark_get_onchain_address_success: {}. This test requires a functional wallet.", msg);
        }
    }
    let mut address_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_onchain_address(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        &mut address_out_ptr,
    );

    assert!(
        error_ptr.is_null(),
        "bark_get_onchain_address should succeed."
    );
    assert!(
        !address_out_ptr.is_null(),
        "Output address pointer should not be null."
    );

    unsafe {
        let address_str = CStr::from_ptr(address_out_ptr)
            .to_str()
            .expect("Address not valid UTF-8");
        assert!(
            !address_str.is_empty(),
            "Address string should not be empty."
        );
        assert!(
            address_str.starts_with("bcrt1"),
            "Regtest address should start with bcrt1, got: {}",
            address_str
        );
        bark_free_string(address_out_ptr);
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_onchain_address_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut address_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr =
        bark_get_onchain_address(ptr::null(), mnemonic_c.as_ptr(), &mut address_out_ptr);
    assert!(
        !error_ptr.is_null(),
        "Expected an error for null datadir in test_bark_get_onchain_address_error_null_datadir"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Error message mismatch. Got: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(address_out_ptr.is_null(), "address_out_ptr should remain null on error in test_bark_get_onchain_address_error_null_datadir");
}

#[test]
fn test_bark_get_onchain_address_error_null_mnemonic() {
    let test_name = "get_onchain_address_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, create_wallet_error_ptr) = setup_temp_wallet(test_name);

    if !create_wallet_error_ptr.is_null() {
        let error_message = unsafe {
            CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy()
        };
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable as this test only needs a valid datadir.", test_name, error_message);
        bark_free_error(create_wallet_error_ptr);
    }

    let mut address_out_ptr: *mut c_char = ptr::null_mut();
    let error_ptr = bark_get_onchain_address(datadir_c.as_ptr(), ptr::null(), &mut address_out_ptr);

    assert!(!error_ptr.is_null(), "Expected an error for null mnemonic");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(msg
                .to_string_lossy()
                .contains("Null pointer argument provided"));
            bark_free_error(error_ptr);
        }
    }
    assert!(
        address_out_ptr.is_null(),
        "address_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_onchain_address_error_null_address_out() {
    let test_name = "get_onchain_address_error_null_address_out";
    let (temp_dir, datadir_c, mnemonic_c, create_wallet_error_ptr) = setup_temp_wallet(test_name);

    if !create_wallet_error_ptr.is_null() {
        let error_message = unsafe {
            CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy()
        };
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable as this test only needs valid datadir and mnemonic.", test_name, error_message);
        bark_free_error(create_wallet_error_ptr);
    }

    let error_ptr =
        bark_get_onchain_address(datadir_c.as_ptr(), mnemonic_c.as_ptr(), ptr::null_mut());

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null address_out"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(msg
                .to_string_lossy()
                .contains("Null pointer argument provided"));
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_balance_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut balance_out = BarkBalance {
        onchain: 0,
        offchain: 0,
        pending_exit: 0,
    };

    let error_ptr = bark_get_balance(ptr::null(), true, mnemonic_c.as_ptr(), &mut balance_out);

    assert!(!error_ptr.is_null(), "Expected an error for null datadir");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy().contains("C string is null"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
}

#[test]
fn test_bark_get_balance_error_null_mnemonic() {
    let test_name = "get_balance_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, create_wallet_error_ptr) = setup_temp_wallet(test_name);

    if !create_wallet_error_ptr.is_null() {
        let error_message = unsafe {
            CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy()
        };
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable as this test only needs a valid datadir.", test_name, error_message);
        bark_free_error(create_wallet_error_ptr);
    }

    let mut balance_out = BarkBalance {
        onchain: 0,
        offchain: 0,
        pending_exit: 0,
    };
    let error_ptr = bark_get_balance(datadir_c.as_ptr(), true, ptr::null(), &mut balance_out);

    assert!(!error_ptr.is_null(), "Expected an error for null mnemonic");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy().contains("C string is null"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_balance_error_null_balance_out() {
    let test_name = "get_balance_error_null_balance_out";
    let (temp_dir, datadir_c, mnemonic_c, create_wallet_error_ptr) = setup_temp_wallet(test_name);

    if !create_wallet_error_ptr.is_null() {
        let error_message = unsafe {
            CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy()
        };
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable as this test only needs valid datadir and mnemonic.", test_name, error_message);
        bark_free_error(create_wallet_error_ptr);
    }

    let error_ptr = bark_get_balance(
        datadir_c.as_ptr(),
        true,
        mnemonic_c.as_ptr(),
        ptr::null_mut(),
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null balance_out"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(msg.to_string_lossy().contains("balance_out is null"));
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
#[ignore = "This test requires a running (mock) server or will fail due to network dependency in wallet creation"]
fn test_bark_get_balance_success_new_wallet() {
    let (temp_dir, datadir_c, mnemonic_c, create_wallet_error_ptr) =
        setup_temp_wallet("get_balance_success_new_wallet");

    if !create_wallet_error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(create_wallet_error_ptr)).to_string_lossy();
            bark_free_error(create_wallet_error_ptr);
            panic!("Wallet creation failed in setup for ignored test_bark_get_balance_success_new_wallet: {}. This test requires a functional wallet.", msg);
        }
    }

    let mut balance_out = BarkBalance {
        onchain: 0,
        offchain: 0,
        pending_exit: 0,
    };

    let error_ptr = bark_get_balance(
        datadir_c.as_ptr(),
        true,
        mnemonic_c.as_ptr(),
        &mut balance_out,
    );

    assert!(
        error_ptr.is_null(),
        "bark_get_balance failed for new wallet (no_sync=true)"
    );
    assert_eq!(balance_out.onchain, 0);
    assert_eq!(balance_out.offchain, 0);
    assert_eq!(balance_out.pending_exit, 0);

    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_onchain_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"); // Dummy address
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_onchain(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000, // amount_sat
        true, // no_sync
        &mut txid_out_ptr,
    );

    assert!(!error_ptr.is_null(), "Expected an error for null datadir");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_send_onchain_error_null_mnemonic() {
    let test_name = "send_onchain_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        // Log but don't panic, as wallet setup might fail due to network but datadir_c is still valid.
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }

    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_onchain(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        destination_c.as_ptr(),
        1000,
        true,
        &mut txid_out_ptr,
    );

    assert!(!error_ptr.is_null(), "Expected an error for null mnemonic");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_onchain_error_null_destination() {
    let test_name = "send_onchain_error_null_destination";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }

    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null destination
        1000,
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null destination"
    );
    if !error_ptr.is_null() {
        let msg = unsafe { CStr::from_ptr(bark_error_message(error_ptr)) };
        assert!(
            msg.to_string_lossy()
                .contains("Null pointer argument provided"),
            "Unexpected error message: {}",
            msg.to_string_lossy()
        );
        bark_free_error(error_ptr);
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_onchain_error_null_txid_out() {
    let test_name = "send_onchain_error_null_txid_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    let error_ptr = bark_send_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000,
        true,
        ptr::null_mut(), // Test null txid_out
    );

    assert!(!error_ptr.is_null(), "Expected an error for null txid_out");
    if !error_ptr.is_null() {
        let msg = unsafe { CStr::from_ptr(bark_error_message(error_ptr)) };
        assert!(
            msg.to_string_lossy()
                .contains("Null pointer argument provided"),
            "Unexpected error message: {}",
            msg.to_string_lossy()
        );
        bark_free_error(error_ptr);
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_drain_onchain_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"); // Dummy address
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_drain_onchain(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        true, // no_sync
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null datadir in bark_drain_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_drain_onchain_error_null_mnemonic() {
    let test_name = "drain_onchain_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }

    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_drain_onchain(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        destination_c.as_ptr(),
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null mnemonic in bark_drain_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_drain_onchain_error_null_destination() {
    let test_name = "drain_onchain_error_null_destination";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }

    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_drain_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null destination
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null destination in bark_drain_onchain"
    );
    if !error_ptr.is_null() {
        let msg = unsafe { CStr::from_ptr(bark_error_message(error_ptr)) };
        assert!(
            msg.to_string_lossy()
                .contains("Null pointer argument provided"),
            "Unexpected error message for null destination: {}",
            msg.to_string_lossy()
        );
        bark_free_error(error_ptr);
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_drain_onchain_error_null_txid_out() {
    let test_name = "drain_onchain_error_null_txid_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!("Note: Wallet creation failed in setup for {}: {}. This is acceptable for this specific test.", test_name, unsafe {CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy()});
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    let error_ptr = bark_drain_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        true,
        ptr::null_mut(), // Test null txid_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected an error for null txid_out in bark_drain_onchain"
    );
    if !error_ptr.is_null() {
        let msg = unsafe { CStr::from_ptr(bark_error_message(error_ptr)) };
        assert!(
            msg.to_string_lossy()
                .contains("Null pointer argument provided"),
            "Unexpected error message for null txid_out: {}",
            msg.to_string_lossy()
        );
        bark_free_error(error_ptr);
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_many_onchain_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let dest_str = "bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let dest_c_str = c_string_for_test(dest_str);
    let destinations_ptr: [*const c_char; 1] = [dest_c_str.as_ptr()];
    let amounts_sat: [u64; 1] = [1000];
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_many_onchain(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        destinations_ptr.as_ptr(),
        amounts_sat.as_ptr(),
        1,    // num_outputs
        true, // no_sync
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_send_many_onchain_error_null_mnemonic() {
    let test_name = "send_many_onchain_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let dest_str = "bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let dest_c_str = c_string_for_test(dest_str);
    let destinations_ptr: [*const c_char; 1] = [dest_c_str.as_ptr()];
    let amounts_sat: [u64; 1] = [1000];
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_many_onchain(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        destinations_ptr.as_ptr(),
        amounts_sat.as_ptr(),
        1,
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_many_onchain_error_null_destinations() {
    let test_name = "send_many_onchain_error_null_destinations";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let amounts_sat: [u64; 1] = [1000];
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_many_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null destinations
        amounts_sat.as_ptr(),
        1,
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null destinations in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for null destinations: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_many_onchain_error_null_amounts_sat() {
    let test_name = "send_many_onchain_error_null_amounts_sat";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let dest_str = "bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let dest_c_str = c_string_for_test(dest_str);
    let destinations_ptr: [*const c_char; 1] = [dest_c_str.as_ptr()];
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_many_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destinations_ptr.as_ptr(),
        ptr::null(), // Test null amounts_sat
        1,
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null amounts_sat in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for null amounts_sat: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_many_onchain_error_null_txid_out() {
    let test_name = "send_many_onchain_error_null_txid_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let dest_str = "bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let dest_c_str = c_string_for_test(dest_str);
    let destinations_ptr: [*const c_char; 1] = [dest_c_str.as_ptr()];
    let amounts_sat: [u64; 1] = [1000];

    let error_ptr = bark_send_many_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destinations_ptr.as_ptr(),
        amounts_sat.as_ptr(),
        1,
        true,
        ptr::null_mut(), // Test null txid_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null txid_out in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for null txid_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_many_onchain_error_zero_outputs() {
    let test_name = "send_many_onchain_error_zero_outputs";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    // Even if pointers are valid, zero outputs should be an error.
    // Using dummy valid pointers for destinations and amounts.
    let dest_str = "bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let dest_c_str = c_string_for_test(dest_str);
    let destinations_ptr: [*const c_char; 1] = [dest_c_str.as_ptr()]; // Dummy, won't be accessed
    let amounts_sat: [u64; 1] = [1000]; // Dummy, won't be accessed
    let mut txid_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_many_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destinations_ptr.as_ptr(),
        amounts_sat.as_ptr(),
        0, // Test zero outputs
        true,
        &mut txid_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for zero outputs in bark_send_many_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer or zero outputs provided"),
                "Unexpected error message for zero outputs: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        txid_out_ptr.is_null(),
        "txid_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

// It's implied by the "Null pointer or zero outputs provided" check that if num_outputs > 0,
// then destinations and amounts_sat must not be null.
// The existing checks for null destinations/amounts_sat when num_outputs is implicitly > 0 (e.g. 1)
// already cover this. If num_outputs > 0 and destinations is null, it's caught.
// If num_outputs > 0 and amounts_sat is null, it's caught.
// So, specific tests like `_error_destinations_null_with_outputs` are redundant
// given the current error message and validation logic in ffi.rs.

#[test]
fn test_bark_get_onchain_utxos_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut utxos_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_onchain_utxos(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        true, // no_sync
        &mut utxos_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_get_onchain_utxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        utxos_json_out_ptr.is_null(),
        "utxos_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_get_onchain_utxos_error_null_mnemonic() {
    let test_name = "get_onchain_utxos_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut utxos_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_onchain_utxos(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        true,
        &mut utxos_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_get_onchain_utxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        utxos_json_out_ptr.is_null(),
        "utxos_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_onchain_utxos_error_null_utxos_json_out() {
    let test_name = "get_onchain_utxos_error_null_utxos_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_get_onchain_utxos(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        true,
        ptr::null_mut(), // Test null utxos_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null utxos_json_out in bark_get_onchain_utxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null utxos_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_vtxo_pubkey_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut pubkey_hex_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_vtxo_pubkey(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        &mut pubkey_hex_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_get_vtxo_pubkey"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        pubkey_hex_out_ptr.is_null(),
        "pubkey_hex_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_get_vtxo_pubkey_error_null_mnemonic() {
    let test_name = "get_vtxo_pubkey_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut pubkey_hex_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_vtxo_pubkey(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        &mut pubkey_hex_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_get_vtxo_pubkey"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        pubkey_hex_out_ptr.is_null(),
        "pubkey_hex_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_vtxo_pubkey_error_null_pubkey_hex_out() {
    let test_name = "get_vtxo_pubkey_error_null_pubkey_hex_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_get_vtxo_pubkey(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null_mut(), // Test null pubkey_hex_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null pubkey_hex_out in bark_get_vtxo_pubkey"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null pubkey_hex_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_vtxos_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut vtxos_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_vtxos(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        true, // no_sync
        &mut vtxos_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_get_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        vtxos_json_out_ptr.is_null(),
        "vtxos_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_get_vtxos_error_null_mnemonic() {
    let test_name = "get_vtxos_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut vtxos_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_get_vtxos(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        true,
        &mut vtxos_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_get_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        vtxos_json_out_ptr.is_null(),
        "vtxos_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_get_vtxos_error_null_vtxos_json_out() {
    let test_name = "get_vtxos_error_null_vtxos_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_get_vtxos(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        true,
        ptr::null_mut(), // Test null vtxos_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null vtxos_json_out in bark_get_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null vtxos_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_refresh_vtxos_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let refresh_opts = BarkRefreshOpts {
        mode_type: BarkRefreshModeType::DefaultThreshold,
        threshold_value: 0,
        specific_vtxo_ids: ptr::null(),
        num_specific_vtxo_ids: 0,
    };
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_refresh_vtxos(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        refresh_opts,
        true, // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_refresh_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_refresh_vtxos_error_null_mnemonic() {
    let test_name = "refresh_vtxos_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let refresh_opts = BarkRefreshOpts {
        mode_type: BarkRefreshModeType::DefaultThreshold,
        threshold_value: 0,
        specific_vtxo_ids: ptr::null(),
        num_specific_vtxo_ids: 0,
    };
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_refresh_vtxos(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        refresh_opts,
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_refresh_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_refresh_vtxos_error_null_status_json_out() {
    let test_name = "refresh_vtxos_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let refresh_opts = BarkRefreshOpts {
        mode_type: BarkRefreshModeType::DefaultThreshold,
        threshold_value: 0,
        specific_vtxo_ids: ptr::null(),
        num_specific_vtxo_ids: 0,
    };

    let error_ptr = bark_refresh_vtxos(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        refresh_opts,
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_refresh_vtxos"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_refresh_vtxos_error_specific_mode_null_ids_ptr() {
    let test_name = "refresh_vtxos_error_specific_mode_null_ids_ptr";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let refresh_opts = BarkRefreshOpts {
        mode_type: BarkRefreshModeType::Specific,
        threshold_value: 0,
        specific_vtxo_ids: ptr::null(), // Null pointer for IDs
        num_specific_vtxo_ids: 1,       // But count is > 0
    };
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_refresh_vtxos(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        refresh_opts,
        true,
        &mut status_json_out_ptr,
    );

    assert!(!error_ptr.is_null(), "Expected error for Specific mode with null specific_vtxo_ids pointer and num_specific_vtxo_ids > 0");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null specific_vtxo_ids pointer for Specific mode"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_board_amount_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_board_amount(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        1000, // amount_sat
        true, // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_board_amount"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_board_amount_error_null_mnemonic() {
    let test_name = "board_amount_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_board_amount(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        1000,
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_board_amount"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_board_amount_error_null_status_json_out() {
    let test_name = "board_amount_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_board_amount(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        1000,
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_board_amount"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_board_amount_error_zero_amount() {
    let test_name = "board_amount_error_zero_amount";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_board_amount(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        0, // Test zero amount
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for zero amount in bark_board_amount"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Board amount cannot be zero"),
                "Unexpected error message for zero amount: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_board_all_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_board_all(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        true, // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_board_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_board_all_error_null_mnemonic() {
    let test_name = "board_all_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_board_all(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_board_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_board_all_error_null_status_json_out() {
    let test_name = "board_all_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_board_all(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_board_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"); // Dummy address
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000,        // amount_sat
        ptr::null(), // comment (can be null)
        true,        // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_send"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_send_error_null_mnemonic() {
    let test_name = "send_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        destination_c.as_ptr(),
        1000,
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_send"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_error_null_destination() {
    let test_name = "send_error_null_destination";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null destination
        1000,
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null destination in bark_send"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null destination: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_error_null_status_json_out() {
    let test_name = "send_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    let error_ptr = bark_send(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000,
        ptr::null(),
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_send"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_round_onchain_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_round_onchain(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000, // amount_sat
        true, // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_send_round_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_send_round_onchain_error_null_mnemonic() {
    let test_name = "send_round_onchain_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_round_onchain(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        destination_c.as_ptr(),
        1000,
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_send_round_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_round_onchain_error_null_destination() {
    let test_name = "send_round_onchain_error_null_destination";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_round_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null destination
        1000,
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null destination in bark_send_round_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null destination: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_round_onchain_error_null_status_json_out() {
    let test_name = "send_round_onchain_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    let error_ptr = bark_send_round_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        1000,
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_send_round_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_send_round_onchain_error_zero_amount() {
    let test_name = "send_round_onchain_error_zero_amount";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let destination_c = c_string_for_test("bcrt1qxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_send_round_onchain(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        destination_c.as_ptr(),
        0, // Test zero amount
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for zero amount in bark_send_round_onchain"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy().contains("Amount cannot be zero"),
                "Unexpected error message for zero amount: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_specific_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_specific(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(),
        1,           // num_specific_vtxo_ids
        ptr::null(), // optional_address (can be null)
        true,        // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_offboard_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_offboard_specific_error_null_mnemonic() {
    let test_name = "offboard_specific_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_specific(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        vtxo_ids_ptr.as_ptr(),
        1,
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_offboard_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_specific_error_null_specific_vtxo_ids() {
    let test_name = "offboard_specific_error_null_specific_vtxo_ids";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null specific_vtxo_ids
        1,           // num_specific_vtxo_ids > 0
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(!error_ptr.is_null(), "Expected error for null specific_vtxo_ids with num_specific_vtxo_ids > 0 in bark_offboard_specific");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_specific_error_zero_num_specific_vtxo_ids() {
    let test_name = "offboard_specific_error_zero_num_specific_vtxo_ids";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    // specific_vtxo_ids can be a valid pointer even if num is 0, though it won't be accessed.
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()]; // Dummy
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(), // Can be non-null
        0,                     // Test zero num_specific_vtxo_ids
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for zero num_specific_vtxo_ids in bark_offboard_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy().contains("No VTXO IDs provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_specific_error_null_status_json_out() {
    let test_name = "offboard_specific_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];

    let error_ptr = bark_offboard_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(),
        1,
        ptr::null(),
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_offboard_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_all_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_all(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        ptr::null(), // optional_address (can be null)
        true,        // no_sync
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_offboard_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_offboard_all_error_null_mnemonic() {
    let test_name = "offboard_all_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_offboard_all(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        ptr::null(),
        true,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_offboard_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_offboard_all_error_null_status_json_out() {
    let test_name = "offboard_all_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_offboard_all(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(),
        true,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_offboard_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_specific_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_specific(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(),
        1, // num_specific_vtxo_ids
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_exit_start_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_exit_start_specific_error_null_mnemonic() {
    let test_name = "exit_start_specific_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_specific(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        vtxo_ids_ptr.as_ptr(),
        1,
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_exit_start_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_specific_error_null_specific_vtxo_ids() {
    let test_name = "exit_start_specific_error_null_specific_vtxo_ids";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null(), // Test null specific_vtxo_ids
        1,           // num_specific_vtxo_ids > 0
        &mut status_json_out_ptr,
    );

    assert!(!error_ptr.is_null(), "Expected error for null specific_vtxo_ids with num_specific_vtxo_ids > 0 in bark_exit_start_specific");
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_specific_error_zero_num_specific_vtxo_ids() {
    let test_name = "exit_start_specific_error_zero_num_specific_vtxo_ids";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()]; // Dummy
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(), // Can be non-null
        0,                     // Test zero num_specific_vtxo_ids
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for zero num_specific_vtxo_ids in bark_exit_start_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy().contains("No VTXO IDs provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_specific_error_null_status_json_out() {
    let test_name = "exit_start_specific_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let vtxo_id_str = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let vtxo_id_c = c_string_for_test(vtxo_id_str);
    let vtxo_ids_ptr: [*const c_char; 1] = [vtxo_id_c.as_ptr()];

    let error_ptr = bark_exit_start_specific(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        vtxo_ids_ptr.as_ptr(),
        1,
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_exit_start_specific"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_all_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_all(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_exit_start_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_exit_start_all_error_null_mnemonic() {
    let test_name = "exit_start_all_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_start_all(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_exit_start_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_start_all_error_null_status_json_out() {
    let test_name = "exit_start_all_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_exit_start_all(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_exit_start_all"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_progress_once_error_null_datadir() {
    let mnemonic_c = c_string_for_test(VALID_MNEMONIC);
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_progress_once(
        ptr::null(), // Test null datadir
        mnemonic_c.as_ptr(),
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null datadir in bark_exit_progress_once"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null datadir: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
}

#[test]
fn test_bark_exit_progress_once_error_null_mnemonic() {
    let test_name = "exit_progress_once_error_null_mnemonic";
    let (temp_dir, datadir_c, _mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }
    let mut status_json_out_ptr: *mut c_char = ptr::null_mut();

    let error_ptr = bark_exit_progress_once(
        datadir_c.as_ptr(),
        ptr::null(), // Test null mnemonic
        &mut status_json_out_ptr,
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null mnemonic in bark_exit_progress_once"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null mnemonic: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    assert!(
        status_json_out_ptr.is_null(),
        "status_json_out_ptr should remain null on error"
    );
    cleanup_temp_wallet(&temp_dir);
}

#[test]
fn test_bark_exit_progress_once_error_null_status_json_out() {
    let test_name = "exit_progress_once_error_null_status_json_out";
    let (temp_dir, datadir_c, mnemonic_c, wallet_err_ptr) = setup_temp_wallet(test_name);
    if !wallet_err_ptr.is_null() {
        println!(
            "Note: Wallet creation failed in setup for {}: {}. This is acceptable.",
            test_name,
            unsafe { CStr::from_ptr(bark_error_message(wallet_err_ptr)).to_string_lossy() }
        );
        bark_free_error(wallet_err_ptr);
    }

    let error_ptr = bark_exit_progress_once(
        datadir_c.as_ptr(),
        mnemonic_c.as_ptr(),
        ptr::null_mut(), // Test null status_json_out
    );

    assert!(
        !error_ptr.is_null(),
        "Expected error for null status_json_out in bark_exit_progress_once"
    );
    if !error_ptr.is_null() {
        unsafe {
            let msg = CStr::from_ptr(bark_error_message(error_ptr));
            assert!(
                msg.to_string_lossy()
                    .contains("Null pointer argument provided"),
                "Unexpected error message for null status_json_out: {}",
                msg.to_string_lossy()
            );
            bark_free_error(error_ptr);
        }
    }
    cleanup_temp_wallet(&temp_dir);
}
