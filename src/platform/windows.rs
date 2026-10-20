//! Windows-specific primitives.
//!
//! The key one is a **Job Object** with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`:
//! every process Servisan spawns is assigned to the job, and the OS terminates
//! the whole group if the handle closes — i.e. when Servisan exits or crashes.
//! This guarantees `mysqld.exe`/`php.exe` never leak into Task Manager.

use std::sync::OnceLock;

use anyhow::Result;
use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows::Win32::System::Threading::{
    CreateMutexW, OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

/// RAII wrapper around a Windows named mutex used for single-instance detection.
pub struct SingleInstance(HANDLE);

unsafe impl Send for SingleInstance {}
unsafe impl Sync for SingleInstance {}

impl SingleInstance {
    /// Try to acquire a named mutex.
    /// Returns `Ok(Some(guard))` if this process is the first/only instance.
    /// Returns `Ok(None)` if another instance is already running holding this mutex.
    pub fn acquire(name: &str) -> Result<Option<Self>> {
        unsafe {
            let hstring = HSTRING::from(name);
            let handle = CreateMutexW(None, true, &hstring)?;
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(handle);
                Ok(None)
            } else {
                Ok(Some(Self(handle)))
            }
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// RAII wrapper around a Windows job-object handle.
pub struct JobHandle(HANDLE);

// HANDLE is a raw pointer; the job object itself is safe to share across threads.
unsafe impl Send for JobHandle {}
unsafe impl Sync for JobHandle {}

impl JobHandle {
    /// Create a job object that kills all its members when the handle closes.
    pub fn new() -> Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(None, None)?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )?;

            Ok(Self(handle))
        }
    }

    /// Put an already-running process into the job.
    pub fn assign(&self, pid: u32) -> Result<()> {
        unsafe {
            let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid)?;
            let result = AssignProcessToJobObject(self.0, process);
            let _ = CloseHandle(process);
            result?;
        }
        Ok(())
    }
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

static GLOBAL_JOB: OnceLock<Option<JobHandle>> = OnceLock::new();

/// The process-wide job object, created on first use.
///
/// Returns `None` if the job could not be created; callers then degrade
/// gracefully rather than failing to start services.
pub fn global_job() -> Option<&'static JobHandle> {
    GLOBAL_JOB.get_or_init(|| JobHandle::new().ok()).as_ref()
}

/// Assign a freshly spawned process to the global job (best effort).
pub fn assign_to_global_job(pid: u32) -> Result<()> {
    match global_job() {
        Some(job) => job.assign(pid),
        None => Ok(()),
    }
}

/// Open a URL, document, or directory using Windows ShellExecuteW.
/// This runs independently of cmd.exe, does not inherit stdio handles,
/// and works safely even after FreeConsole() or in windowless processes.
pub fn shell_open(target: &str) -> Result<()> {
    use windows::core::{w, HSTRING, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    unsafe {
        let htarget = HSTRING::from(target);
        let ret = ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(htarget.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
        let code = ret.0 as isize;
        if code <= 32 {
            anyhow::bail!("ShellExecuteW failed for '{target}' with error code {code}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    #[test]
    fn creates_and_assigns_job() {
        let job = JobHandle::new().expect("create job object");
        let mut child = Command::new("cmd")
            .args(["/c", "ping", "-n", "30", "127.0.0.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn child");
        job.assign(child.id()).expect("assign to job");

        // Closing the job handle must terminate the assigned process.
        drop(job);

        let mut died = false;
        for _ in 0..50 {
            if let Ok(Some(_)) = child.try_wait() {
                died = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = child.kill();
        assert!(died, "child should be killed when job handle closes");
    }

    #[test]
    fn single_instance_blocks_duplicate() {
        let name = "Local\\Servisan_Test_Mutex_Unique_12345";
        let first = SingleInstance::acquire(name).expect("acquire first instance");
        assert!(first.is_some(), "first instance should be acquired");

        let second = SingleInstance::acquire(name).expect("acquire second instance");
        assert!(second.is_none(), "second instance should be blocked");

        drop(first);

        let third = SingleInstance::acquire(name).expect("acquire third instance after drop");
        assert!(
            third.is_some(),
            "third instance should succeed after first is dropped"
        );
    }
}
