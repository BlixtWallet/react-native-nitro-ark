use anyhow::Context;

use libsql::Transaction;

use super::Migration;

pub struct Migration0004 {}

impl Migration for Migration0004 {
    fn name(&self) -> &str {
        "Updating the VtxoState"
    }

    fn to_version(&self) -> i64 {
        4
    }

    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let summary = self.summary();
        Box::pin(async move {
            // Rename Ready to Spendable
            let query = "UPDATE bark_vtxo_state SET state = 'Spendable' WHERE state = 'Ready'";

            conn.execute(query, ())
                .await
                .with_context(|| format!("Failed to execute migration: {}", summary))?;
            Ok(())
        })
    }
}
