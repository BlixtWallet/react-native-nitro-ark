#![allow(unused_imports)]
use crate::cxx::{
    self,
    ffi::{self, RefreshModeType},
};
use anyhow::Context;
use bark::ark::bitcoin::Amount;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::tempdir;

// --- Test Setup ---

/// Creates a temporary directory and basic wallet creation options for tests.
fn setup_test_wallet_opts() -> (tempfile::TempDir, ffi::CreateOpts) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let mnemonic = cxx::create_mnemonic().expect("Failed to create mnemonic for test");

    let config_opts = ffi::ConfigOpts {
        // Using placeholder values for services not directly hit in most unit tests.
        // For real integration tests, these would point to live regtest services.
        ark: "http://127.0.0.1:50051".to_string(),
        esplora: "http://127.0.0.1:3002".to_string(),
        bitcoind: "".to_string(),
        bitcoind_cookie: "".to_string(),
        bitcoind_user: "".to_string(),
        bitcoind_pass: "".to_string(),
        vtxo_refresh_expiry_threshold: 3600,
        fallback_fee_rate: 1,
        htlc_recv_claim_delta: 18,
        vtxo_exit_margin: 12,
        deep_round_confirmations: 0,
    };

    let create_opts = ffi::CreateOpts {
        regtest: true,
        signet: false,
        bitcoin: false,
        mnemonic,
        birthday_height: std::ptr::null(),
        config: config_opts,
    };

    (temp_dir, create_opts)
}

/// A test fixture to ensure the wallet is loaded for a test and closed afterward.
struct WalletTestFixture {
    _temp_dir: tempfile::TempDir,
}

