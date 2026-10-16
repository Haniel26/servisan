use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Resolves the Servisan "home" directory and all well-known sub-paths.
///
/// The home directory is discovered relative to the running executable so
/// that the whole folder stays portable: move it to another drive or machine
/// and everything keeps working without reconfiguration.
pub struct Paths {
    root: PathBuf,
    data_override: Option<PathBuf>,
}

impl Paths {
    /// Build a `Paths` from an explicit root (used in tests and overrides).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            data_override: None,
        }
    }

    /// Use a custom data directory instead of `<root>/data`.
    pub fn with_data_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.data_override = Some(dir.into());
        self
    }

    /// Discover the home directory.
    ///
    /// Resolution order:
    /// 1. `SERVISAN_HOME` environment variable, when it points to a directory.
    /// 2. The executable's directory or one of its ancestors that looks like a
    ///    Servisan root (contains `Cargo.toml` + `src`, or `bin` + `config`).
    pub fn discover() -> Result<Self> {
        if let Some(dir) = std::env::var_os("SERVISAN_HOME") {
            let dir = PathBuf::from(dir);
            if dir.is_dir() {
                return Ok(Self::new(dir));
            }
        }

        let exe = std::env::current_exe()
            .map_err(|e| anyhow!("cannot determine executable path: {e}"))?;
        let start = exe
            .parent()
            .ok_or_else(|| anyhow!("executable has no parent directory: {}", exe.display()))?;

        for dir in start.ancestors() {
            if is_root(dir) {
                return Ok(Self::new(dir));
            }
        }

        Err(anyhow!(
            "could not locate Servisan root above {} (set SERVISAN_HOME to override)",
            start.display()
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.root.join("bin")
    }

    pub fn php_dir(&self) -> PathBuf {
        self.bin_dir().join("php")
    }

    pub fn php_exe(&self) -> PathBuf {
        self.php_dir().join("php.exe")
    }

    pub fn php_cgi_exe(&self) -> PathBuf {
        self.php_dir().join("php-cgi.exe")
    }

    pub fn mysql_dir(&self) -> PathBuf {
        self.bin_dir().join("mysql")
    }

    pub fn mysqld_exe(&self) -> PathBuf {
        self.mysql_dir().join("bin").join("mysqld.exe")
    }

    pub fn mysqladmin_exe(&self) -> PathBuf {
        self.mysql_dir().join("bin").join("mysqladmin.exe")
    }

    /// True if the installed database server is MariaDB rather than Oracle MySQL.
    pub fn is_mariadb(&self) -> bool {
        let bin = self.mysql_dir().join("bin");
        bin.join("mariadb-install-db.exe").is_file() || bin.join("mysql_install_db.exe").is_file()
    }

    pub fn www_dir(&self) -> PathBuf {
        self.root.join("www")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_override
            .clone()
            .unwrap_or_else(|| self.root.join("data"))
    }

    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("app.toml")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.root.join("tmp")
    }

    pub fn phpmyadmin_dir(&self) -> PathBuf {
        self.www_dir().join("phpmyadmin")
    }

    /// Directories that must be on `PATH` for PHP and MySQL tools to be
    /// callable: the PHP CLI and the MySQL client tools.
    pub fn path_entries(&self) -> Vec<PathBuf> {
        vec![self.php_dir(), self.mysql_dir().join("bin")]
    }

    /// `current` with [`Self::path_entries`] prepended, so Servisan's tools win
    /// the lookup without dropping whatever the host already had.
    pub fn prepend_path(&self, current: impl AsRef<std::ffi::OsStr>) -> std::ffi::OsString {
        let mut out = std::ffi::OsString::new();
        for (i, entry) in self.path_entries().iter().enumerate() {
            if i > 0 {
                out.push(PATH_SEP);
            }
            out.push(entry);
        }

        let current = current.as_ref();
        if !current.is_empty() {
            if !out.is_empty() {
                out.push(PATH_SEP);
            }
            out.push(current);
        }
        out
    }

    /// Prepend Servisan's tool directories to `cmd`'s `PATH`, so the child can
    /// resolve sibling binaries and DLLs regardless of the inherited PATH.
    pub fn apply_path_env(&self, cmd: &mut std::process::Command) {
        let current = std::env::var_os("PATH").unwrap_or_default();
        cmd.env("PATH", self.prepend_path(current));
    }
}

/// Separator used by the platform's `PATH` environment variable.
#[cfg(windows)]
const PATH_SEP: &str = ";";
#[cfg(not(windows))]
const PATH_SEP: &str = ":";

/// Heuristic for "this directory is a Servisan root".
///
/// - Source layout: has `Cargo.toml` and `src/`.
/// - Distribution layout: has `bin/` and `config/`.
fn is_root(dir: &Path) -> bool {
    let cargo = dir.join("Cargo.toml").is_file() && dir.join("src").is_dir();
    let dist = dir.join("bin").is_dir() && dir.join("config").is_dir();
    cargo || dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_sub_paths_from_root() {
        let p = Paths::new(r"D:\ServiSan");
        assert_eq!(p.bin_dir(), PathBuf::from(r"D:\ServiSan\bin"));
        assert_eq!(p.php_exe(), PathBuf::from(r"D:\ServiSan\bin\php\php.exe"));
        assert_eq!(
            p.mysqld_exe(),
            PathBuf::from(r"D:\ServiSan\bin\mysql\bin\mysqld.exe")
        );
        assert_eq!(
            p.config_file(),
            PathBuf::from(r"D:\ServiSan\config\app.toml")
        );
    }

    #[test]
    fn detects_source_layout_root() {
        let dir = std::env::temp_dir().join("servisan_root_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
        assert!(is_root(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prepends_servisan_dirs_before_existing_path() {
        let p = Paths::new(r"D:\ServiSan");
        let out = p.prepend_path(r"C:\Windows\system32");
        let text = out.to_string_lossy();
        assert!(text.starts_with(r"D:\ServiSan\bin\php;"));
        assert!(text.contains(r"D:\ServiSan\bin\mysql\bin;"));
        assert!(text.ends_with(r"C:\Windows\system32"));
        assert_eq!(p.path_entries().len(), 2);
    }

    #[test]
    fn prepend_path_handles_empty_current() {
        let p = Paths::new(r"D:\ServiSan");
        let out = p.prepend_path("");
        let text = out.to_string_lossy();
        assert!(text.starts_with(r"D:\ServiSan\bin\php"));
        assert!(!text.ends_with(PATH_SEP));
    }

    #[test]
    fn apply_path_env_sets_command_path() {
        let p = Paths::new(r"D:\ServiSan");
        let mut cmd = std::process::Command::new("x");
        p.apply_path_env(&mut cmd);

        let path = cmd
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
            .map(|(_, value)| value)
            .expect("PATH env var applied");
        let path = path.expect("PATH set rather than cleared");

        let text = path.to_string_lossy();
        assert!(text.starts_with(r"D:\ServiSan\bin\php"));
        assert!(text.contains(r"D:\ServiSan\bin\mysql\bin"));
    }
}
