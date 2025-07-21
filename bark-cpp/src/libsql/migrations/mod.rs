mod m0001_initial_version;
mod m0002_config;
mod m0003_payment_history;
mod m0004_unregistered_board;
mod m0005_offchain_boards;
mod m0006_exit_rework;
mod m0007_vtxo_refresh_expiry_threshold;
mod m0008_fee_rate_implementation;

use anyhow::{bail, Context};
use libsql::{Connection, Transaction};
use logger::log::{debug, trace};

use m0001_initial_version::Migration0001;
use m0002_config::Migration0002;
use m0003_payment_history::Migration0003;
use m0004_unregistered_board::Migration0004;
use m0005_offchain_boards::Migration0005;
use m0006_exit_rework::Migration0006;
use m0007_vtxo_refresh_expiry_threshold::Migration0007;
use m0008_fee_rate_implementation::Migration0008;

pub struct MigrationContext {}

impl MigrationContext {
    /// Creates a new migration context
    pub fn new() -> Self {
        MigrationContext {}
    }

    /// Perform all initliazation scripts
    pub async fn do_all_migrations(&self, conn: &mut Connection) -> anyhow::Result<()> {
        let tx = conn
            .transaction()
            .await
            .context("Failed to start transcation")?;
        self.init_migrations(&tx).await?;
        tx.commit().await.context("Failed to commit transaction")?;

        // Run all migration scripts
        self.try_migration(conn, &Migration0001 {}).await?;
        self.try_migration(conn, &Migration0002 {}).await?;
        self.try_migration(conn, &Migration0003 {}).await?;
        self.try_migration(conn, &Migration0004 {}).await?;
        self.try_migration(conn, &Migration0005 {}).await?;
        self.try_migration(conn, &Migration0006 {}).await?;
        self.try_migration(conn, &Migration0007 {}).await?;
        self.try_migration(conn, &Migration0008 {}).await?;
        Ok(())
    }

    /// Initiliazes the migrations table in the database if needed
    ///
    /// This function returns the current schema sversion if succesful
    async fn init_migrations(&self, conn: &Connection) -> anyhow::Result<i64> {
        self.create_migrations_table_if_not_exists(conn).await?;
        match self.get_current_version(conn).await {
            Ok(version) => Ok(version),
            Err(_) => {
                // The database hasn't been initialized yet
                self.update_version(conn, 0).await?;
                Ok(0)
            }
        }
    }

    /// Attempts to perform a migration if needed
    async fn try_migration<'a>(
        &self,
        conn: &mut Connection,
        migration: &impl Migration,
    ) -> anyhow::Result<()> {
        // Start the transaction
        let tx = conn
            .transaction()
            .await
            .context("Failed to init transaction")?;

        let current_version = self.get_current_version(&tx).await?;
        let from_version = migration.from_version();

        if current_version == from_version {
            debug!("Performing migration {}", migration.summary());
            migration.do_migration(&tx).await?;
            self.update_version(&tx, migration.to_version()).await?;
        } else if current_version < from_version {
            bail!(
                "Failed to perform migration. Database is at {} for migration {}",
                current_version,
                migration.summary()
            );
        } else {
            trace!(
                "Skipping migration {}. Nothing to be done",
                migration.summary()
            );
        };
        tx.commit().await.context("Failed to commit transaction")?;
        Ok(())
    }

    /// Retrieves the current schema version
    async fn get_current_version(&self, conn: &Connection) -> anyhow::Result<i64> {
        const ERR_MSG: &'static str = "Failed to get_current_version from database";

        let query = "SELECT value FROM migrations ORDER BY value DESC LIMIT 1";
        let mut rows = conn.query(query, ()).await.context(ERR_MSG)?;

        let row = rows
            .next()
            .await
            .context(ERR_MSG)?
            .context("the current schema version is not defined in the databases")?;
        Ok(row.get(0).context(ERR_MSG)?)
    }

    /// Update schema version
    async fn update_version(&self, conn: &Connection, new_version: i64) -> anyhow::Result<i64> {
        const ERR_MSG: &'static str = "Failed to update_version for database";

        let query = "INSERT INTO migrations (value) VALUES (?1)";
        conn.execute(query, [new_version]).await.context(ERR_MSG)?;

        Ok(new_version)
    }

    /// Creates the migrations table if it doesn't exist yet
    async fn create_migrations_table_if_not_exists(&self, conn: &Connection) -> anyhow::Result<()> {
        let query = "CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
    value INTEGER NOT NULL
   )";

        conn.execute(query, ())
            .await
            .context("Failed to create migration table")?;

        Ok(())
    }
}

trait Migration {
    fn name(&self) -> &str;
    fn to_version(&self) -> i64;

    fn from_version(&self) -> i64 {
        self.to_version() - 1
    }

    /// Performs the migration script on the provided connection
    fn do_migration<'a>(
        &self,
        conn: &'a Transaction,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn summary(&self) -> String {
        format!(
            "{}->{}:'{}'",
            self.from_version(),
            self.to_version(),
            self.name()
        )
    }
}
