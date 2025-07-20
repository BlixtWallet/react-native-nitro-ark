use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use bark::ark::bitcoin::bip32::Fingerprint;
use bark::ark::bitcoin::consensus;
use bark::ark::bitcoin::hashes::Hash;
use bark::ark::bitcoin::secp256k1::PublicKey;
use bark::ark::bitcoin::{Amount, BlockHash, FeeRate, Network, Txid};
use bark::ark::{ProtocolEncoding, Vtxo, VtxoId};
use bark::exit::vtxo::ExitEntry;
use bark::json::exit::ExitState;
use bark::movement::Movement;
use bark::persist::{OffchainBoard, OffchainPayment};
use bark::vtxo_state::{VtxoState, VtxoStateKind, WalletVtxo};
use bark::{Config, KeychainKind, Pagination, WalletProperties};
use bitcoin_ext::*;
use libsql::{params, Connection, Transaction, Value};

use super::convert::{row_to_movement, row_to_offchain_board};

pub(crate) async fn set_properties(
    tx: &Transaction,
    properties: &WalletProperties,
) -> anyhow::Result<()> {
    let query = "INSERT INTO bark_properties (id, network, fingerprint) VALUES (1, ?1, ?2)";
    tx.execute(
        query,
        params![
            properties.network.to_string(),
            properties.fingerprint.to_string(),
        ],
    )
    .await?;
    Ok(())
}

pub(crate) async fn set_config(tx: &Transaction, config: &Config) -> anyhow::Result<()> {
    let query = "INSERT INTO bark_config (id, asp_address, esplora_address, bitcoind_address, bitcoind_cookiefile, bitcoind_user, bitcoind_pass, vtxo_refresh_expiry_threshold, fallback_fee_kwu) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT (id) DO UPDATE SET asp_address = ?1, esplora_address = ?2, bitcoind_address = ?3, bitcoind_cookiefile = ?4, bitcoind_user = ?5, bitcoind_pass = ?6, vtxo_refresh_expiry_threshold = ?7, fallback_fee_kwu = ?8";
    tx.execute(
        query,
        params![
            config.asp_address.clone(),
            config.esplora_address.clone(),
            config.bitcoind_address.clone(),
            config
                .bitcoind_cookiefile
                .clone()
                .and_then(|f| f.to_str().map(String::from)),
            config.bitcoind_user.clone(),
            config.bitcoind_pass.clone(),
            config.vtxo_refresh_expiry_threshold,
            config
                .fallback_fee_rate
                .map(|f| f.to_sat_per_kwu() as i64)
                .map(Value::Integer)
                .unwrap_or(Value::Null),
        ],
    )
    .await?;
    Ok(())
}

pub(crate) async fn fetch_properties(
    conn: &Connection,
) -> anyhow::Result<Option<WalletProperties>> {
    let query = "SELECT network, fingerprint FROM bark_properties";
    let mut rows = conn.query(query, ()).await?;
    if let Some(row) = rows.next().await? {
        let network: String = row.get(0)?;
        let fingerprint: String = row.get(1)?;
        Ok(Some(WalletProperties {
            network: Network::from_str(&network).context("invalid network")?,
            fingerprint: Fingerprint::from_str(&fingerprint).context("invalid fingerprint")?,
        }))
    } else {
        Ok(None)
    }
}

pub(crate) async fn fetch_config(conn: &Connection) -> anyhow::Result<Option<Config>> {
    let query = "SELECT asp_address, esplora_address, bitcoind_address, bitcoind_cookiefile, bitcoind_user, bitcoind_pass, vtxo_refresh_expiry_threshold, fallback_fee_kwu FROM bark_config";
    let mut rows = conn.query(query, ()).await?;
    if let Some(row) = rows.next().await? {
        let bitcoind_cookiefile_opt: Option<String> = row.get(3)?;
        let bitcoind_cookiefile = if let Some(bitcoind_cookiefile) = bitcoind_cookiefile_opt {
            Some(PathBuf::try_from(bitcoind_cookiefile)?)
        } else {
            None
        };
        let kwu_fee: Option<i64> = row.get(7)?;
        Ok(Some(Config {
            asp_address: row.get(0)?,
            esplora_address: row.get(1)?,
            bitcoind_address: row.get(2)?,
            bitcoind_cookiefile,
            bitcoind_user: row.get(4)?,
            bitcoind_pass: row.get(5)?,
            vtxo_refresh_expiry_threshold: row.get(6)?,
            fallback_fee_rate: kwu_fee.map(|f| FeeRate::from_sat_per_kwu(f as u64)),
        }))
    } else {
        Ok(None)
    }
}

