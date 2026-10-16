use std::path::Path;
use std::process::Command;

use anyhow::{ensure, Result};

use crate::config::Config;
use crate::paths::Paths;
use crate::process::ManagedChild;

/// How PHP is served. The MVP ships only [`BuiltinServer`]; `Nginx`/`Apache`
/// engines can be added later without touching the service lifecycle code.
pub trait HttpEngine {
    /// Short identifier used in logs and status output.
    fn name(&self) -> &'static str;

    /// Build the command that serves the configured web root.
    fn command(&self, paths: &Paths, cfg: &Config) -> Command;
}

/// PHP's built-in web server (`php -S host:port -t <root>`).
///
/// Note: on Windows this server handles one request at a time, which is fine
/// for local development. Swap the engine in Phase 2 for concurrency.
pub struct BuiltinServer;

impl HttpEngine for BuiltinServer {
    fn name(&self) -> &'static str {
        "php-builtin"
    }

    fn command(&self, paths: &Paths, cfg: &Config) -> Command {
        let mut cmd = Command::new(paths.php_exe());
        let addr = format!("{}:{}", cfg.general.host, cfg.php.port);
        let root = paths.root().join(&cfg.general.web_root);

        if crate::ini::php_ini_path(paths).is_file() {
            cmd.arg("-c").arg(crate::ini::php_ini_path(paths));
        }
        cmd.arg("-S").arg(addr);
        cmd.arg("-t").arg(root);
        if cfg.php.workers > 1 {
            cmd.env("PHP_CLI_SERVER_WORKERS", cfg.php.workers.to_string());
        }
        paths.apply_path_env(&mut cmd);
        cmd
    }
}

/// Manages the PHP HTTP process lifecycle and exposes its status.
pub struct PhpService {
    engine: Box<dyn HttpEngine>,
    child: ManagedChild,
    port: u16,
}

impl PhpService {
    /// Start the PHP HTTP process using `engine`.
    pub fn start(paths: &Paths, cfg: &Config, engine: Box<dyn HttpEngine>) -> Result<Self> {
        let php = paths.php_exe();
        ensure!(
            php.is_file(),
            "PHP executable not found at {}\nRun scripts/install-binaries.ps1 first.",
            php.display()
        );

        let cmd = engine.command(paths, cfg);
        let log = paths.logs_dir().join("php.log");
        let child = ManagedChild::spawn(format!("php ({})", engine.name()), cmd, log)?;

        Ok(Self {
            engine,
            child,
            port: cfg.php.port,
        })
    }

    pub fn engine_name(&self) -> &'static str {
        self.engine.name()
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

    pub fn stop(&mut self) -> Result<()> {
        self.child.stop()
    }

    pub fn log_path(&self) -> &Path {
        self.child.log_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::paths::Paths;

    fn cfg_for(root: &Path, port: u16) -> Config {
        let mut cfg = Config::default();
        cfg.general.web_root = root.to_string_lossy().into_owned();
        cfg.php.port = port;
        cfg
    }

    #[test]
    fn builtin_command_uses_php_and_port() {
        let paths = Paths::new(r"D:\ServiSan");
        let cfg = cfg_for(Path::new("www"), 8080);
        let cmd = BuiltinServer.command(&paths, &cfg);
        assert_eq!(cmd.get_program(), paths.php_exe());
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"-S".to_string()));
        assert!(args.contains(&"127.0.0.1:8080".to_string()));
        assert!(args.contains(&"-t".to_string()));

        let workers_env = cmd
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("PHP_CLI_SERVER_WORKERS"))
            .and_then(|(_, val)| val)
            .map(|val| val.to_string_lossy().into_owned());
        assert_eq!(workers_env, Some("4".to_string()));
    }

    #[test]
    fn builtin_server_actually_serves() {
        let paths = match Paths::discover() {
            Ok(p) => p,
            Err(_) => return,
        };
        if !paths.php_exe().is_file() {
            eprintln!("skipping: php.exe not installed");
            return;
        }

        // Temp web root + generated php.ini.
        let root = std::env::temp_dir().join("servisan_php_root");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("index.php"), "<?php echo 'ok';").unwrap();

        let port = free_port();
        let cfg = cfg_for(&root, port);
        crate::ini::write_all(&paths, &cfg).unwrap();

        let mut svc = PhpService::start(&paths, &cfg, Box::new(BuiltinServer)).unwrap();
        let ready = wait_for_port(&cfg.general.host, port, 20);
        assert!(ready, "php builtin server did not open port {port}");
        assert!(svc.is_running());

        svc.stop().unwrap();
        let _ = std::fs::remove_dir_all(&root);
    }

    fn free_port() -> u16 {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    }

    fn wait_for_port(host: &str, port: u16, tries: u32) -> bool {
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;
        let addr = format!("{host}:{port}")
            .to_socket_addrs()
            .ok()
            .and_then(|mut it| it.next());
        let addr = match addr {
            Some(a) => a,
            None => return false,
        };
        for _ in 0..tries {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        false
    }
}
