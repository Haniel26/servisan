use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};

/// Install a Ctrl+C handler that flips the returned flag.
///
/// The caller's main loop polls the flag and shuts services down gracefully.
/// A second Ctrl+C would normally be ignored; here we simply keep the flag set,
/// and the job object guarantees cleanup even on a hard exit.
pub fn install_shutdown_flag() -> Result<Arc<AtomicBool>> {
    let flag = Arc::new(AtomicBool::new(false));
    let handler_flag = flag.clone();
    ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    })
    .context("installing Ctrl+C handler")?;
    Ok(flag)
}
