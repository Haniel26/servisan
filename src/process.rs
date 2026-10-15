use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};

/// A spawned child process whose stdout/stderr are redirected to a log file.
///
/// This is the low-level primitive shared by all managed services. Job-object
/// based cleanup on Windows is layered on top in `platform::windows`.
pub struct ManagedChild {
    name: String,
    pid: u32,
    log_path: PathBuf,
    child: Child,
}

impl ManagedChild {
    /// Spawn `cmd`, redirecting its output to `log_path`.
    pub fn spawn(
        name: impl Into<String>,
        mut cmd: Command,
        log_path: impl Into<PathBuf>,
    ) -> Result<Self> {
        let name = name.into();
        let log_path = log_path.into();

        if let Some(dir) = log_path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let file = File::create(&log_path)
            .with_context(|| format!("creating log file {}", log_path.display()))?;
        let err = file
            .try_clone()
            .context("cloning log file handle for stderr")?;

        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(file))
            .stderr(Stdio::from(err));

        let child = cmd
            .spawn()
            .with_context(|| format!("failed to start '{name}' ({})", log_path.display()))?;
        let pid = child.id();

        // Ensure the child dies if Servisan exits or crashes.
        #[cfg(windows)]
        if let Err(e) = crate::platform::assign_to_global_job(pid) {
            eprintln!("warning: could not attach '{name}' to cleanup job: {e}");
        }

        Ok(Self {
            name,
            pid,
            log_path,
            child,
        })
    }

    /// Poll the child without blocking.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Terminate the child and reap it. Best-effort: never fails on an already
    /// dead process.
    pub fn stop(&mut self) -> Result<()> {
        if self.is_running() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        Ok(())
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Access the raw child, e.g. to attach a Windows job object.
    pub fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }
}
