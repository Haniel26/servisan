use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// The kind of local server installation detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SourceKind {
    Laragon,
    Xampp,
    Wamp,
    Custom,
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Laragon => write!(f, "Laragon"),
            Self::Xampp => write!(f, "XAMPP"),
            Self::Wamp => write!(f, "WAMP"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// A detected local server installation with its MySQL data directory and tools.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedSource {
    pub kind: SourceKind,
    /// Root directory of the installation, e.g. `C:\laragon`.
    pub root: PathBuf,
    /// MySQL data directory containing database sub-folders.
    pub data_dir: PathBuf,
    /// Path to `mysqld.exe` (or `mariadbd.exe`) server binary if available.
    pub mysqld_exe: Option<PathBuf>,
    /// Path to `mysqldump.exe` (or `mariadb-dump.exe`) for live exports.
    pub mysqldump_exe: Option<PathBuf>,
    /// Path to `mysql.exe` (or `mariadb.exe`) client binary.
    pub mysql_client_exe: Option<PathBuf>,
    /// Configured MySQL port (parsed from config files when possible).
    pub mysql_port: u16,
    /// Whether the source MySQL server appears to be running on its port.
    pub is_running: bool,
}

impl fmt::Display for DetectedSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_running {
            "running"
        } else {
            "stopped"
        };
        write!(
            f,
            "{:<8} {}  (MySQL port {}, {})",
            self.kind,
            self.root.display(),
            self.mysql_port,
            status
        )
    }
}

/// Scan the system for local server installations (Laragon, XAMPP, WAMP).
///
/// Checks all available drive letters for known installation paths.
pub fn scan_sources() -> Result<Vec<DetectedSource>> {
    let mut sources = Vec::new();

    for drive in available_drives() {
        // Laragon: {Drive}:\laragon
        let laragon = PathBuf::from(format!(r"{}:\laragon", drive));
        if laragon.is_dir() {
            if let Some(src) = detect_laragon(&laragon) {
                sources.push(src);
            }
        }

        // XAMPP: {Drive}:\xampp
        let xampp = PathBuf::from(format!(r"{}:\xampp", drive));
        if xampp.is_dir() {
            if let Some(src) = detect_xampp(&xampp) {
                sources.push(src);
            }
        }

        // WAMP: {Drive}:\wamp64 or {Drive}:\wamp
        for name in ["wamp64", "wamp"] {
            let wamp = PathBuf::from(format!(r"{}:\{}", drive, name));
            if wamp.is_dir() {
                if let Some(src) = detect_wamp(&wamp) {
                    sources.push(src);
                }
            }
        }
    }

    Ok(sources)
}

/// Build a `DetectedSource` from a custom directory path.
///
/// Can accept either an installation root (like `D:\laragon` or `E:\xampp`)
/// or a direct data directory (like `D:\laragon\data\mysql` or `E:\my_data`).
pub fn custom_source(path: PathBuf, port: u16) -> DetectedSource {
    let root = path.clone();
    let mut data_dir = path.clone();
    let mut mysqld_exe = None;
    let mut mysqldump_exe = None;
    let mut mysql_client_exe = None;

    // Check if the path is an installation root containing known data directories
    for sub in [
        PathBuf::from(r"data\mysql"),
        PathBuf::from(r"mysql\data"),
        PathBuf::from("data"),
    ] {
        let candidate = path.join(&sub);
        if candidate.is_dir() {
            data_dir = candidate;
            break;
        }
    }

    // Try to discover binaries relative to root or data_dir
    let search_roots = [
        path.clone(),
        path.join("bin").join("mysql"),
        path.join("mysql").join("bin"),
        path.join("bin"),
        data_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.clone()),
    ];

    for sr in &search_roots {
        if let Some(bin) = find_latest_version_bin(sr) {
            mysqld_exe = resolve_server_exe(Some(&bin));
            mysqldump_exe = resolve_dump_exe(Some(&bin));
            mysql_client_exe = resolve_client_exe(Some(&bin));
            if mysqld_exe.is_some() || mysqldump_exe.is_some() {
                break;
            }
        }
        if sr.is_dir() {
            mysqld_exe = resolve_server_exe(Some(sr));
            mysqldump_exe = resolve_dump_exe(Some(sr));
            mysql_client_exe = resolve_client_exe(Some(sr));
            if mysqld_exe.is_some() || mysqldump_exe.is_some() {
                break;
            }
        }
    }

    let is_running = port_probe(port);

    DetectedSource {
        kind: SourceKind::Custom,
        root,
        data_dir,
        mysqld_exe,
        mysqldump_exe,
        mysql_client_exe,
        mysql_port: port,
        is_running,
    }
}

