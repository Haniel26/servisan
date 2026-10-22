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

/// Random token handed to the dashboard for CSRF checks on its actions.
///
/// The value is only shared with the dashboard page and is regenerated on
/// every manager start, so other websites cannot forge local actions.
fn dashboard_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let mut token = String::with_capacity(32);
    for _ in 0..2 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or_default(),
        );
        token.push_str(&format!("{:016x}", hasher.finish()));
    }
    token
}

/// Whether the HTTP server only accepts connections from this machine.
pub fn is_loopback_host(host: &str) -> bool {
    let host = host.trim().trim_matches(|c| c == '[' || c == ']');
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    host.parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

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
        // The dashboard reads these through getenv() so app.toml (MySQL port)
        // and Cargo.toml (version) stay the source of truth.
        cmd.env("SERVISAN_MYSQL_HOST", "127.0.0.1");
        cmd.env("SERVISAN_MYSQL_PORT", cfg.mysql.port.to_string());
        cmd.env("SERVISAN_VERSION", crate::VERSION);
        cmd.env("SERVISAN_DASHBOARD_TOKEN", dashboard_token());
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
        if !is_loopback_host(&cfg.general.host) {
            eprintln!(
                "warning: dashboard binds to {} and is reachable from other machines; \
                 dashboard actions stay limited to this computer.",
                cfg.general.host
            );
        }
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

    fn command_env(cmd: &std::process::Command, name: &str) -> Option<String> {
        cmd.get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new(name))
            .and_then(|(_, value)| value)
            .map(|value| value.to_string_lossy().into_owned())
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

        assert_eq!(
            command_env(&cmd, "PHP_CLI_SERVER_WORKERS"),
            Some("4".to_string())
        );
        assert_eq!(
            command_env(&cmd, "SERVISAN_MYSQL_PORT"),
            Some("3307".to_string())
        );
        assert_eq!(
            command_env(&cmd, "SERVISAN_MYSQL_HOST"),
            Some("127.0.0.1".to_string())
        );
        assert_eq!(
            command_env(&cmd, "SERVISAN_VERSION"),
            Some(crate::VERSION.to_string())
        );
        let token = command_env(&cmd, "SERVISAN_DASHBOARD_TOKEN").expect("dashboard token");
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn dashboard_token_is_fresh_for_every_start() {
        let paths = Paths::new(r"D:\ServiSan");
        let cfg = cfg_for(Path::new("www"), 8080);
        let first = command_env(
            &BuiltinServer.command(&paths, &cfg),
            "SERVISAN_DASHBOARD_TOKEN",
        );
        let second = command_env(
            &BuiltinServer.command(&paths, &cfg),
            "SERVISAN_DASHBOARD_TOKEN",
        );
        assert!(first.is_some());
        assert_ne!(first, second);
    }

    #[test]
    fn loopback_hosts_are_recognised() {
        for host in [
            "127.0.0.1",
            "127.0.0.53",
            "localhost",
            "LOCALHOST",
            "::1",
            "[::1]",
        ] {
            assert!(is_loopback_host(host), "{host} should be loopback");
        }
        for host in [
            "0.0.0.0",
            "192.168.1.10",
            "10.0.0.5",
            "::",
            "example.com",
            "",
        ] {
            assert!(!is_loopback_host(host), "{host} should not be loopback");
        }
    }

    #[test]
    fn dashboard_env_tracks_configured_mysql_port() {
        let paths = Paths::new(r"D:\ServiSan");
        let mut cfg = cfg_for(Path::new("www"), 8080);
        cfg.mysql.port = 3310;
        let cmd = BuiltinServer.command(&paths, &cfg);

        assert_eq!(
            command_env(&cmd, "SERVISAN_MYSQL_PORT"),
            Some("3310".to_string())
        );
    }

    fn run_php_test(name: &str) {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(name);
        let output = match std::process::Command::new("php").arg(&script).output() {
            Ok(output) => output,
            Err(_) => {
                eprintln!("skipping: php not installed");
                return;
            }
        };

        assert!(
            output.status.success(),
            "{name} failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn dashboard_renders_when_mysql_is_offline() {
        run_php_test("dashboard_offline.php");
    }

    #[test]
    fn dashboard_actions_require_loopback_post_and_token() {
        run_php_test("dashboard_security.php");
    }

    #[test]
    fn dashboard_cli_args_never_pass_through_a_shell() {
        run_php_test("dashboard_cli.php");
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
