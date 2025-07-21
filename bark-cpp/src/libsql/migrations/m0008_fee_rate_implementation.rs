use anyhow::Context;

use libsql::Transaction;

use super::Migration;

pub struct Migration0008 {}

impl Migration for Migration0008 {
    fn name(&self) -> &str {
        "Add fallback_fee column to bark_config"
    }

    fn to_version(&self) -> i64 {
        8
    }

    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let summary = self.summary();
        Box::pin(async move {
            let queries = ["ALTER TABLE bark_config ADD COLUMN fallback_fee_kwu INTEGER;"];
            for query in queries {
                conn.execute(query, ())
                    .await
                    .with_context(|| format!("Failed to execute migration: {}", summary))?;
            }
            Ok(())
        })
    }
}
