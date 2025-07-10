use crate::utils;
use bark::ark::bitcoin::Address;
use std::path::Path;
use std::str::FromStr;

#[cxx::bridge(namespace = "bark_cxx")]
pub(crate) mod ffi {
    pub struct CxxBalance {
        onchain: u64,
        offchain: u64,
        pending_exit: u64,
    }

    pub struct ConfigOpts {
        asp: String,
        esplora: String,
        bitcoind: String,
        bitcoind_cookie: String,
        bitcoind_user: String,
        bitcoind_pass: String,
        vtxo_refresh_expiry_threshold: u32,
        fallback_fee_rate: u64,
    }

    pub struct CreateOpts {
        regtest: bool,
        signet: bool,
        bitcoin: bool,
        mnemonic: String,
        birthday_height: u32,
        config: ConfigOpts,
    }

    pub struct SendManyOutput {
        destination: String,
        amount_sat: u64,
    }

    pub enum RefreshModeType {
        DefaultThreshold,
        ThresholdBlocks,
        ThresholdHours,
        Counterparty,
        All,
        Specific,
    }

    pub struct RefreshOpts {
        mode_type: RefreshModeType,
        threshold_value: u32,
        specific_vtxo_ids: Vec<String>,
    }

    extern "Rust" {
        fn init_logger();
        fn create_mnemonic() -> Result<String>;
        fn is_wallet_loaded() -> bool;
        fn close_wallet() -> Result<()>;
        fn get_onchain_address() -> Result<String>;
        fn get_balance(no_sync: bool) -> Result<CxxBalance>;
        fn get_onchain_utxos(no_sync: bool) -> Result<String>;
        fn get_vtxo_pubkey(index: u32) -> Result<String>;
        fn get_vtxos(no_sync: bool) -> Result<String>;
        fn bolt11_invoice(amount_msat: u64) -> Result<String>;
        fn claim_bolt11_payment(bolt11: &str) -> Result<()>;
        fn load_wallet(datadir: &str, opts: CreateOpts) -> Result<()>;
        fn send_onchain(destination: &str, amount_sat: u64, no_sync: bool) -> Result<String>;
        fn drain_onchain(destination: &str, no_sync: bool) -> Result<String>;
        fn send_many_onchain(outputs: Vec<SendManyOutput>, no_sync: bool) -> Result<String>;
        fn refresh_vtxos(opts: RefreshOpts, no_sync: bool) -> Result<String>;
        fn board_amount(amount_sat: u64, no_sync: bool) -> Result<String>;
        fn board_all(no_sync: bool) -> Result<String>;
        fn send_payment(
            destination: &str,
            amount_sat: u64,
            comment: &str,
            no_sync: bool,
        ) -> Result<String>;
        fn send_round_onchain(destination: &str, amount_sat: u64, no_sync: bool) -> Result<String>;
        fn offboard_specific(
            vtxo_ids: Vec<String>,
            destination_address: &str,
            no_sync: bool,
        ) -> Result<String>;
        fn offboard_all(destination_address: &str, no_sync: bool) -> Result<String>;
        fn start_exit_for_vtxos(vtxo_ids: Vec<String>) -> Result<String>;
        fn start_exit_for_entire_wallet() -> Result<String>;
        fn exit_progress_once() -> Result<String>;
    }
}

pub(crate) fn init_logger() {
    crate::init_logger()
}

pub(crate) fn create_mnemonic() -> anyhow::Result<String> {
    crate::create_mnemonic()
}

pub(crate) fn is_wallet_loaded() -> bool {
    crate::TOKIO_RUNTIME.block_on(crate::is_wallet_loaded())
}

pub(crate) fn close_wallet() -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::close_wallet())
}

pub(crate) fn get_onchain_address() -> anyhow::Result<String> {
    let address = crate::TOKIO_RUNTIME.block_on(crate::get_onchain_address())?;
    Ok(address.to_string())
}

pub(crate) fn get_balance(no_sync: bool) -> anyhow::Result<ffi::CxxBalance> {
    let balance = crate::TOKIO_RUNTIME.block_on(crate::get_balance(no_sync))?;
    Ok(ffi::CxxBalance {
        onchain: balance.onchain,
        offchain: balance.offchain,
        pending_exit: balance.pending_exit,
    })
}

pub(crate) fn get_onchain_utxos(no_sync: bool) -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::get_onchain_utxos(no_sync))
}

pub(crate) fn get_vtxo_pubkey(index: u32) -> anyhow::Result<String> {
    let index_opt = if index == u32::MAX { None } else { Some(index) };
    crate::TOKIO_RUNTIME.block_on(crate::get_vtxo_pubkey(index_opt))
}

pub(crate) fn get_vtxos(no_sync: bool) -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::get_vtxos(no_sync))
}

pub(crate) fn bolt11_invoice(amount_msat: u64) -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::bolt11_invoice(amount_msat))
}

pub(crate) fn claim_bolt11_payment(bolt11: &str) -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::claim_bolt11_payment(bolt11.to_string()))
}

pub(crate) fn load_wallet(datadir: &str, opts: ffi::CreateOpts) -> anyhow::Result<()> {
    let config_opts = utils::ConfigOpts {
        asp: Some(opts.config.asp),
        esplora: Some(opts.config.esplora),
        bitcoind: Some(opts.config.bitcoind),
        bitcoind_cookie: Some(opts.config.bitcoind_cookie),
        bitcoind_user: Some(opts.config.bitcoind_user),
        bitcoind_pass: Some(opts.config.bitcoind_pass),
        vtxo_refresh_expiry_threshold: opts.config.vtxo_refresh_expiry_threshold,
        fallback_fee_rate: Some(bark::ark::bitcoin::FeeRate::from_sat_per_vb_unchecked(
            opts.config.fallback_fee_rate,
        )),
    };

    let create_opts = utils::CreateOpts {
        regtest: opts.regtest,
        signet: opts.signet,
        bitcoin: opts.bitcoin,
        mnemonic: bip39::Mnemonic::from_str(&opts.mnemonic)?,
        birthday_height: if opts.birthday_height == 0 {
            None
        } else {
            Some(opts.birthday_height)
        },
        config: config_opts,
    };
    crate::TOKIO_RUNTIME.block_on(crate::load_wallet(Path::new(datadir), create_opts))
}

