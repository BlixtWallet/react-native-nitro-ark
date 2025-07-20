use anyhow::Context;

use libsql::Transaction;

use super::Migration;

pub struct Migration0005 {}

impl Migration for Migration0005 {
    fn name(&self) -> &str {
        "Add table to support offchain boards with HTLCs"
    }

    fn to_version(&self) -> i64 {
        5
    }

    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let summary = self.summary();
        Box::pin(async move {
            // Rename Ready to Spendable
            let query = "CREATE TABLE bark_offchain_board (
   payment_hash BLOB NOT NULL PRIMARY KEY,
   preimage BLOB NOT NULL UNIQUE,
   serialised_payment BLOB,
   created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
  )";

            conn.execute(query, ())
                .await
                .with_context(|| format!("Failed to execute migration: {}", summary))?;
            Ok(())
        })
    }
}