pub(crate) async fn create_movement(
    tx: &Transaction,
    fees_sat: Option<Amount>,
) -> anyhow::Result<i64> {
    let query = "INSERT INTO bark_movement (fees_sat) VALUES (?1) RETURNING id";
    let mut rows = tx
        .query(query, params![fees_sat.unwrap_or(Amount::ZERO).to_sat()])
        .await?;
    let row = rows.next().await?.context("No rows returned")?;
    Ok(row.get(0)?)
}

pub(crate) async fn create_recipient(
    tx: &Transaction,
    movement: i64,
    recipient: &str,
    amount: Amount,
) -> anyhow::Result<i64> {
    let query =
        "INSERT INTO bark_recipient (movement, recipient, amount_sat) VALUES (?1, ?2, ?3) RETURNING id";
    let mut rows = tx
        .query(query, params![movement, recipient, amount.to_sat()])
        .await?;
    let row = rows.next().await?.context("No rows returned")?;
    Ok(row.get(0)?)
}

pub(crate) async fn check_recipient_exists(
    conn: &Connection,
    recipient: &str,
) -> anyhow::Result<bool> {
    let query = "SELECT COUNT(*) FROM bark_recipient WHERE recipient = ?1";
    let mut rows = conn.query(query, params![recipient]).await?;
    let row = rows.next().await?.context("No rows returned")?;
    Ok(row.get::<i64>(0)? > 0)
}

pub(crate) async fn get_paginated_movements(
    conn: &Connection,
    pagination: Pagination,
) -> anyhow::Result<Vec<Movement>> {
    let take = pagination.page_size;
    let skip = pagination.page_index * take;
    let query =
        "SELECT * FROM movement_view ORDER BY movement_view.created_at DESC LIMIT ?1 OFFSET ?2";
    let mut rows = conn.query(query, params![take, skip]).await?;
    let mut movements = Vec::with_capacity(take as usize);
    while let Some(row) = rows.next().await? {
        movements.push(row_to_movement(&row)?);
    }
    Ok(movements)
}

pub(crate) async fn store_vtxo_with_initial_state(
    tx: &Transaction,
    vtxo: &Vtxo,
    movement_id: i64,
    state: &VtxoState,
) -> anyhow::Result<()> {
    let q1 = "INSERT INTO bark_vtxo (id, expiry_height, amount_sat, received_in, raw_vtxo) VALUES (?1, ?2, ?3, ?4, ?5)";
    tx.execute(
        q1,
        params![
            vtxo.id().to_string(),
            vtxo.expiry_height(),
            vtxo.amount().to_sat(),
            movement_id,
            vtxo.serialize(),
        ],
    )
    .await?;
    let q2 = "INSERT INTO bark_vtxo_state (vtxo_id, state_kind, state) VALUES (?1, ?2, ?3)";
    tx.execute(
        q2,
        params![
            vtxo.id().to_string(),
            state.as_kind().as_str(),
            serde_json::to_vec(&state)?,
        ],
    )
    .await?;
    Ok(())
}

pub(crate) async fn get_wallet_vtxo_by_id(
    conn: &Connection,
    id: VtxoId,
) -> anyhow::Result<Option<WalletVtxo>> {
    let query = "SELECT raw_vtxo, state FROM vtxo_view WHERE id = ?1";
    let mut rows = conn.query(query, params![id.to_string()]).await?;
    if let Some(row) = rows.next().await? {
        let vtxo = Vtxo::deserialize(&row.get::<Vec<u8>>(0)?)?;
        let state = serde_json::from_slice::<VtxoState>(&row.get::<Vec<u8>>(1)?)?;
        Ok(Some(WalletVtxo { vtxo, state }))
    } else {
        Ok(None)
    }
}

