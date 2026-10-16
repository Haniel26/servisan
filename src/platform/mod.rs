#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{assign_to_global_job, global_job, JobHandle, SingleInstance};

#[cfg(not(windows))]
pub struct SingleInstance;

#[cfg(not(windows))]
impl SingleInstance {
    pub fn acquire(_name: &str) -> anyhow::Result<Option<Self>> {
        Ok(Some(Self))
    }
}
