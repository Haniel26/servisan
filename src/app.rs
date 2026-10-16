use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::config::{Config, EngineKind};
use crate::paths::Paths;
use crate::services::mysql::MysqlService;
use crate::services::php::{BuiltinServer, HttpEngine, PhpService};

/// Which service(s) an action applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    Php,
    Mysql,
}

/// Top-level application: configuration plus resolved paths.
pub struct App {
    pub paths: Paths,
    pub config: Config,
}

/// Handles to the services started by this process.
#[derive(Default)]
pub struct Running {
    php: Option<PhpService>,
    mysql: Option<MysqlService>,
}

impl Running {
    /// True while at least one managed service is still alive.
    pub fn any_alive(&mut self) -> bool {
        let php = self.php.as_mut().map(PhpService::is_running).unwrap_or(false);
        let mysql = self
            .mysql
            .as_mut()
            .map(MysqlService::is_running)
            .unwrap_or(false);
        php || mysql
    }
}

impl App {
    /// Discover paths and load configuration.
    pub fn load() -> Result<Self> {
        let paths = Paths::discover()?;
        let config = Config::load_or_default(&paths.config_file())?;
        Ok(Self { paths, config })
    }

    /// Create the standard working directories if missing.
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            self.paths.www_dir(),
            self.paths.data_dir(),
            self.paths.logs_dir(),
            self.paths.config_dir(),
        ] {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("creating {}", dir.display()))?;
        }
        Ok(())
    }

    /// First-run setup: write config (if absent), generated ini files, and an
    /// initialized MySQL data directory.
    pub fn init(&self) -> Result<()> {
        self.ensure_dirs()?;

        if !self.paths.config_file().is_file() {
            self.config.save(&self.paths.config_file())?;
            println!("wrote {}", self.paths.config_file().display());
        }

        crate::ini::write_all(&self.paths, &self.config)?;
        MysqlService::ensure_initialized(&self.paths, &self.config)?;
        println!("initialized MySQL data directory at {}", self.paths.data_dir().display());
        println!("done.");
        Ok(())
    }

    /// Start the requested services and return handles to them.
    pub fn start(&self, only: Option<ServiceKind>) -> Result<Running> {
        self.ensure_dirs()?;
        crate::ini::write_all(&self.paths, &self.config)?;
        if self.config.phpmyadmin.enabled {
            if let Err(e) = self.ensure_phpmyadmin_configured() {
                eprintln!("warning: phpMyAdmin configuration failed: {e}");
            }
        }

        let want_php = matches!(only, None | Some(ServiceKind::Php));
        let want_mysql = matches!(only, None | Some(ServiceKind::Mysql));

        if want_mysql && !MysqlService::is_initialized(&self.paths) {
            println!("initializing MySQL data directory (first run) ...");
            MysqlService::ensure_initialized(&self.paths, &self.config)?;
        }

        let host = self.config.general.host.as_str();
        if want_php {
            check_port_free(host, self.config.php.port, "PHP")?;
        }
        if want_mysql {
            check_port_free(host, self.config.mysql.port, "MySQL")?;
        }

        let mut running = Running::default();

        if want_mysql {
            let mut mysql = MysqlService::start(&self.paths, &self.config)?;
            if !mysql.wait_ready(Duration::from_secs(30)) {
                let _ = mysql.stop(&self.paths, &self.config);
                bail!("MySQL did not become ready (see {})", mysql.log_path().display());
            }
            println!(
                "MySQL   pid {} -> mysql://127.0.0.1:{}",
                mysql.pid(),
                self.config.mysql.port
            );
            running.mysql = Some(mysql);
        }

        if want_php {
            let engine = self.http_engine()?;
            let php = PhpService::start(&self.paths, &self.config, engine)?;
            println!(
                "PHP     pid {} ({}) -> http://{}:{}",
                php.pid(),
                php.engine_name(),
                self.config.general.host,
                self.config.php.port
            );
            running.php = Some(php);
        }

        if want_php && self.config.general.auto_open_browser {
            let raw_host = self.config.general.host.as_str();
            let browser_host = if raw_host == "0.0.0.0" {
                "127.0.0.1"
            } else {
                raw_host
            };
            let url = format!("http://{}:{}/", browser_host, self.config.php.port);

            // Keep the worker ready before launching the browser.
            std::thread::sleep(Duration::from_millis(500));
            let ready_deadline = std::time::Instant::now() + Duration::from_secs(3);
            while std::time::Instant::now() < ready_deadline {
                if port_open(browser_host, self.config.php.port) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if let Err(e) = crate::phpmyadmin::open_url(&url) {
                eprintln!("warning: could not open browser: {e}");
            }
        }

        Ok(running)
    }

    /// Gracefully stop the services started by this process.
    pub fn stop_running(&self, running: &mut Running) -> Result<()> {
        if let Some(mysql) = running.mysql.as_mut() {
            println!("stopping MySQL ...");
            mysql.stop(&self.paths, &self.config)?;
        }
        if let Some(php) = running.php.as_mut() {
            println!("stopping PHP ...");
            php.stop()?;
        }
        running.mysql = None;
        running.php = None;
        Ok(())
    }

    /// Stop services started by *another* Servisan process, identified by
    /// executable location within this installation's `bin/`.
    pub fn stop_external(&self) -> Result<()> {
        // Try a graceful MySQL shutdown first.
        let mysqladmin = self.paths.mysqladmin_exe();
        if mysqladmin.is_file() && port_open(&self.config.general.host, self.config.mysql.port) {
            let mut cmd = std::process::Command::new(&mysqladmin);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }
            cmd.arg("--host=127.0.0.1")
                .arg(format!("--port={}", self.config.mysql.port))
                .arg("--user=root")
                .arg("shutdown");
            let _ = cmd.status();
        }

        let killed = kill_processes_under(&self.paths.bin_dir())?;
        if killed == 0 {
            println!("no running Servisan services found.");
        } else {
            println!("stopped {killed} Servisan process(es).");
        }
        Ok(())
    }

    /// Print current status of PHP and MySQL.
    pub fn status(&self) -> Result<()> {
        let host = self.config.general.host.as_str();
        let php_up = port_open(host, self.config.php.port);
        let mysql_up = port_open(host, self.config.mysql.port);

        println!("Servisan {}", crate::VERSION);
        println!("  root      : {}", self.paths.root().display());
        println!(
            "  PHP       : {} http://{}:{}  ({})",
            updown(php_up),
            host,
            self.config.php.port,
            engine_label(self.config.php.engine),
        );
        println!(
            "  MySQL     : {} mysql://{}:{}",
            updown(mysql_up),
            host,
            self.config.mysql.port
        );
        println!(
            "  phpMyAdmin: {} {}",
            if self.phpmyadmin_installed() { "installed" } else { "not installed" },
            self.phpmyadmin_url().as_deref().unwrap_or(""),
        );
        println!("  MySQL data: {}",
            if MysqlService::is_initialized(&self.paths) {
                "initialized"
            } else {
                "not initialized (run `servisan init`)"
            }
        );
        Ok(())
    }

    /// `PATH` with Servisan's PHP and MySQL directories prepended.
    pub fn runtime_path(&self) -> std::ffi::OsString {
        let current = std::env::var_os("PATH").unwrap_or_default();
        self.paths.prepend_path(current)
    }

    /// Open an interactive shell whose `PATH` includes `php`, `mysql` and
    /// `mysqladmin`, started in the Servisan root.
    pub fn shell(&self, shell: Option<&str>) -> Result<()> {
        let program = match shell {
            Some(s) => s.to_string(),
            None => std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string()),
        };

        for dir in self.paths.path_entries() {
            println!("PATH += {}", dir.display());
        }

        let status = std::process::Command::new(&program)
            .env("PATH", self.runtime_path())
            .env("SERVISAN_HOME", self.paths.root())
            .current_dir(self.paths.root())
            .status()
            .with_context(|| format!("launching shell '{program}'"))?;

        if !status.success() {
            println!("shell exited with {status}");
        }
        Ok(())
    }

    pub fn phpmyadmin_dir(&self) -> PathBuf {
        self.paths.phpmyadmin_dir()
    }

    pub fn phpmyadmin_installed(&self) -> bool {
        crate::phpmyadmin::is_installed(&self.paths)
    }

    pub fn phpmyadmin_url(&self) -> Option<String> {
        if self.phpmyadmin_installed() {
            Some(crate::phpmyadmin::url(&self.config))
        } else {
            None
        }
    }

    pub fn ensure_phpmyadmin_configured(&self) -> Result<()> {
        if !self.phpmyadmin_installed() {
            return Ok(());
        }
        crate::phpmyadmin::configure(&self.paths, &self.config)
    }

    pub fn install_phpmyadmin(&self, source: Option<PathBuf>, force: bool) -> Result<()> {
        crate::phpmyadmin::install(&self.paths, &self.config, source, force)?;
        if self.config.phpmyadmin.enabled {
            self.ensure_phpmyadmin_configured()?;
        }
        println!("phpMyAdmin installed at {}", self.phpmyadmin_dir().display());
        if let Some(url) = self.phpmyadmin_url() {
            println!("phpMyAdmin URL: {url}");
        }
        println!("done.");
        Ok(())
    }

    pub fn open_phpmyadmin(&self) -> Result<()> {
        if !self.phpmyadmin_installed() {
            bail!(
                "phpMyAdmin is not installed in {}.\n\
                 Run 'servisan phpmyadmin install' first.",
                self.phpmyadmin_dir().display()
            );
        }
        crate::phpmyadmin::open_url(&crate::phpmyadmin::url(&self.config))
    }

    fn http_engine(&self) -> Result<Box<dyn HttpEngine>> {
        match self.config.php.engine {
            EngineKind::Builtin => Ok(Box::new(BuiltinServer)),
            EngineKind::Nginx => bail!("php.engine = \"nginx\" is not implemented yet"),
            EngineKind::Apache => bail!("php.engine = \"apache\" is not implemented yet"),
        }
    }
}

fn updown(up: bool) -> &'static str {
    if up {
        "running"
    } else {
        "stopped"
    }
}

fn engine_label(kind: EngineKind) -> &'static str {
    match kind {
        EngineKind::Builtin => "builtin",
        EngineKind::Nginx => "nginx",
        EngineKind::Apache => "apache",
    }
}

/// Fail with a clear message if `port` cannot be bound on `host`.
fn check_port_free(host: &str, port: u16, label: &str) -> Result<()> {
    match TcpListener::bind((host, port)) {
        Ok(_) => Ok(()),
        Err(e) => bail!(
            "{label} port {host}:{port} is already in use ({e}).\
             \nStop the other service (e.g. Laragon) or change the port in config/app.toml."
        ),
    }
}

/// True if something is listening on `host:port`.
pub fn port_open(host: &str, port: u16) -> bool {
    let addr = match format!("{host}:{port}").parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

/// Terminate every process whose executable lives under `dir`.
fn kill_processes_under(dir: &Path) -> Result<usize> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut killed = 0;
    for process in sys.processes().values() {
        if let Some(exe) = process.exe() {
            if exe.starts_with(dir) && process.kill() {
                killed += 1;
            }
        }
    }
    Ok(killed)
}