pub(crate) async fn get_wallet_vtxo_by_id_in_tx(
    tx: &Transaction,
    id: VtxoId,
) -> anyhow::Result<Option<WalletVtxo>> {
    let query = "SELECT raw_vtxo, state FROM vtxo_view WHERE id = ?1";
    let mut rows = tx.query(query, params![id.to_string()]).await?;
    if let Some(row) = rows.next().await? {
        let vtxo = Vtxo::deserialize(&row.get::<Vec<u8>>(0)?)?;
        let state = serde_json::from_slice::<VtxoState>(&row.get::<Vec<u8>>(1)?)?;
        Ok(Some(WalletVtxo { vtxo, state }))
    } else {
        Ok(None)
    }
}

pub(crate) async fn get_vtxos_by_state(
    conn: &Connection,
    state: &[VtxoStateKind],
) -> anyhow::Result<Vec<WalletVtxo>> {
    let query = "SELECT raw_vtxo, state FROM vtxo_view WHERE state_kind IN (SELECT value FROM json_each(?1)) ORDER BY amount_sat DESC, expiry_height ASC";
    let mut rows = conn
        .query(query, params![serde_json::to_string(&state)?])
        .await?;
    let mut result = Vec::new();
    while let Some(row) = rows.next().await? {
        let vtxo = {
            let raw_vtxo: Vec<u8> = row.get(0)?;
            Vtxo::deserialize(&raw_vtxo)?
        };
        let state = {
            let raw_state: Vec<u8> = row.get(1)?;
            serde_json::from_slice::<VtxoState>(&raw_state)?
        };
        result.push(WalletVtxo { vtxo, state });
    }
    Ok(result)
}

pub(crate) async fn delete_vtxo(tx: &Transaction, id: VtxoId) -> anyhow::Result<Option<Vtxo>> {
    let query = "DELETE FROM bark_vtxo_state WHERE vtxo_id = ?1";
    tx.execute(query, params![id.to_string()]).await?;
    let query = "DELETE FROM bark_vtxo WHERE id = ?1 RETURNING raw_vtxo";
    let mut rows = tx.query(query, params![id.to_string()]).await?;
    let vtxo = if let Some(row) = rows.next().await? {
        let raw_vtxo: Vec<u8> = row.get(0)?;
        Some(Vtxo::deserialize(&raw_vtxo)?)
    } else {
        None
    };
    Ok(vtxo)
}

pub(crate) async fn get_vtxo_state(
    conn: &Connection,
    id: VtxoId,
) -> anyhow::Result<Option<VtxoState>> {
    let query =
        "SELECT state FROM bark_vtxo_state WHERE vtxo_id = ?1 ORDER BY created_at DESC LIMIT 1";
    let mut rows = conn.query(query, params![id.to_string()]).await?;
    if let Some(row) = rows.next().await? {
        let state = row.get::<Vec<u8>>(0)?;
        Ok(Some(serde_json::from_slice(&state)?))
    } else {
        Ok(None)
    }
}

pub(crate) async fn link_spent_vtxo_to_movement(
    tx: &Transaction,
    id: VtxoId,
    movement_id: i64,
) -> anyhow::Result<()> {
    let query = "UPDATE bark_vtxo SET spent_in = ?1 WHERE id = ?2";
    tx.execute(query, params![movement_id, id.to_string()])
        .await?;
    Ok(())
}

pub(crate) async fn update_vtxo_state_checked(
    tx: &Transaction,
    vtxo_id: VtxoId,
    new_state: VtxoState,
    old_states: &[VtxoStateKind],
) -> anyhow::Result<WalletVtxo> {
    let query = r"INSERT INTO bark_vtxo_state (vtxo_id, state_kind, state) SELECT ?1, ?2, ?3 FROM most_recent_vtxo_state WHERE vtxo_id = ?1 AND state_kind IN (SELECT value FROM json_each(?4))";
    let nb_inserted = tx
        .execute(
            query,
            params![
                vtxo_id.to_string(),
                new_state.as_kind().as_str(),
                serde_json::to_vec(&new_state)?,
                serde_json::to_string(old_states)?,
            ],
        )
        .await?;
    match nb_inserted {
        0 => anyhow::bail!("No vtxo with provided id or old states"),
        1 => Ok(get_wallet_vtxo_by_id_in_tx(tx, vtxo_id).await?.unwrap()),
        _ => panic!("Corrupted database. A vtxo can have only one state"),
    }
}

