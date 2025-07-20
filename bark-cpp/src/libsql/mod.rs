use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use bark::ark::bitcoin::secp256k1::PublicKey;
use bark::ark::bitcoin::Txid;
mod migrations;
use bark::ark::{Vtxo, VtxoId};
use bark::exit::vtxo::ExitEntry;
use bark::movement::{Movement, MovementArgs};
use bark::persist::{BarkPersister, OffchainBoard, OffchainPayment};
use bark::vtxo_state::{VtxoState, VtxoStateKind, WalletVtxo};
use bdk_wallet::bitcoin::{Amount, Transaction};
use bdk_wallet::ChangeSet;
use bitcoin_ext::{BlockHeight, BlockRef};

use bark::Config;
use bark::KeychainKind;
use bark::Pagination;
use bark::WalletProperties;
use libsql::{Builder, Connection, Database};
use logger::log::debug;

use crate::TOKIO_RUNTIME;

mod convert;
mod query;

#[derive(Clone)]
pub struct LibsqlClient {
    db: Arc<Database>,
}

impl LibsqlClient {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        debug!("Opening database at {}", path.display());
        let db_path = path.to_str().context("Invalid database path")?.to_owned();

        let db = std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async {
                let url = "libsql://nitro-ark-niteshbalusu11.aws-us-east-2.turso.io".to_string();
                let token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicnciLCJleHAiOjE3NTM1NzkxMTUsImlhdCI6MTc1Mjk3NDMxNSwiaWQiOiIzYmE5NGIyZS00NjIxLTQzMjEtOTI2Yi0wNzM0MWI5MGVlYTkiLCJyaWQiOiI1OWE0MjI4Ny03NTBkLTRkODMtYTQ2Mi01MGEyOTg2OWJjZDUifQ.6Z7sFWUWg-PXyFe0YBIKlMUpMl2QhFWw29tnPMsmTvSa5-6Jk71jV0lmN_kuHTV0Qq-rfIfAumrdRNF6jZT8AA".to_string();
                let db: anyhow::Result<Database> = async {
                    let db = Builder::new_synced_database(db_path, url, token)
                        .build()
                        .await?;
                    let migrations = migrations::MigrationContext::new();
                    migrations
                        .do_all_migrations(&mut db.connect()?)
                        .await?;
                    Ok(db)
                }
                .await;
                db
            })
        })
        .join()
        .unwrap()
        .context("Failed to build database")?;

        // TODO: Run migrations

        Ok(Self { db: Arc::new(db) })
    }

    async fn connect(&self) -> anyhow::Result<Connection> {
        self.db.connect().context("Failed to connect to database")
    }
}

impl BarkPersister for LibsqlClient {
    fn init_wallet(&self, config: &Config, properties: &WalletProperties) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let config = config.clone();
        let properties = properties.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::set_properties(&tx, &properties).await?;
                query::set_config(&tx, &config).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn initialize_bdk_wallet(&self) -> anyhow::Result<ChangeSet> {
        // TODO: Implement bdk_wallet persistence for libsql
        // This requires a custom implementation of bdk_wallet::WalletPersister
        // as there is no official support for libsql yet.
        // For now, we return an empty changeset.
        Ok(ChangeSet::default())
    }

    fn store_bdk_wallet_changeset(&self, _changeset: &ChangeSet) -> anyhow::Result<()> {
        // TODO: Implement bdk_wallet persistence for libsql
        Ok(())
    }

