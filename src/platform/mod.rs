#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{assign_to_global_job, global_job, shell_open, JobHandle, SingleInstance};

#[cfg(not(windows))]
pub struct SingleInstance;

#[cfg(not(windows))]
impl SingleInstance {
    pub fn acquire(_name: &str) -> anyhow::Result<Option<Self>> {
        Ok(Some(Self))
    }
}

#[cfg(not(windows))]
pub fn shell_open(_target: &str) -> anyhow::Result<()> {
    Ok(())
}
