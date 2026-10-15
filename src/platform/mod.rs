#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{assign_to_global_job, global_job, JobHandle};