// ---------------------------------------------------------------------------
// Laragon detection
// ---------------------------------------------------------------------------

fn detect_laragon(root: &Path) -> Option<DetectedSource> {
    let data_dir = root.join("data").join("mysql");
    if !data_dir.is_dir() {
        return None;
    }

    // Find MySQL binaries inside bin/mysql/{version}/bin/
    let mysql_bin = find_latest_version_bin(&root.join("bin").join("mysql"));
    let mysqld_exe = resolve_server_exe(mysql_bin.as_deref());
    let mysqldump_exe = resolve_dump_exe(mysql_bin.as_deref());
    let mysql_client_exe = resolve_client_exe(mysql_bin.as_deref());

    // Parse port from usr/laragon.ini
    let port = parse_laragon_port(root).unwrap_or(3306);
    let is_running = port_probe(port);

    Some(DetectedSource {
        kind: SourceKind::Laragon,
        root: root.to_path_buf(),
        data_dir,
        mysqld_exe,
        mysqldump_exe,
        mysql_client_exe,
        mysql_port: port,
        is_running,
    })
}

/// Parse `MySQL Port=3306` from Laragon's `usr/laragon.ini`.
fn parse_laragon_port(root: &Path) -> Option<u16> {
    let ini_path = root.join("usr").join("laragon.ini");
    let text = std::fs::read_to_string(&ini_path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("MySQL Port=") {
            return rest.trim().parse().ok();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// XAMPP detection
// ---------------------------------------------------------------------------

fn detect_xampp(root: &Path) -> Option<DetectedSource> {
    let data_dir = root.join("mysql").join("data");
    if !data_dir.is_dir() {
        return None;
    }

    let bin_dir = root.join("mysql").join("bin");
    let mysqld_exe = resolve_server_exe(Some(&bin_dir));
    let mysqldump_exe = resolve_dump_exe(Some(&bin_dir));
    let mysql_client_exe = resolve_client_exe(Some(&bin_dir));

    // Parse port from mysql/bin/my.ini
    let port = parse_ini_port(&root.join("mysql").join("bin").join("my.ini")).unwrap_or(3306);
    let is_running = port_probe(port);

    Some(DetectedSource {
        kind: SourceKind::Xampp,
        root: root.to_path_buf(),
        data_dir,
        mysqld_exe,
        mysqldump_exe,
        mysql_client_exe,
        mysql_port: port,
        is_running,
    })
}

// ---------------------------------------------------------------------------
// WAMP detection
// ---------------------------------------------------------------------------

fn detect_wamp(root: &Path) -> Option<DetectedSource> {
    // WAMP stores MySQL under bin/mysql/{version}/, pick the latest version.
    let mysql_base = root.join("bin").join("mysql");
    let version_dir = find_latest_version_dir(&mysql_base)?;

    let data_dir = version_dir.join("data");
    if !data_dir.is_dir() {
        return None;
    }

    let bin_dir = version_dir.join("bin");
    let mysqld_exe = resolve_server_exe(Some(&bin_dir));
    let mysqldump_exe = resolve_dump_exe(Some(&bin_dir));
    let mysql_client_exe = resolve_client_exe(Some(&bin_dir));

    // Parse port from the version's my.ini
    let port = parse_ini_port(&version_dir.join("my.ini"))
        .or_else(|| parse_ini_port(&bin_dir.join("my.ini")))
        .unwrap_or(3306);
    let is_running = port_probe(port);

    Some(DetectedSource {
        kind: SourceKind::Wamp,
        root: root.to_path_buf(),
        data_dir,
        mysqld_exe,
        mysqldump_exe,
        mysql_client_exe,
        mysql_port: port,
        is_running,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Parse `port=3306` from a standard MySQL `my.ini` / `my.cnf`.
fn parse_ini_port(path: &Path) -> Option<u16> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_mysqld = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_mysqld = line == "[mysqld]";
            continue;
        }
        if in_mysqld {
            if let Some(rest) = line.strip_prefix("port") {
                let rest = rest.trim_start_matches(|c: char| c == '=' || c.is_whitespace());
                if let Some(val) = rest.split_whitespace().next() {
                    return val.parse().ok();
                }
            }
        }
    }
    None
}

/// Find the latest version sub-directory inside a MySQL base dir.
///
/// E.g. given `C:\laragon\bin\mysql`, returns the path to the directory
/// with the highest version name like `mariadb-11.4.5-winx64`.
fn find_latest_version_dir(base: &Path) -> Option<PathBuf> {
    if !base.is_dir() {
        return None;
    }
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(base)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    candidates.sort();
    candidates.last().cloned()
}

/// Find the `bin/` sub-directory inside the latest MySQL version directory.
///
/// E.g. `C:\laragon\bin\mysql\mariadb-11.4.5-winx64\bin`.
fn find_latest_version_bin(base: &Path) -> Option<PathBuf> {
    let version_dir = find_latest_version_dir(base)?;
    let bin = version_dir.join("bin");
    if bin.is_dir() {
        Some(bin)
    } else {
        None
    }
}

/// Resolve the `mysqld` (or `mariadbd`) server executable inside a `bin/` directory.
fn resolve_server_exe(bin_dir: Option<&Path>) -> Option<PathBuf> {
    let bin = bin_dir?;
    for name in ["mysqld.exe", "mariadbd.exe"] {
        let p = bin.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Resolve the `mysqldump` (or `mariadb-dump`) executable inside a `bin/` directory.
fn resolve_dump_exe(bin_dir: Option<&Path>) -> Option<PathBuf> {
    let bin = bin_dir?;
    for name in ["mysqldump.exe", "mariadb-dump.exe"] {
        let p = bin.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Resolve the `mysql` (or `mariadb`) client executable inside a `bin/` directory.
fn resolve_client_exe(bin_dir: Option<&Path>) -> Option<PathBuf> {
    let bin = bin_dir?;
    for name in ["mysql.exe", "mariadb.exe"] {
        let p = bin.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Return available Windows drive letters (A–Z) that exist.
pub fn available_drives() -> Vec<char> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let path = format!("{}:\\", letter as char);
        if Path::new(&path).exists() {
            drives.push(letter as char);
        }
    }
    drives
}

/// Quick TCP probe to check if a port is listening.
fn port_probe(port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = format!("127.0.0.1:{}", port);
    if let Ok(addr) = addr.parse() {
        TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mysql_ini_port() {
        let dir = std::env::temp_dir().join("servisan_detect_port");
        let _ = std::fs::create_dir_all(&dir);
        let ini = dir.join("my.ini");
        std::fs::write(
            &ini,
            "[client]\nport=3306\n\n[mysqld]\nport=3307\ndatadir=C:/data\n",
        )
        .unwrap();
        assert_eq!(parse_ini_port(&ini), Some(3307));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_laragon_ini_port() {
        let dir = std::env::temp_dir().join("servisan_detect_laragon");
        let usr = dir.join("usr");
        let _ = std::fs::create_dir_all(&usr);
        std::fs::write(usr.join("laragon.ini"), "MySQL Port=3308\n").unwrap();
        assert_eq!(parse_laragon_port(&dir), Some(3308));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn available_drives_returns_at_least_c() {
        let drives = available_drives();
        assert!(drives.contains(&'C'), "expected at least C: drive to exist");
    }

    #[test]
    fn custom_source_builds_correctly() {
        let src = custom_source(PathBuf::from(r"E:\mydata"), 3399);
        assert_eq!(src.kind, SourceKind::Custom);
        assert_eq!(src.mysql_port, 3399);
        assert!(src.mysqldump_exe.is_none());
        assert!(src.mysqld_exe.is_none());
    }

    #[test]
    fn display_source_kind() {
        assert_eq!(format!("{}", SourceKind::Laragon), "Laragon");
        assert_eq!(format!("{}", SourceKind::Xampp), "XAMPP");
        assert_eq!(format!("{}", SourceKind::Wamp), "WAMP");
        assert_eq!(format!("{}", SourceKind::Custom), "Custom");
    }
}
