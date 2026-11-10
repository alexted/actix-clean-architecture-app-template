//! Application configuration read from the environment (and from `.env` when present).

use std::sync::OnceLock;

use dotenv::dotenv;

/// Name of the environment variable holding the PostgreSQL connection string.
pub const POSTGRESQL_DSN_ENV: &str = "DATABASE_URL";

/// Runtime environments known to the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Testing,
    Test,
    Dev,
    Stage,
    Prod,
}

impl Environment {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "LOCAL" => Some(Self::Local),
            "TESTING" => Some(Self::Testing),
            "TEST" => Some(Self::Test),
            "DEV" => Some(Self::Dev),
            "STAGE" => Some(Self::Stage),
            "PROD" => Some(Self::Prod),
            _ => None,
        }
    }
}

/// Application settings; every field can be set by the environment variable of the same name.
///
/// `POSTGRESQL_DSN` is read from the `DATABASE_URL` environment variable and is required:
/// the application does not start without it.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The runtime environment the application runs in.
    pub environment: Environment,
    /// The address the server binds to.
    pub server_host: String,
    /// The port the server listens on.
    pub server_port: u16,
    /// Logging severity of the application code and the request log (when `RUST_LOG` is unset).
    pub log_level: String,
    /// The connection string of PostgreSQL.
    pub postgres_dsn: String,
}

impl AppConfig {
    fn from_env() -> Self {
        dotenv().ok();

        let environment = std::env::var("ENVIRONMENT")
            .ok()
            .and_then(|value| Environment::parse(&value))
            .unwrap_or(Environment::Local);
        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
        let postgres_dsn = std::env::var(POSTGRESQL_DSN_ENV)
            .unwrap_or_else(|_| panic!("{POSTGRESQL_DSN_ENV} must be set"));

        AppConfig {
            environment,
            server_host,
            server_port,
            log_level,
            postgres_dsn,
        }
    }
}

/// Return the settings, read once per process.
pub fn get_config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(AppConfig::from_env)
}
