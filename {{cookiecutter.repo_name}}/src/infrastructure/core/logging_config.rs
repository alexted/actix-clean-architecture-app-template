//! Logging configuration for the application process.
//!
//! Records below `LOG_LEVEL` are dropped; `RUST_LOG` takes precedence when set. Calling this
//! more than once (for example once per worker or per test) is safe.

use super::settings::AppConfig;

/// Configure logging for the application process.
pub fn init_logging(config: &AppConfig) {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(config.log_level.to_lowercase()),
    )
    .try_init()
    .ok();
}
