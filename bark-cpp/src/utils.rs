use std::{path::Path, str::FromStr, sync::Arc};

use anyhow::{self, bail, Context};
use bark::{
    ark::{
        bitcoin::{secp256k1::PublicKey, FeeRate, Network},
        VtxoId,
    },
    lightning_invoice::Bolt11Invoice,
    lnurllib::lightning_address::LightningAddress,
    onchain::OnchainWallet,
    Config, SqliteClient, Wallet as BarkWallet,
};

use bitcoin_ext::FeeRateExt;
use logger::log::{debug, info};
use tokio::fs;
use tonic::transport::Uri;

use crate::cxx::ffi;

pub(crate) const DB_FILE: &str = "db.sqlite";

impl ConfigOpts {
    pub fn merge_into(self, cfg: &mut Config) -> anyhow::Result<()> {
        if let Some(url) = self.ark {
            cfg.server_address = https_default_scheme(url).context("invalid ark url")?;
        }
        if let Some(v) = self.esplora {
            cfg.esplora_address = match v.is_empty() {
                true => None,
                false => Some(https_default_scheme(v).context("invalid esplora url")?),
            };
        }
        if let Some(v) = self.bitcoind {
            cfg.bitcoind_address = if v == "" { None } else { Some(v) };
        }
        if let Some(v) = self.bitcoind_cookie {
            cfg.bitcoind_cookiefile = if v == "" { None } else { Some(v.into()) };
        }
        if let Some(v) = self.bitcoind_user {
            cfg.bitcoind_user = if v == "" { None } else { Some(v) };
        }
        if let Some(v) = self.bitcoind_pass {
            cfg.bitcoind_pass = if v == "" { None } else { Some(v) };
        }
        if let Some(v) = self.vtxo_refresh_expiry_threshold {
            cfg.vtxo_refresh_expiry_threshold = v;
        }
        cfg.fallback_fee_rate = self
            .fallback_fee_rate
            .map(|f| FeeRate::from_sat_per_kvb_ceil(f));

        if cfg.esplora_address.is_none() && cfg.bitcoind_address.is_none() {
            bail!("Provide either an esplora or bitcoind url as chain source.");
        }

        Ok(())
    }
}

/// Parse the URL and add `https` scheme if no scheme is given.
pub fn https_default_scheme(url: String) -> anyhow::Result<String> {
    // default scheme to https if unset
    let mut uri_parts = Uri::from_str(&url).context("invalid url")?.into_parts();
    if uri_parts.authority.is_none() {
        bail!("invalid url '{}': missing authority", url);
    }
    if uri_parts.scheme.is_none() {
        uri_parts.scheme = Some("https".parse().unwrap());
        // because from_parts errors for missing PathAndQuery, set it
        uri_parts.path_and_query = Some(
            uri_parts
                .path_and_query
                .unwrap_or_else(|| "".parse().unwrap()),
        );
        let new = Uri::from_parts(uri_parts).unwrap();
        Ok(new.to_string())
    } else {
        Ok(url)
    }
}

#[derive(Debug, Clone)]
pub struct ConfigOpts {
    pub ark: Option<String>,

