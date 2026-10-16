pub mod app;
pub mod config;
pub mod ini;
pub mod paths;
pub mod phpmyadmin;
pub mod platform;
pub mod process;
pub mod services;
pub mod signal;
pub mod tray;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Serializes heavy integration tests (those that spawn real `mysqld`), which
/// cannot run concurrently.
#[cfg(test)]
pub(crate) static HEAVY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
