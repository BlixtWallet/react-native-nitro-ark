use anyhow::Context;

use libsql::Transaction;

use super::Migration;

pub struct Migration0007 {}

impl Migration for Migration0007 {
    fn name(&self) -> &str {
        "Rename vtxo_refresh_threshold to vtxo_refresh_expiry_threshold"
    }

    fn to_version(&self) -> i64 {
        7
    }

    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let summary = self.summary();
        Box::pin(async move {
            // We can't use JSONB with rusqlite, so we make do with strings
            let queries = [
   "ALTER TABLE bark_config RENAME COLUMN vtxo_refresh_threshold TO vtxo_refresh_expiry_threshold;",
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
