use crate::utils;
use bark::ark::bitcoin::Address;
use std::path::Path;
use std::str::FromStr;

#[cxx::bridge(namespace = "bark_cxx")]
pub(crate) mod ffi {
    pub struct CxxArkInfo {
        network: String,
        asp_pubkey: String,
        round_interval_secs: u64,
        vtxo_exit_delta: u16,
        vtxo_expiry_delta: u16,
        htlc_expiry_delta: u16,
        max_vtxo_amount_sat: u64,
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
        birthday_height: *const u32,
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

    extern "Rust" {
        fn init_logger();
        fn create_mnemonic() -> Result<String>;
        fn is_wallet_loaded() -> bool;
        fn close_wallet() -> Result<()>;
        fn persist_config(opts: ConfigOpts) -> Result<()>;
        fn get_ark_info() -> Result<CxxArkInfo>;
        fn get_onchain_address() -> Result<String>;
        fn offchain_balance() -> Result<u64>;
        fn onchain_balance() -> Result<u64>;
        fn get_onchain_utxos(no_sync: bool) -> Result<String>;
        unsafe fn get_vtxo_pubkey(index: *const u32) -> Result<String>;
        fn get_vtxos(no_sync: bool) -> Result<String>;
        fn bolt11_invoice(amount_msat: u64) -> Result<String>;
        fn claim_bolt11_payment(bolt11: &str) -> Result<()>;
        fn maintenance() -> Result<()>;
        fn sync() -> Result<()>;
        fn sync_ark() -> Result<()>;
        fn sync_rounds() -> Result<()>;
        fn load_wallet(datadir: &str, opts: CreateOpts) -> Result<()>;
        fn send_onchain(destination: &str, amount_sat: u64, no_sync: bool) -> Result<String>;
        fn drain_onchain(destination: &str, no_sync: bool) -> Result<String>;
        fn send_many_onchain(outputs: Vec<SendManyOutput>, no_sync: bool) -> Result<String>;
        fn board_amount(amount_sat: u64) -> Result<String>;
        fn board_all() -> Result<String>;
        fn send_arkoor_payment(destination: &str, amount_sat: u64) -> Result<String>;
        unsafe fn send_bolt11_payment(destination: &str, amount_sat: *const u64) -> Result<String>;
        fn send_lnaddr(addr: &str, amount_sat: u64, comment: &str) -> Result<String>;
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

pub(crate) fn persist_config(opts: ffi::ConfigOpts) -> anyhow::Result<()> {
    let config_opts = utils::ConfigOpts {
        asp: Some(opts.asp),
        esplora: Some(opts.esplora),
        bitcoind: Some(opts.bitcoind),
        bitcoind_cookie: Some(opts.bitcoind_cookie),
        bitcoind_user: Some(opts.bitcoind_user),
        bitcoind_pass: Some(opts.bitcoind_pass),
        vtxo_refresh_expiry_threshold: opts.vtxo_refresh_expiry_threshold,
        fallback_fee_rate: Some(bark::ark::bitcoin::FeeRate::from_sat_per_vb_unchecked(
            opts.fallback_fee_rate,
        )),
    };

    crate::TOKIO_RUNTIME.block_on(async {
        let mut wallet_guard = crate::GLOBAL_WALLET.lock().await;
        let w = wallet_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Wallet not loaded"))?;
        let mut current_config = w.config().clone();
        config_opts.merge_into(&mut current_config)?;
        crate::persist_config(current_config).await
    })
}

pub(crate) fn get_ark_info() -> anyhow::Result<ffi::CxxArkInfo> {
    let info = crate::TOKIO_RUNTIME.block_on(crate::get_ark_info())?;
    Ok(ffi::CxxArkInfo {
        network: info.network.to_string(),
        asp_pubkey: info.asp_pubkey.to_string(),
        round_interval_secs: info.round_interval.as_secs(),
        vtxo_exit_delta: info.vtxo_exit_delta,
        vtxo_expiry_delta: info.vtxo_expiry_delta,
        htlc_expiry_delta: info.htlc_expiry_delta,
        max_vtxo_amount_sat: info.max_vtxo_amount.map_or(0, |a| a.to_sat()),
    })
}

pub(crate) fn offchain_balance() -> anyhow::Result<u64> {
    let balance = crate::TOKIO_RUNTIME.block_on(crate::offchain_balance())?;
    Ok(balance.to_sat())
}

pub(crate) fn onchain_balance() -> anyhow::Result<u64> {
    let balance = crate::TOKIO_RUNTIME.block_on(crate::onchain_balance())?;
    Ok(balance.to_sat())
}

pub(crate) fn get_onchain_utxos(no_sync: bool) -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::get_onchain_utxos(no_sync))
}

pub(crate) fn get_vtxo_pubkey(index: *const u32) -> anyhow::Result<String> {
    let index_opt = unsafe { index.as_ref().map(|r| *r) };
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

pub(crate) fn maintenance() -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::maintenance())
}

pub(crate) fn sync() -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::sync())
}

pub(crate) fn sync_ark() -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::sync_ark())
}

pub(crate) fn sync_rounds() -> anyhow::Result<()> {
    crate::TOKIO_RUNTIME.block_on(crate::sync_rounds())
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
        birthday_height: unsafe { opts.birthday_height.as_ref().map(|r| *r) },
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

pub(crate) fn board_amount(amount_sat: u64) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    crate::TOKIO_RUNTIME.block_on(crate::board_amount(amount))
}

pub(crate) fn board_all() -> anyhow::Result<String> {
    crate::TOKIO_RUNTIME.block_on(crate::board_all())
}

pub(crate) fn send_arkoor_payment(destination: &str, amount_sat: u64) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    crate::TOKIO_RUNTIME.block_on(crate::send_arkoor_payment(destination, amount))
}

pub(crate) fn send_bolt11_payment(
    destination: &str,
    amount_sat: *const u64,
) -> anyhow::Result<String> {
    let amount_opt = match unsafe { amount_sat.as_ref().map(|r| *r) } {
        Some(amount) => Some(bark::ark::bitcoin::Amount::from_sat(amount)),
        None => None,
    };

    crate::TOKIO_RUNTIME.block_on(crate::send_bolt11_payment(destination, amount_opt))
}

pub(crate) fn send_lnaddr(addr: &str, amount_sat: u64, comment: &str) -> anyhow::Result<String> {
    let amount = bark::ark::bitcoin::Amount::from_sat(amount_sat);
    let comment_opt = if comment.is_empty() {
        None
    } else {
        Some(comment)
    };
    crate::TOKIO_RUNTIME.block_on(crate::send_lnaddr(addr, amount, comment_opt))
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
