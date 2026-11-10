//! Process-wide connection pool of PostgreSQL.

use std::sync::OnceLock;

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

use crate::infrastructure::core::settings::AppConfig;

/// Connection pool of one database type.
pub type Pool<T> = r2d2::Pool<ConnectionManager<T>>;

/// The PostgreSQL connection pool of the application.
pub type PostgresPool = Pool<PgConnection>;

static POOL: OnceLock<PostgresPool> = OnceLock::new();

/// Create the process-wide connection pool and return it.
///
/// Called once per process; a repeated call returns the existing pool. Connections are opened
/// lazily on first use, but the pool is created eagerly at startup, so a unreachable database
/// fails the application start.
pub fn init_database(config: &AppConfig) -> PostgresPool {
    POOL.get_or_init(|| {
        let manager = ConnectionManager::<PgConnection>::new(config.postgres_dsn.clone());
        Pool::builder()
            .build(manager)
            .expect("Failed to create pool")
    })
    .clone()
}

/// Return the process-wide connection pool.
///
/// # Panics
///
/// Panics if `init_database` has not been called in this process.
pub fn db_pool() -> PostgresPool {
    POOL.get()
        .expect("The database is not initialized: call init_database() first")
        .clone()
}