impl WalletTestFixture {
    fn new() -> Self {
        cxx::init_logger();
        let (temp_dir, opts) = setup_test_wallet_opts();
        let datadir_str = temp_dir.path().to_str().unwrap();

        if cxx::is_wallet_loaded() {
            cxx::close_wallet().unwrap();
        }

        cxx::create_wallet(datadir_str, opts)
            .with_context(|| format!("Failed to load wallet in test setup"))
            .unwrap();

        WalletTestFixture {
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for WalletTestFixture {
    fn drop(&mut self) {
        if cxx::is_wallet_loaded() {
            cxx::close_wallet().expect("Failed to close wallet in test teardown");
        }
    }
}

// --- Tests ---

#[test]
fn test_init_logger_ffi() {
    // This just ensures the function can be called without panicking.
    // The logger is initialized globally, so this will be a no-op on subsequent calls.
    cxx::init_logger();
}

#[test]
fn test_create_mnemonic_ffi() {
    cxx::init_logger();
    let result = cxx::create_mnemonic();
    assert!(result.is_ok());
    let mnemonic_str = result.unwrap();
    assert_eq!(mnemonic_str.split_whitespace().count(), 12);
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_wallet_management_ffi() {
    cxx::init_logger();
    let (temp_dir, opts) = setup_test_wallet_opts();
    let datadir_str = temp_dir.path().to_str().unwrap();

    // 1. Wallet should not be loaded initially
    assert!(!cxx::is_wallet_loaded());

    // 2. Load wallet
    let load_result = cxx::create_wallet(datadir_str, opts);
    assert!(
        load_result.is_ok(),
        "Failed to load wallet: {:?}",
        load_result.err()
    );
    assert!(cxx::is_wallet_loaded());

    // 3. Try loading again (should fail)
    let (_temp_dir2, opts2) = setup_test_wallet_opts();
    let datadir_str2 = _temp_dir2.path().to_str().unwrap();
    let load_again_result = cxx::create_wallet(datadir_str2, opts2);
    assert!(
        load_again_result.is_err(),
        "Should not be able to load a second wallet"
    );

    // 4. Close wallet
    let close_result = cxx::close_wallet();
    assert!(close_result.is_ok());
    assert!(!cxx::is_wallet_loaded());

    // 5. Try closing again (should fail)
    let close_again_result = cxx::close_wallet();
    assert!(
        close_again_result.is_err(),
        "Should not be able to close a non-loaded wallet"
    );
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_get_onchain_address_ffi() {
    let _fixture = WalletTestFixture::new();
    let address_result = cxx::onchain_address();
    assert!(address_result.is_ok());
    let address = address_result.unwrap();
    assert!(
        address.starts_with("bcrt1"),
        "Address should be a regtest address"
    );
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_get_onchain_balance_ffi() {
    let _fixture = WalletTestFixture::new();
    // Use no_sync = true to avoid network calls in a unit test context.
    let balance_result = cxx::onchain_balance();
    assert!(balance_result.is_ok());
    let balance = balance_result.unwrap().confirmed;
    assert_eq!(balance, 0);
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_get_vtxo_pubkey_ffi() {
    let _fixture = WalletTestFixture::new();
    // Request the next available pubkey
    let _fixture = WalletTestFixture::new();
    // On a fresh wallet, these should return empty JSON arrays.
    let onchain_utxos_res = cxx::onchain_utxos();
    assert!(onchain_utxos_res.is_ok());
    assert_eq!(onchain_utxos_res.unwrap(), "[]");

    let vtxos_res = cxx::derive_store_next_keypair();
    assert!(vtxos_res.is_ok());
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_bolt11_invoice_ffi() {
    let _fixture = WalletTestFixture::new();
    // This test requires a running LDK node, which is part of the wallet.
    // It should succeed even without onchain funds.
    let amount_msat = 100_000; // 100 sat
    let invoice_res = cxx::bolt11_invoice(amount_msat);
    assert!(
        invoice_res.is_ok(),
        "Failed to create bolt11 invoice: {:?}",
        invoice_res.err()
    );
    let invoice_str = invoice_res.unwrap().bolt11_invoice;
    assert!(
        invoice_str.starts_with("lnbcrt1"),
        "Invoice should be for regtest"
    );
}

#[test]
#[ignore = "requires live regtest backend"]
fn test_onchain_and_boarding_flow_ffi() {
    let _fixture = WalletTestFixture::new();
    // This is an integration test and requires a funded regtest node.
    // 1. Get address
    let _address = cxx::onchain_address().unwrap();

    // (Manual step: fund this address from bitcoind-cli)
    // e.g., `bitcoin-cli -regtest sendtoaddress <address> 1`
    // (Manual step: mine a block)
    // e.g., `bitcoin-cli -regtest -generate 1`

    // 2. Check balance (with sync)
    let balance = cxx::onchain_balance().unwrap().confirmed;
    assert!(
        balance > 0,
        "Wallet should have onchain funds after funding and syncing"
    );

    // 3. Board amount
    let board_amount_sat = 50_000;
    let board_res = cxx::board_amount(board_amount_sat);
    assert!(board_res.is_ok(), "Boarding failed: {:?}", board_res.err());

    // (Manual step: mine the board transaction)

    // 4. Check balance again
    let final_balance = cxx::onchain_balance().unwrap().confirmed;
    assert!(
        final_balance >= board_amount_sat,
        "On chain balance should increase after boarding"
    );
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet"]
fn test_send_onchain_ffi() {
    let _fixture = WalletTestFixture::new();
    let address = cxx::onchain_address().unwrap();

    // This test requires the address to be funded manually.
    let send_res = cxx::onchain_send(&address, 5000, std::ptr::null());
    assert!(
        send_res.is_ok(),
        "send_onchain failed: {:?}",
        send_res.err()
    );
    let txid = send_res.unwrap();
    assert_eq!(txid.txid.len(), 64);
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet"]
fn test_drain_onchain_ffi() {
    let _fixture = WalletTestFixture::new();
    let address = cxx::onchain_address().unwrap();

    // This test requires the address to be funded manually.
    let drain_res = cxx::onchain_drain(&address, std::ptr::null());
    assert!(
        drain_res.is_ok(),
        "drain_onchain failed: {:?}",
        drain_res.err()
    );
    let txid = drain_res.unwrap();
    assert_eq!(txid.len(), 64);
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet"]
fn test_send_many_onchain_ffi() {
    let _fixture = WalletTestFixture::new();
    let address1 = cxx::onchain_address().unwrap();
    let address2 = cxx::onchain_address().unwrap();

    let outputs = vec![
        ffi::SendManyOutput {
            destination: address1,
            amount_sat: 5000,
        },
        ffi::SendManyOutput {
            destination: address2,
            amount_sat: 6000,
        },
    ];

    let send_many_res = cxx::onchain_send_many(outputs, std::ptr::null());
    assert!(
        send_many_res.is_ok(),
        "send_many failed: {:?}",
        send_many_res.err()
    );
    let txid = send_many_res.unwrap();
    assert_eq!(txid.len(), 64);
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet"]
fn test_board_all_ffi() {
    let _fixture = WalletTestFixture::new();
    // Requires wallet to be funded.
    let board_all_res = cxx::board_all();
    assert!(
        board_all_res.is_ok(),
        "board_all failed: {:?}",
        board_all_res.err()
    );
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet with vtxos"]
fn test_send_arkoot_payment_ffi() {
    let _fixture = WalletTestFixture::new();
    // This is a complex test as it can handle different destination types.
    // Here we test sending to a VTXO pubkey (OOR).
    let keypair = cxx::derive_store_next_keypair().unwrap();
    let send_res = cxx::send_arkoor_payment(&keypair.public_key, 5000);
    assert!(
        send_res.is_ok(),
        "send_payment (OOR) failed: {:?}",
        send_res.err()
    );
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet with vtxos"]
fn test_send_bolt11_payment_ffi() {
    let _fixture = WalletTestFixture::new();
    // This is a complex test as it can handle different destination types.
    // Here we test sending to a bolt11 invoice.
    let invoice = cxx::bolt11_invoice(10000).unwrap();
    let amount: u64 = 5000;
    let send_res = cxx::pay_lightning_invoice(&invoice.bolt11_invoice, &amount as *const u64);
    assert!(
        send_res.is_ok(),
        "send_payment (bolt11) failed: {:?}",
        send_res.err()
    );
}

#[test]
#[ignore = "requires live regtest backend and a funded wallet with vtxos"]
fn test_offboard_ffi() {
    let _fixture = WalletTestFixture::new();
    // This test would require creating VTXOs first.
    // We test that the call with no VTXOs doesn't panic.
    let offboard_all_res = cxx::offboard_all("");
    assert!(offboard_all_res.is_ok());

    let offboard_specific_res = cxx::offboard_specific(vec![], "");
    assert!(offboard_specific_res.is_ok());
}

#[test]
#[ignore = "requires live regtest backend with a funded lightning node"]
fn test_claim_bolt11_payment_ffi() {
    let _fixture = WalletTestFixture::new();
    // This requires another LN node to pay an invoice generated by our wallet.
    let invoice = cxx::bolt11_invoice(10000).unwrap();
    // In a real test, you would now pay this invoice from another node.
    // For this unit test, we just check that trying to claim an unpaid invoice fails gracefully.
    let claim_res =
        cxx::try_claim_lightning_receive(invoice.payment_hash, false, std::ptr::null());
    // Depending on the LDK setup, this might error differently.
    // The key is that it shouldn't panic.
    assert!(claim_res.is_err(), "Claiming an unpaid invoice should fail");
}
