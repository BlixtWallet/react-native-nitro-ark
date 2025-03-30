use anyhow;
use anyhow::bail;
use bark;
use bark::ark::bitcoin::Network;
use bark::ark::BlockHeight;
use bark::Config;
use bark::SqliteClient;
use bark::Wallet;
mod ffi;
use logger::log::{debug, info, warn};
use std::fs;
use std::path::Path;
const DB_FILE: &str = "db.sqlite";
const MNEMONIC_FILE: &str = "mnemonic";

use std::io::{self, Write};
use std::str::FromStr;

use anyhow::Context;
use serde::Serialize;
use serde_json;
use tonic::transport::Uri;

impl ConfigOpts {
    fn merge_into(self, cfg: &mut Config) -> anyhow::Result<()> {
        if let Some(url) = self.asp {
            cfg.asp_address = https_default_scheme(url).context("invalid asp url")?;
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

/// Writes a [`Serializable`] value to stdout
pub fn output_json<T>(value: &T) -> ()
where
    T: ?Sized + Serialize,
{
    serde_json::to_writer_pretty(io::stdout(), value).expect("value is serializable");
    write!(io::stdout(), "\n").expect("Failed to write newline to stdout");
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
    pub mnemonic: Option<bip39::Mnemonic>,

    /// The wallet/mnemonic's birthday blockheight to start syncing when recovering.
    pub birthday_height: Option<BlockHeight>,

    pub config: ConfigOpts,
}

pub async fn create_wallet(datadir: &Path, opts: CreateOpts) -> anyhow::Result<()> {
    debug!("Creating wallet in {}", datadir.display());

    let net = match (opts.bitcoin, opts.signet, opts.regtest) {
        (true, false, false) => Network::Bitcoin,
        (false, true, false) => Network::Signet,
        (false, false, true) => Network::Regtest,
        _ => bail!("A network must be specified. Use either --signet, --regtest or --bitcoin"),
    };

    let mut config = Config {
        // required args
        asp_address: opts
            .config
            .asp
            .clone()
            .context("ASP address missing, use --asp")?,
        ..Default::default()
    };
    opts.config
        .merge_into(&mut config)
        .context("invalid configuration")?;

    // check if dir doesn't exists, then create it
    if datadir.exists() {
        if opts.force {
            fs::remove_dir_all(datadir)?;
        } else {
            bail!("Directory {} already exists", datadir.display());
        }
    }

    if opts.mnemonic.is_some() {
        if opts.birthday_height.is_none() {
            bail!("You need to set the --birthday-height field when recovering from mnemonic.");
        }
        warn!("Recovering from mnemonic currently only supports recovering on-chain funds!");
    } else {
        if opts.birthday_height.is_some() {
            bail!("Can't set --birthday-height if --mnemonic is not set.");
        }
    }

    // Everything that errors after this will wipe the datadir again.
    if let Err(e) =
        try_create_wallet(&datadir, net, config, opts.mnemonic, opts.birthday_height).await
    {
        // Remove the datadir if it exists
        if datadir.exists() {
            if let Err(e) = fs::remove_dir_all(datadir) {
                warn!("Failed to remove '{}", datadir.display());
                warn!("{}", e.to_string());
            }
        }
        bail!("Error while creating wallet: {:?}", e);
    }
    Ok(())
}

/// In this method we create the wallet and if it fails, the datadir will be wiped again.
async fn try_create_wallet(
    datadir: &Path,
    net: Network,
    config: Config,
    mnemonic: Option<bip39::Mnemonic>,
    birthday: Option<BlockHeight>,
) -> anyhow::Result<()> {
    info!("Creating new bark Wallet at {}", datadir.display());

    fs::create_dir_all(datadir).context("can't create dir")?;

    // generate seed
    let mnemonic = mnemonic.unwrap_or_else(|| bip39::Mnemonic::generate(12).expect("12 is valid"));
    fs::write(datadir.join(MNEMONIC_FILE), mnemonic.to_string().as_bytes())
        .context("failed to write mnemonic")?;

    info!("Mnemonic is {:?} ", mnemonic.to_string());

    // open db
    let db = SqliteClient::open(datadir.join(DB_FILE))?;

    Wallet::create(&mnemonic, net, config, db, birthday)
        .await
        .context("error creating wallet")?;

    Ok(())
}

pub struct Balance {
    pub onchain: u64,
    pub offchain: u64,
    pub pending_exit: u64
}

/// Get offchain and onchain balances
pub async fn get_balance(datadir: &Path, no_sync: bool) -> anyhow::Result<Balance> {
    let mut w = open_wallet(&datadir).await.context("error opening wallet")?;


    if !no_sync {
        info!("Syncing wallet...");
        if let Err(e) = w.sync().await {
            warn!("Sync error: {}", e)
        }
    }

    let onchain = w.onchain.balance().to_sat();
    let offchain =  w.offchain_balance().await?.to_sat();
    let pending_exit = w.exit.pending_total().await?.to_sat();

    
    let balances = Balance { onchain, offchain, pending_exit };
    Ok(balances)
}

pub async fn open_wallet(datadir: &Path) -> anyhow::Result<Wallet<SqliteClient>> {
	debug!("Opening bark wallet in {}", datadir.display());

	// read mnemonic file
	let mnemonic_path = datadir.join(MNEMONIC_FILE);
	let mnemonic_str = fs::read_to_string(&mnemonic_path)
		.with_context(|| format!("failed to read mnemonic file at {}", mnemonic_path.display()))?;
	let mnemonic = bip39::Mnemonic::from_str(&mnemonic_str).context("broken mnemonic")?;

	let db = SqliteClient::open(datadir.join(DB_FILE))?;

	Wallet::open(&mnemonic, db).await
}

