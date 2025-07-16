use std::{path::Path, str::FromStr};

use crate::GLOBAL_WALLET;
use anyhow::{self, bail, Context};
use bark::{
    ark::{
        bitcoin::{secp256k1::PublicKey, FeeRate, Network},
        rounds::RoundId,
        Vtxo, VtxoId,
    },
    lightning_invoice::Bolt11Invoice,
    lnurllib::lightning_address::LightningAddress,
    vtxo_selection::VtxoFilter,
    Config, SqliteClient, Wallet,
};

use logger::log::{debug, info, warn};
use tokio::fs;
use tonic::transport::Uri;

pub(crate) const DB_FILE: &str = "db.sqlite";

impl ConfigOpts {
    pub fn merge_into(&self, cfg: &mut Config) -> anyhow::Result<()> {
        if let Some(url) = &self.asp {
            cfg.asp_address = https_default_scheme(url.clone()).context("invalid asp url")?;
        }
        if let Some(v) = &self.esplora {
            cfg.esplora_address = match v.is_empty() {
                true => None,
                false => Some(https_default_scheme(v.clone()).context("invalid esplora url")?),
            };
        }
        if let Some(v) = &self.bitcoind {
            cfg.bitcoind_address = if v.is_empty() { None } else { Some(v.clone()) };
        }
        if let Some(v) = &self.bitcoind_cookie {
            cfg.bitcoind_cookiefile = if v.is_empty() {
                None
            } else {
                Some(v.clone().into())
            };
        }
        if let Some(v) = &self.bitcoind_user {
            cfg.bitcoind_user = if v.is_empty() { None } else { Some(v.clone()) };
        }
        if let Some(v) = &self.bitcoind_pass {
            cfg.bitcoind_pass = if v.is_empty() { None } else { Some(v.clone()) };
        }
        if cfg.esplora_address.is_none() && cfg.bitcoind_address.is_none() {
            bail!("Provide either an esplora or bitcoind url as chain source.");
        }

        cfg.vtxo_refresh_expiry_threshold = self.vtxo_refresh_expiry_threshold;
        if self.fallback_fee_rate.is_some() {
            cfg.fallback_fee_rate = self.fallback_fee_rate;
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

pub struct ConfigOpts {
    pub asp: Option<String>,

    /// The esplora HTTP API endpoint
    pub esplora: Option<String>,
    /// The bitcoind address
    pub bitcoind: Option<String>,
    pub bitcoind_cookie: Option<String>,
    pub bitcoind_user: Option<String>,
    pub bitcoind_pass: Option<String>,
    pub vtxo_refresh_expiry_threshold: u32,
    pub fallback_fee_rate: Option<FeeRate>,
}
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
    mnemonic: bip39::Mnemonic,
    birthday: Option<u32>,
) -> anyhow::Result<()> {
    info!("Creating new bark Wallet at {}", datadir.display());

    fs::create_dir_all(datadir)
        .await
        .context("can't create dir")?;

    debug!("datadir {:?} ", datadir);
    debug!("network {:?}", net);
    debug!("config {:?}", config);

    // open db
    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::create(&mnemonic, net, config, db, birthday)
        .await
        .context("error creating wallet")?;
    Ok(())
}

// Internal function encapsulating refresh logic
pub async fn refresh_vtxos_internal(
    mode: RefreshMode,
    no_sync: bool,
) -> anyhow::Result<Option<RoundId>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let w = wallet_guard.as_mut().context("Wallet not loaded")?;

    if !no_sync {
        info!("Syncing wallet before refreshing VTXOs...");
        if let Err(e) = w.maintenance().await {
            warn!("Wallet maintenance sync error during refresh: {}", e);
        }
    }

    // Determine VTXOs to refresh based on the mode
    let vtxos_to_refresh: Vec<Vtxo> = match mode {
        RefreshMode::DefaultThreshold => {
            let threshold = w.config().vtxo_refresh_expiry_threshold;

            debug!(
                "Refreshing VTXOs expiring within default threshold: {} blocks",
                threshold
            );
            w.get_expiring_vtxos(threshold).await?
        }
        RefreshMode::ThresholdBlocks(blocks) => {
            debug!("Refreshing VTXOs expiring within {} blocks", blocks);
            w.get_expiring_vtxos(blocks).await?
        }
        RefreshMode::ThresholdHours(hours) => {
            let blocks = hours.saturating_mul(6); // Approx blocks per hour
            debug!(
                "Refreshing VTXOs expiring within {} hours ({} blocks)",
                hours, blocks
            );
            w.get_expiring_vtxos(blocks).await?
        }
        RefreshMode::Counterparty => {
            debug!("Refreshing all VTXOs with counterparty risk");
            let filter = VtxoFilter::new(&w).counterparty();
            w.vtxos_with(filter)
                .context("Failed to get counterparty VTXOs")?
        }
        RefreshMode::All => {
            debug!("Refreshing all VTXOs");
            w.vtxos().context("Failed to get all VTXOs")?
        }
        RefreshMode::Specific(ids) => {
            if ids.is_empty() {
                info!("No specific VTXO IDs provided for refresh.");
                return Ok(None); // Nothing to refresh
            }
            debug!("Refreshing {} specific VTXOs", ids.len());
            // Fetch Vtxo objects for the given ids
            // Need to handle potential errors if an ID doesn't exist
            let mut found_vtxos = Vec::with_capacity(ids.len());
            for id in ids {
                match w.get_vtxo_by_id(id) {
                    // Assuming get_vtxo_by_id exists and is synchronous as per current code
                    Ok(vtxo) => found_vtxos.push(vtxo),
                    Err(e) => {
                        // Log or potentially error out if strict matching is required
                        warn!("Could not find VTXO with id {} for refresh: {}", id, e);
                        // Decide whether to continue or fail; let's continue for now
                    }
                }
            }
            if found_vtxos.is_empty() {
                info!("None of the specified VTXO IDs were found.");
                return Ok(None);
            }
            found_vtxos
        }
    };

    if vtxos_to_refresh.is_empty() {
        info!("No VTXOs found matching refresh criteria.");
        return Ok(None);
    }

    info!("Attempting to refresh {} VTXOs...", vtxos_to_refresh.len());
    let round_id: Option<RoundId> = w.refresh(&vtxos_to_refresh, true).await?;

    if let Some(id) = &round_id {
        info!("Participating in refresh round: {}", id);
    } else {
        info!("No refresh round participation necessary or possible at this time.");
    }

    Ok(round_id)
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
