use anyhow::Context;

use libsql::Transaction;

use super::Migration;

pub struct Migration0006 {}

impl Migration for Migration0006 {
    fn name(&self) -> &str {
        "Update the exit system to be a state machine"
    }

    fn to_version(&self) -> i64 {
        6
    }

    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let summary = self.summary();
        Box::pin(async move {
            // We can't use JSONB with rusqlite, so we make do with strings
            let queries = [
                "DROP TABLE bark_exit;",
                "CREATE TABLE IF NOT EXISTS bark_exit_states (
    vtxo_id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    history TEXT NOT NULL
   );",
                "CREATE TABLE IF NOT EXISTS bark_exit_child_transactions (
    exit_id TEXT PRIMARY KEY,
    child_tx BLOB NOT NULL,
    block_hash BLOB,
    height INTEGER
   );",
            ];
            for query in queries {
                conn.execute(query, ())
                    .await
                    .with_context(|| format!("Failed to execute migration: {}", summary))?;
            }
            Ok(())
        })
    }
}