pub(crate) fn send_onchain(
    destination: &str,
    amount_sat: u64,
    no_sync: bool,
) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    let txid = crate::TOKIO_RUNTIME.block_on(crate::send_onchain(destination, amount, no_sync))?;
    Ok(txid.to_string())
}

pub(crate) fn drain_onchain(destination: &str, no_sync: bool) -> anyhow::Result<String> {
    let txid = crate::TOKIO_RUNTIME.block_on(crate::drain_onchain(destination, no_sync))?;
    Ok(txid.to_string())
}

pub(crate) fn send_many_onchain(
    outputs: Vec<ffi::SendManyOutput>,
    no_sync: bool,
) -> anyhow::Result<String> {
    let txid = crate::TOKIO_RUNTIME.block_on(async {
        let mut rust_outputs = Vec::new();
        let wallet_guard = crate::GLOBAL_WALLET.lock().await;
        let w = wallet_guard.as_ref().unwrap();
        let net = w.properties().unwrap().network;
        for output in outputs {
            let address = Address::from_str(&output.destination)
                .unwrap()
                .require_network(net)
                .unwrap();
            let amount = bark::ark::bitcoin::Amount::from_sat(output.amount_sat);
            rust_outputs.push((address, amount));
        }
        crate::send_many_onchain(rust_outputs, no_sync).await
    })?;
    Ok(txid.to_string())
}

pub(crate) fn refresh_vtxos(opts: ffi::RefreshOpts, no_sync: bool) -> anyhow::Result<String> {
    let rust_mode = match opts.mode_type {
        ffi::RefreshModeType::DefaultThreshold => crate::RefreshMode::DefaultThreshold,
        ffi::RefreshModeType::ThresholdBlocks => {
            crate::RefreshMode::ThresholdBlocks(opts.threshold_value)
        }
        ffi::RefreshModeType::ThresholdHours => {
            crate::RefreshMode::ThresholdHours(opts.threshold_value)
        }
        ffi::RefreshModeType::Counterparty => crate::RefreshMode::Counterparty,
        ffi::RefreshModeType::All => crate::RefreshMode::All,
        ffi::RefreshModeType::Specific => {
            let ids = opts
                .specific_vtxo_ids
                .into_iter()
                .map(|s| bark::ark::VtxoId::from_str(&s))
                .collect::<Result<Vec<_>, _>>()?;
            crate::RefreshMode::Specific(ids)
        }
        _ => return Err(anyhow::anyhow!("Unknown refresh mode")),
    };
    crate::TOKIO_RUNTIME.block_on(crate::refresh_vtxos(rust_mode, no_sync))
}

pub(crate) fn board_amount(amount_sat: u64, no_sync: bool) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    crate::TOKIO_RUNTIME.block_on(crate::board_amount(amount, no_sync))
}

pub(crate) fn board_all(no_sync: bool) -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::board_all(no_sync))
}

pub(crate) fn send_payment(
    destination: &str,
    amount_sat: u64,
    comment: &str,
    no_sync: bool,
) -> anyhow::Result<String> {
    let amount_opt = if amount_sat == 0 {
        None
    } else {
        Some(amount_sat)
    };
    let comment_opt = if comment.is_empty() {
        None
    } else {
        Some(comment.to_string())
    };
    crate::TOKIO_RUNTIME.block_on(crate::send_payment(
        destination,
        amount_opt,
        comment_opt,
        no_sync,
    ))
}

pub(crate) fn send_round_onchain(
    destination: &str,
    amount_sat: u64,
    no_sync: bool,
) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    crate::TOKIO_RUNTIME.block_on(crate::send_round_onchain(destination, amount, no_sync))
}

pub(crate) fn offboard_specific(
    vtxo_ids: Vec<String>,
    destination_address: &str,
    no_sync: bool,
) -> anyhow::Result<String> {
    let ids = vtxo_ids
        .into_iter()
        .map(|s| bark::ark::VtxoId::from_str(&s))
        .collect::<Result<Vec<_>, _>>()?;
    let address_opt = if destination_address.is_empty() {
        None
    } else {
        Some(destination_address.to_string())
    };
    crate::TOKIO_RUNTIME.block_on(crate::offboard_specific(ids, address_opt, no_sync))
}

pub(crate) fn offboard_all(destination_address: &str, no_sync: bool) -> anyhow::Result<String> {
    let address_opt = if destination_address.is_empty() {
        None
    } else {
        Some(destination_address.to_string())
    };
    crate::TOKIO_RUNTIME.block_on(crate::offboard_all(address_opt, no_sync))
}

pub(crate) fn start_exit_for_vtxos(vtxo_ids: Vec<String>) -> anyhow::Result<String> {
    let ids = vtxo_ids
        .into_iter()
        .map(|s| bark::ark::VtxoId::from_str(&s))
        .collect::<Result<Vec<_>, _>>()?;
    crate::TOKIO_RUNTIME.block_on(crate::start_exit_for_vtxos(ids))
}

pub(crate) fn start_exit_for_entire_wallet() -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::start_exit_for_entire_wallet())
}

pub(crate) fn exit_progress_once() -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::exit_progress_once())
}