pub(crate) async fn store_vtxo_key(
    tx: &Transaction,
    keychain: KeychainKind,
    index: u32,
    public_key: PublicKey,
) -> anyhow::Result<()> {
    let query = "INSERT INTO bark_vtxo_key (keychain, idx, public_key) VALUES (?1, ?2, ?3)";
    tx.execute(
        query,
        params![keychain as i64, index, public_key.to_string()],
    )
    .await?;
    Ok(())
}

pub(crate) async fn get_vtxo_key(
    conn: &Connection,
    vtxo: &Vtxo,
) -> anyhow::Result<Option<(KeychainKind, u32)>> {
    let query = "SELECT keychain, idx FROM bark_vtxo_key WHERE public_key = (?1)";
    let pk = vtxo.user_pubkey().to_string();
    let mut rows = conn.query(query, params![pk]).await?;
    if let Some(row) = rows.next().await? {
        let index = u32::try_from(row.get::<i64>(1)?)?;
        let keychain = KeychainKind::try_from(row.get::<i64>(0)?)?;
        Ok(Some((keychain, index)))
    } else {
        Ok(None)
    }
}

pub(crate) async fn check_vtxo_key_exists(
    conn: &Connection,
    public_key: &PublicKey,
) -> anyhow::Result<bool> {
    let query = "SELECT idx FROM bark_vtxo_key WHERE public_key = (?1)";
    let mut rows = conn.query(query, params![public_key.to_string()]).await?;
    Ok(rows.next().await?.is_some())
}

pub(crate) async fn get_last_vtxo_key_index(
    conn: &Connection,
    keychain: KeychainKind,
) -> anyhow::Result<Option<u32>> {
    let query = "SELECT idx FROM bark_vtxo_key WHERE keychain = ?1 ORDER BY idx DESC LIMIT 1";
    let mut rows = conn.query(query, params![keychain as i64]).await?;
    if let Some(row) = rows.next().await? {
        let index = u32::try_from(row.get::<i64>(0)?)?;
        Ok(Some(index))
    } else {
        Ok(None)
    }
}

pub(crate) async fn store_last_ark_sync_height(
    tx: &Transaction,
    height: BlockHeight,
) -> anyhow::Result<()> {
    let query = "INSERT INTO bark_ark_sync (id, sync_height) VALUES (1, ?1) ON CONFLICT (id) DO UPDATE SET sync_height = ?1";
    tx.execute(query, params![height as i64]).await?;
    Ok(())
}

pub(crate) async fn get_last_ark_sync_height(conn: &Connection) -> anyhow::Result<BlockHeight> {
    let query = "SELECT sync_height FROM bark_ark_sync ORDER BY id DESC LIMIT 1";
    let mut rows = conn.query(query, ()).await?;
    if let Some(row) = rows.next().await? {
        let height_i64: i64 = row.get(0)?;
        let height = u32::try_from(height_i64)?;
        Ok(height)
    } else {
        Ok(0)
    }
}

pub(crate) async fn store_offchain_board(
    tx: &Transaction,
    payment_hash: &[u8; 32],
    preimage: &[u8; 32],
    payment: OffchainPayment,
) -> anyhow::Result<()> {
    let query = "INSERT INTO bark_offchain_board (payment_hash, preimage, serialised_payment) VALUES (?1, ?2, ?3)";
    tx.execute(
        query,
        params![
            payment_hash.to_vec(),
            preimage.to_vec(),
            serde_json::to_vec(&payment)?,
        ],
    )
    .await?;
    Ok(())
}

pub(crate) async fn fetch_offchain_board_by_payment_hash(
    conn: &Connection,
    payment_hash: &[u8; 32],
) -> anyhow::Result<Option<OffchainBoard>> {
    let query = "SELECT * FROM bark_offchain_board WHERE payment_hash = ?1";
    let mut rows = conn.query(query, params![payment_hash.to_vec()]).await?;
    Ok(rows
        .next()
        .await?
        .map(|row| row_to_offchain_board(&row))
        .transpose()?)
}

