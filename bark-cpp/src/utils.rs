use std::{path::Path, str::FromStr};

use anyhow::{self, bail, Context};
use bark::{ark::bitcoin::Network, Config, SqliteClient, Wallet};
use log::{debug, info};
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
}
pub struct CreateOpts {
    /// Force re-create the wallet even if it already exists.
    /// Any funds in the old wallet will be lost
    pub force: bool,

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
    pub birthday_height: Option<u64>,

    pub config: ConfigOpts,
}

/// In this method we create the wallet and if it fails, the datadir will be wiped again.
pub(crate) async fn try_create_wallet(
    datadir: &Path,
    net: Network,
    config: Config,
    mnemonic: bip39::Mnemonic,
    birthday: Option<u64>,
) -> anyhow::Result<()> {
    info!("Creating new bark Wallet at {}", datadir.display());

    fs::create_dir_all(datadir)
        .await
        .context("can't create dir")?;

    debug!("datadir {:?} ", datadir);
    debug!("network {:?}", net);
    debug!("config {} {:?}", config.asp_address, config.esplora_address);

    // open db
    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::create(&mnemonic, net, config, db, birthday)
        .await
        .context("error creating wallet")?;
    Ok(())
}