    fn write_config(&self, config: &Config) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::set_config(&tx, &config).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn read_properties(&self) -> anyhow::Result<Option<WalletProperties>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::fetch_properties(&conn).await
            })
        })
        .join()
        .unwrap()
    }

    fn read_config(&self) -> anyhow::Result<Option<Config>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::fetch_config(&conn).await
            })
        })
        .join()
        .unwrap()
    }

    fn check_recipient_exists(&self, recipient: &str) -> anyhow::Result<bool> {
        let self_clone = self.clone();
        let recipient = recipient.to_string();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::check_recipient_exists(&conn, &recipient).await
            })
        })
        .join()
        .unwrap()
    }

    fn get_paginated_movements(&self, pagination: Pagination) -> anyhow::Result<Vec<Movement>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_paginated_movements(&conn, pagination).await
            })
        })
        .join()
        .unwrap()
    }

    fn register_movement(&self, movement: MovementArgs) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let spends: Vec<Vtxo> = movement.spends.iter().map(|v| (*v).clone()).collect();
        let receives: Vec<(Vtxo, VtxoState)> = movement
            .receives
            .iter()
            .map(|(v, s)| ((*v).clone(), s.clone()))
            .collect();
        let recipients: Vec<(String, Amount)> = movement
            .recipients
            .iter()
            .map(|(r, a)| (r.to_string(), *a))
            .collect();
        let fees = movement.fees;

        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;

                let movement_id = query::create_movement(&tx, fees).await?;

                for v in &spends {
                    query::update_vtxo_state_checked(
                        &tx,
                        v.id(),
                        VtxoState::Spent,
                        &[
                            VtxoStateKind::Spendable,
                            VtxoStateKind::PendingLightningSend,
                        ],
                    )
                    .await?;
                    query::link_spent_vtxo_to_movement(&tx, v.id(), movement_id).await?;
                }

                for (v, s) in &receives {
                    query::store_vtxo_with_initial_state(&tx, v, movement_id, s).await?;
                }

                for (recipient, amount) in &recipients {
                    query::create_recipient(&tx, movement_id, recipient, *amount).await?;
                }
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn get_wallet_vtxo(&self, id: VtxoId) -> anyhow::Result<Option<WalletVtxo>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_wallet_vtxo_by_id(&conn, id).await
            })
        })
        .join()
        .unwrap()
    }

    fn get_vtxos_by_state(&self, state: &[VtxoStateKind]) -> anyhow::Result<Vec<WalletVtxo>> {
        let self_clone = self.clone();
        let state = state.to_vec();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_vtxos_by_state(&conn, &state).await
            })
        })
        .join()
        .unwrap()
    }

    fn remove_vtxo(&self, id: VtxoId) -> anyhow::Result<Option<Vtxo>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                let result = query::delete_vtxo(&tx, id).await;
                tx.commit().await?;
                self_clone.db.sync().await?;
                result
            })
        })
        .join()
        .unwrap()
    }

    fn has_spent_vtxo(&self, id: VtxoId) -> anyhow::Result<bool> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let state: Option<VtxoState> = query::get_vtxo_state(&conn, id).await?;
                let result = state.map(|s| s == VtxoState::Spent).unwrap_or(false);
                Ok(result)
            })
        })
        .join()
        .unwrap()
    }

    fn store_vtxo_key(
        &self,
        keychain: KeychainKind,
        index: u32,
        public_key: PublicKey,
    ) -> anyhow::Result<()> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::store_vtxo_key(&tx, keychain, index, public_key).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn get_last_vtxo_key_index(&self, keychain: KeychainKind) -> anyhow::Result<Option<u32>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_last_vtxo_key_index(&conn, keychain).await
            })
        })
        .join()
        .unwrap()
    }

    fn get_vtxo_key(&self, vtxo: &Vtxo) -> anyhow::Result<(KeychainKind, u32)> {
        let self_clone = self.clone();
        let vtxo = vtxo.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_vtxo_key(&conn, &vtxo)
                    .await?
                    .context("vtxo not found in the db")
            })
        })
        .join()
        .unwrap()
    }

    fn check_vtxo_key_exists(&self, public_key: &PublicKey) -> anyhow::Result<bool> {
        let self_clone = self.clone();
        let public_key = *public_key;
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                let result = query::check_vtxo_key_exists(&tx, &public_key).await;
                tx.commit().await?;
                self_clone.db.sync().await?;
                result
            })
        })
        .join()
        .unwrap()
    }

    fn store_offchain_board(
        &self,
        payment_hash: &[u8; 32],
        preimage: &[u8; 32],
        payment: OffchainPayment,
    ) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let payment_hash = *payment_hash;
        let preimage = *preimage;
        let payment = payment.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::store_offchain_board(&tx, &payment_hash, &preimage, payment).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn fetch_offchain_board_by_payment_hash(
        &self,
        payment_hash: &[u8; 32],
    ) -> anyhow::Result<Option<OffchainBoard>> {
        let self_clone = self.clone();
        let payment_hash = *payment_hash;
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::fetch_offchain_board_by_payment_hash(&conn, &payment_hash).await
            })
        })
        .join()
        .unwrap()
    }

    fn store_exit_vtxo_entry(&self, exit: &ExitEntry) -> anyhow::Result<()> {
        let self_clone = self.clone();

        let exit_data = ExitEntry {
            history: exit.history.clone(),
            vtxo_id: exit.vtxo_id.clone(),
            state: exit.state.clone(),
        };
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::store_exit_vtxo_entry(&tx, &exit_data).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn remove_exit_vtxo_entry(&self, id: &VtxoId) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let id = *id;
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::remove_exit_vtxo_entry(&tx, &id).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn get_exit_vtxo_entries(&self) -> anyhow::Result<Vec<ExitEntry>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_exit_vtxo_entries(&conn).await
            })
        })
        .join()
        .unwrap()
    }

    fn store_exit_child_tx(
        &self,
        exit_txid: Txid,
        child_tx: &bdk_wallet::bitcoin::Transaction,
        block: std::option::Option<bitcoin_ext::BlockRef>,
    ) -> anyhow::Result<()> {
        let self_clone = self.clone();
        let child_tx = child_tx.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::store_exit_child_tx(&tx, exit_txid, &child_tx, block).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn get_exit_child_tx(
        &self,
        exit_txid: Txid,
    ) -> anyhow::Result<Option<(Transaction, Option<BlockRef>)>> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_exit_child_tx(&conn, exit_txid).await
            })
        })
        .join()
        .unwrap()
    }

    fn get_last_ark_sync_height(&self) -> anyhow::Result<BlockHeight> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                query::get_last_ark_sync_height(&conn).await
            })
        })
        .join()
        .unwrap()
    }

    fn store_last_ark_sync_height(&self, height: BlockHeight) -> anyhow::Result<()> {
        let self_clone = self.clone();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                query::store_last_ark_sync_height(&tx, height).await?;
                tx.commit().await?;
                self_clone.db.sync().await?;
                Ok(())
            })
        })
        .join()
        .unwrap()
    }

    fn update_vtxo_state_checked(
        &self,
        vtxo_id: VtxoId,
        new_state: VtxoState,
        allowed_old_states: &[VtxoStateKind],
    ) -> anyhow::Result<WalletVtxo> {
        let self_clone = self.clone();
        let allowed_old_states = allowed_old_states.to_vec();
        std::thread::spawn(move || {
            TOKIO_RUNTIME.block_on(async move {
                let conn = self_clone.connect().await?;
                let tx = conn.transaction().await?;
                let result =
                    query::update_vtxo_state_checked(&tx, vtxo_id, new_state, &allowed_old_states)
                        .await;
                tx.commit().await?;
                self_clone.db.sync().await?;
                result
            })
        })
        .join()
        .unwrap()
    }

    fn get_all_spendable_vtxos(&self) -> anyhow::Result<Vec<Vtxo>> {
        Ok(self
            .get_vtxos_by_state(&[VtxoStateKind::Spendable])?
            .into_iter()
            .map(|vtxo| vtxo.vtxo)
            .collect())
    }
}
