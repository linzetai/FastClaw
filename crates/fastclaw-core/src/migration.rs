//! Lightweight numbered-migration runner for SQLite databases.
//!
//! Each migration is a `(version, name, sql)` tuple. The runner ensures that each
//! migration runs at most once and tracks applied versions in a `_migrations` table.

use sqlx::{Pool, Sqlite};

/// A single numbered migration.
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Runs migrations sequentially on a SQLite pool.
pub struct MigrationRunner {
    migrations: Vec<Migration>,
    table_name: String,
}

impl MigrationRunner {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            migrations: Vec::new(),
            table_name: table_name.into(),
        }
    }

    pub fn add(mut self, version: u32, name: &'static str, sql: &'static str) -> Self {
        self.migrations.push(Migration { version, name, sql });
        self
    }

    /// Run all pending migrations against the pool.
    pub async fn run(&self, pool: &Pool<Sqlite>) -> anyhow::Result<u32> {
        let tbl = &self.table_name;

        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {tbl} (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ))
        .execute(pool)
        .await?;

        let applied: Vec<i32> =
            sqlx::query_scalar(&format!("SELECT version FROM {tbl} ORDER BY version"))
                .fetch_all(pool)
                .await?;

        let applied_set: std::collections::HashSet<u32> =
            applied.into_iter().map(|v| v as u32).collect();

        let mut count = 0u32;
        for m in &self.migrations {
            if applied_set.contains(&m.version) {
                continue;
            }
            tracing::info!(version = m.version, name = m.name, table = %tbl, "running migration");
            sqlx::query(m.sql).execute(pool).await?;
            sqlx::query(&format!(
                "INSERT INTO {tbl} (version, name) VALUES (?1, ?2)"
            ))
            .bind(m.version as i32)
            .bind(m.name)
            .execute(pool)
            .await?;
            count += 1;
        }

        if count > 0 {
            tracing::info!(applied = count, "migrations complete");
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mem_pool() -> Pool<Sqlite> {
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn empty_migrations_creates_table() {
        let pool = mem_pool().await;
        let runner = MigrationRunner::new("_migrations");
        let count = runner.run(&pool).await.unwrap();
        assert_eq!(count, 0);
        let rows: Vec<(i32,)> =
            sqlx::query_as("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='_migrations'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(rows[0].0, 1);
    }

    #[tokio::test]
    async fn applies_pending_migrations() {
        let pool = mem_pool().await;
        let runner = MigrationRunner::new("_migrations")
            .add(1, "create_a", "CREATE TABLE a (id INTEGER PRIMARY KEY)")
            .add(2, "create_b", "CREATE TABLE b (id INTEGER PRIMARY KEY)");
        let count = runner.run(&pool).await.unwrap();
        assert_eq!(count, 2);

        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name IN ('a','b') ORDER BY name")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(tables.len(), 2);
    }

    #[tokio::test]
    async fn skips_already_applied() {
        let pool = mem_pool().await;
        let r1 = MigrationRunner::new("_migrations")
            .add(1, "create_a", "CREATE TABLE a (id INTEGER PRIMARY KEY)")
            .add(2, "create_b", "CREATE TABLE b (id INTEGER PRIMARY KEY)");
        assert_eq!(r1.run(&pool).await.unwrap(), 2);

        let r2 = MigrationRunner::new("_migrations")
            .add(1, "create_a", "CREATE TABLE a (id INTEGER PRIMARY KEY)")
            .add(2, "create_b", "CREATE TABLE b (id INTEGER PRIMARY KEY)")
            .add(3, "create_c", "CREATE TABLE c (id INTEGER PRIMARY KEY)");
        assert_eq!(r2.run(&pool).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn idempotent_rerun() {
        let pool = mem_pool().await;
        let runner = MigrationRunner::new("_migrations")
            .add(1, "create_x", "CREATE TABLE x (id INTEGER PRIMARY KEY)");
        assert_eq!(runner.run(&pool).await.unwrap(), 1);
        assert_eq!(runner.run(&pool).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn custom_table_name() {
        let pool = mem_pool().await;
        let runner = MigrationRunner::new("_my_migrations")
            .add(1, "init", "CREATE TABLE t1 (v TEXT)");
        assert_eq!(runner.run(&pool).await.unwrap(), 1);

        let rows: Vec<(i32,)> =
            sqlx::query_as("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='_my_migrations'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(rows[0].0, 1);
    }
}