    /// The esplora HTTP API endpoint
    pub esplora: Option<String>,
    /// The bitcoind address
    pub bitcoind: Option<String>,
    pub bitcoind_cookie: Option<String>,
    pub bitcoind_user: Option<String>,
    pub bitcoind_pass: Option<String>,
    pub vtxo_refresh_expiry_threshold: Option<u32>,
    pub fallback_fee_rate: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CreateOpts {
    /// Use regtest network.
    pub regtest: bool,
    /// Use signet network.
    pub signet: bool,
    /// Use bitcoin mainnet
    pub bitcoin: bool,

    /// Recover a wallet with an existing mnemonic.
    /// This currently only works for on-chain funds.
    pub mnemonic: bip39::Mnemonic,

    /// The wallet/mnemonic's birthday blockheight to start syncing when recovering.
    pub birthday_height: Option<u32>,

    pub config: ConfigOpts,
}

pub enum RefreshMode {
    DefaultThreshold,
    ThresholdBlocks(u32),
    ThresholdHours(u32),
    Counterparty,
    All,
    Specific(Vec<VtxoId>),
}

/// In this method we create the wallet and if it fails, the datadir will be wiped again.
pub(crate) async fn try_create_wallet(
    datadir: &Path,
    net: Network,
    config: Config,
    mnemonic: Option<bip39::Mnemonic>,
) -> anyhow::Result<()> {
    info!("Creating new bark Wallet at {}", datadir.display());

    fs::create_dir_all(datadir)
        .await
        .context("can't create dir")?;

    debug!("try_create_wallet datadir {:?} ", datadir);
    debug!("try_create_walletnetwork {:?}", net);
    debug!("try_create_wallet config {:?}", config);

    // open db
    // generate seed
    let mnemonic = mnemonic.unwrap_or_else(|| bip39::Mnemonic::generate(12).expect("12 is valid"));
    let seed = mnemonic.to_seed("");

    // open db
    let db = Arc::new(SqliteClient::open(datadir.join(DB_FILE))?);

    let bdk_wallet = OnchainWallet::load_or_create(net, seed, db.clone())?;
    BarkWallet::create_with_onchain(&mnemonic, net, config, db, &bdk_wallet, false)
        .await
        .context("error creating wallet")?;

    Ok(())
}

/// Represents the different destinations for the `send` command
pub enum SendDestination {
    VtxoPubkey(PublicKey),
    Bolt11(Bolt11Invoice),
    LnAddress(LightningAddress),
    // Potentially add LNURL string later if direct LNURL payment is supported
}

/// Parses the destination string into a supported type.
pub fn parse_send_destination(destination: &str) -> anyhow::Result<SendDestination> {
    if let Ok(pk) = PublicKey::from_str(destination) {
        Ok(SendDestination::VtxoPubkey(pk))
    } else if let Ok(invoice) = Bolt11Invoice::from_str(destination) {
        // Further validation might be needed (e.g., expiry) but basic parsing is enough here
        Ok(SendDestination::Bolt11(invoice))
    } else if let Ok(lnaddr) = LightningAddress::from_str(destination) {
        Ok(SendDestination::LnAddress(lnaddr))
    } else {
        // Could check for raw lnurl string here if needed
        bail!(
            "Destination is not a valid VTXO pubkey, bolt11 invoice, or lightning address: {}",
            destination
        )
    }
}

/// Configuration of the Bark wallet.
/// Merge CreateOpts into ConfigOpts
pub fn merge_config_opts(opts: CreateOpts) -> anyhow::Result<(Config, Network)> {
    let net = match (opts.bitcoin, opts.signet, opts.regtest) {
        (true, false, false) => Network::Bitcoin,
        (false, true, false) => Network::Signet,
        (false, false, true) => Network::Regtest,
        _ => bail!("A network must be specified. Use either --signet, --regtest or --bitcoin"),
    };

    let mut config = Config {
        server_address: opts
            .config
            .ark
            .clone()
            .context("Ark server address missing, use --ark")?,
        ..Default::default()
    };
    opts.config
        .clone()
        .merge_into(&mut config)
        .context("invalid configuration")?;

    Ok((config, net))
}

pub fn ffi_config_to_config(opts: ffi::CreateOpts) -> anyhow::Result<CreateOpts> {
    let config_opts = ConfigOpts {
        ark: Some(opts.config.ark),
        esplora: Some(opts.config.esplora),
        bitcoind: Some(opts.config.bitcoind),
        bitcoind_cookie: Some(opts.config.bitcoind_cookie),
        bitcoind_user: Some(opts.config.bitcoind_user),
        bitcoind_pass: Some(opts.config.bitcoind_pass),
        vtxo_refresh_expiry_threshold: Some(opts.config.vtxo_refresh_expiry_threshold),
        fallback_fee_rate: Some(opts.config.fallback_fee_rate),
    };

    let create_opts = CreateOpts {
        regtest: opts.regtest,
        signet: opts.signet,
        bitcoin: opts.bitcoin,
        mnemonic: bip39::Mnemonic::from_str(&opts.mnemonic)?,
        birthday_height: unsafe { opts.birthday_height.as_ref().map(|r| *r) },
        config: config_opts,
    };

    Ok(create_opts)
}