pub(crate) async fn store_exit_vtxo_entry(
    tx: &Transaction,
    exit: &ExitEntry,
) -> anyhow::Result<()> {
    let query = r"INSERT INTO bark_exit_states (vtxo_id, state, history) VALUES (?1, ?2, ?3) ON CONFLICT (vtxo_id) DO UPDATE SET state = EXCLUDED.state, history = EXCLUDED.history";
    let id = exit.vtxo_id.to_string();
    let state = serde_json::to_string(&exit.state)
        .map_err(|e| anyhow::format_err!("Exit VTXO {} state can't be serialized: {}", id, e))?;
    let history = serde_json::to_string(&exit.history)
        .map_err(|e| anyhow::format_err!("Exit VTXO {} history can't be serialized: {}", id, e))?;
    tx.execute(query, params![id, state, history]).await?;
    Ok(())
}

pub(crate) async fn remove_exit_vtxo_entry(tx: &Transaction, id: &VtxoId) -> anyhow::Result<()> {
    let query = "DELETE FROM bark_exit_states WHERE vtxo_id = ?1";
    tx.execute(query, params![id.to_string()]).await?;
    Ok(())
}

pub(crate) async fn get_exit_vtxo_entries(conn: &Connection) -> anyhow::Result<Vec<ExitEntry>> {
    let mut statement = conn
        .query("SELECT vtxo_id, state, history FROM bark_exit_states", ())
        .await?;
    let mut result = Vec::new();
    while let Some(row) = statement.next().await? {
        let vtxo_id = VtxoId::from_str(&row.get::<String>(0)?)?;
        let state = serde_json::from_str::<ExitState>(&row.get::<String>(1)?)?;
        let history = serde_json::from_str::<Vec<ExitState>>(&row.get::<String>(2)?)?;
        result.push(ExitEntry {
            vtxo_id,
            state,
            history,
        });
    }
    Ok(result)
}

pub(crate) async fn store_exit_child_tx(
    tx: &Transaction,
    exit_txid: Txid,
    child_tx: &bdk_wallet::bitcoin::Transaction,
    block: Option<BlockRef>,
) -> anyhow::Result<()> {
    let query = r"INSERT INTO bark_exit_child_transactions (exit_id, child_tx, block_hash, height) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (exit_id) DO UPDATE SET child_tx = EXCLUDED.child_tx, block_hash = EXCLUDED.block_hash, height = EXCLUDED.height";
    let exit_id = exit_txid.to_string();
    let child_transaction = consensus::serialize(child_tx);
    let (height, hash) = if let Some(block) = block {
        (
            Some(block.height as i64),
            Some(consensus::serialize(&block.hash)),
        )
    } else {
        (None, None)
    };
    tx.execute(
        query,
        params![
            exit_id,
            child_transaction,
            hash.map(|h: Vec<u8>| Value::Blob(h)).unwrap_or(Value::Null),
            height.map(|h| Value::Integer(h)).unwrap_or(Value::Null)
        ],
    )
    .await?;
    Ok(())
}

pub(crate) async fn get_exit_child_tx(
    conn: &Connection,
    exit_txid: Txid,
) -> anyhow::Result<Option<(bdk_wallet::bitcoin::Transaction, Option<BlockRef>)>> {
    let query =
        r"SELECT child_tx, block_hash, height FROM bark_exit_child_transactions where exit_id = ?1";
    let mut rows = conn.query(query, params![exit_txid.to_string()]).await?;

    if let Some(row) = rows.next().await? {
        let tx_bytes: Vec<u8> = row.get(0)?;
        let tx = consensus::deserialize(&tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize transaction: {}", e))?;

        let block = {
            let hash_bytes: Option<Vec<u8>> = row.get(1)?;
            let height: Option<i64> = row.get(2)?;
            match (hash_bytes, height) {
                (Some(bytes), Some(height)) => {
                    let hash = BlockHash::from_slice(&bytes)
                        .map_err(|e| anyhow::anyhow!("Failed to deserialize block hash: {}", e))?;
                    Some(BlockRef {
                        hash,
                        height: height as u32,
                    })
                }
                (None, None) => None,
                _ => panic!("Invalid data in database"),
            }
        };
        Ok(Some((tx, block)))
    } else {
        Ok(None)
    }
}
