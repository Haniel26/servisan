use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{bail, ensure, Context, Result};

use crate::config::Config;
use crate::paths::Paths;
use crate::process::ManagedChild;

/// Manages the MySQL/MariaDB server lifecycle.
pub struct MysqlService {
    child: ManagedChild,
    port: u16,
}

impl MysqlService {
    /// True when the data directory already contains an initialized database.
    pub fn is_initialized(paths: &Paths) -> bool {
        let data = paths.data_dir();
        data.join("mysql").is_dir() || data.join("ibdata1").is_file()
    }

    /// Initialize a fresh, password-less data directory when needed.
    pub fn ensure_initialized(paths: &Paths, cfg: &Config) -> Result<()> {
        if Self::is_initialized(paths) {
            return Ok(());
        }
        Self::initialize(paths, cfg)
    }

    /// Run `mysqld --initialize-insecure` into the configured data directory.
    pub fn initialize(paths: &Paths, cfg: &Config) -> Result<()> {
        let mysqld = paths.mysqld_exe();
        ensure!(
            mysqld.is_file(),
            "MySQL server not found at {}\nRun scripts/install-binaries.ps1 first.",
            mysqld.display()
        );

        let data = paths.data_dir();
        if data.exists() {
            let empty = std::fs::read_dir(&data)
                .map(|mut it| it.next().is_none())
                .unwrap_or(false);
            ensure!(
                empty,
                "data directory {} is not empty but not a valid MySQL datadir; \
                 delete it or point config to a new location",
                data.display()
            );
        }
        std::fs::create_dir_all(&data).with_context(|| format!("creating {}", data.display()))?;

        crate::ini::write_all(paths, cfg)?;
        let ini = crate::ini::my_ini_path(paths);

        let output = Command::new(&mysqld)
            .arg(format!("--defaults-file={}", ini.display()))
            .arg("--initialize-insecure")
            .output()
            .context("running mysqld --initialize-insecure")?;

        if !output.status.success() {
            bail!(
                "mysqld --initialize-insecure failed ({}):\n{}{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        Ok(())
    }

    /// Start `mysqld`. The datadir must already be initialized.
    pub fn start(paths: &Paths, cfg: &Config) -> Result<Self> {
        ensure!(
            Self::is_initialized(paths),
            "MySQL data directory is not initialized; run `servisan init` first"
        );

        crate::ini::write_all(paths, cfg)?;
        let ini = crate::ini::my_ini_path(paths);
        let init_sql = crate::ini::init_sql_path(paths);

        let mut cmd = Command::new(paths.mysqld_exe());
        cmd.arg(format!("--defaults-file={}", ini.display()));
        cmd.arg(format!("--init-file={}", init_sql.display()));
        cmd.arg("--console");
        paths.apply_path_env(&mut cmd);

        let log = paths.logs_dir().join("mysql.log");
        let child = ManagedChild::spawn("mysqld", cmd, log)?;

        Ok(Self {
            child,
            port: cfg.mysql.port,
        })
    }

    pub fn pid(&self) -> u32 {
        self.child.pid()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_running(&mut self) -> bool {
        self.child.is_running()
    }

    pub fn log_path(&self) -> &std::path::Path {
        self.child.log_path()
    }

    /// Gracefully stop the server via `mysqladmin shutdown`, then force-kill if
    /// it does not exit in time. Avoids datadir corruption from a hard kill.
    pub fn stop(&mut self, paths: &Paths, cfg: &Config) -> Result<()> {
        if self.child.is_running() {
            let mysqladmin = paths.mysqladmin_exe();
            if mysqladmin.is_file() {
                let _ = Command::new(&mysqladmin)
                    .arg("--host=127.0.0.1")
                    .arg(format!("--port={}", cfg.mysql.port))
                    .arg("--user=root")
                    .arg("shutdown")
                    .status();
            }

            let deadline = Instant::now() + Duration::from_secs(15);
            while self.child.is_running() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(250));
            }
        }

        self.child.stop()
    }

    /// Wait until the server accepts TCP connections, or the timeout elapses.
    pub fn wait_ready(&mut self, timeout: Duration) -> bool {
        let addr = format!("127.0.0.1:{}", self.port);
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if !self.child.is_running() {
                return false;
            }
            if std::net::TcpStream::connect_timeout(
                &addr.parse().expect("valid socket addr"),
                Duration::from_millis(300),
            )
            .is_ok()
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free_port() -> u16 {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    }

    fn service_paths() -> Option<(Paths, Config)> {
        let paths = Paths::discover().ok()?;
        if !paths.mysqld_exe().is_file() {
            return None;
        }
        Some((paths, Config::default()))
    }

    #[test]
    fn detects_uninitialized_datadir() {
        let dir = std::env::temp_dir().join("servisan_mysql_uninit");
        let _ = std::fs::remove_dir_all(&dir);
        let paths = Paths::new(&dir);
        assert!(!MysqlService::is_initialized(&paths));
    }

    #[test]
    #[ignore = "slow: initializes a real MySQL datadir and starts the server"]
    fn initialize_start_and_stop() {
        let _guard = crate::HEAVY_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (base, mut cfg) = match service_paths() {
            Some(v) => v,
            None => {
                eprintln!("skipping: mysqld.exe not installed");
                return;
            }
        };

        let data = std::env::temp_dir().join("servisan_mysql_data");
        let _ = std::fs::remove_dir_all(&data);
        let paths = base.with_data_dir(&data);
        cfg.mysql.port = free_port();

        MysqlService::ensure_initialized(&paths, &cfg).unwrap();
        assert!(MysqlService::is_initialized(&paths));

        let mut svc = MysqlService::start(&paths, &cfg).unwrap();
        assert!(svc.wait_ready(Duration::from_secs(30)), "mysqld did not become ready");

        svc.stop(&paths, &cfg).unwrap();
        assert!(!svc.is_running(), "mysqld should have stopped");

        let _ = std::fs::remove_dir_all(&data);
    }
}
