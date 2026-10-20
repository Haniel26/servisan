use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use super::detect::DetectedSource;
use super::discover::{discover_databases, is_system_database, DiscoveredDatabase};
use crate::config::Config;
use crate::paths::Paths;

/// How to handle a database name that already exists in Servisan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Skip the import silently.
    Skip,
    /// Overwrite by dropping and recreating.
    Overwrite,
}

/// Options for a migration import operation.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// If non-empty, only import databases whose names appear here.
    /// An empty list means "import all discovered databases".
    pub filter: Vec<String>,
    /// How to handle name collisions with existing Servisan databases.
    pub on_conflict: ConflictStrategy,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            filter: Vec::new(),
            on_conflict: ConflictStrategy::Skip,
        }
    }
}

/// Outcome for a single database import attempt.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub name: String,
    pub status: ImportStatus,
}

#[derive(Debug, Clone)]
pub enum ImportStatus {
    /// Successfully imported.
    Ok { size_bytes: u64 },
    /// Skipped because it already exists and conflict strategy is Skip.
    Skipped,
    /// Failed with an error message.
    Failed(String),
}

/// Summary report for an entire migration run.
#[derive(Debug, Clone)]
pub struct MigrateReport {
    pub source_label: String,
    pub results: Vec<ImportResult>,
}

impl MigrateReport {
    pub fn imported_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, ImportStatus::Ok { .. }))
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, ImportStatus::Skipped))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| matches!(r.status, ImportStatus::Failed(_)))
            .count()
    }

    /// Print a human-readable summary to stdout.
    pub fn print_summary(&self) {
        println!();
        println!(
            "Migration complete: {} imported, {} skipped, {} failed.",
            self.imported_count(),
            self.skipped_count(),
            self.failed_count()
        );

        for r in &self.results {
            if let ImportStatus::Failed(msg) = &r.status {
                eprintln!("  FAILED {}: {}", r.name, msg);
            }
        }
    }
}

/// Run a complete migration from `source` into the local Servisan MySQL.
///
/// Prerequisites:
/// - Servisan's MySQL must be running on `cfg.mysql.port`.
/// - The source must either be running (for live dump) or have a readable
///   data directory (for filesystem discovery).
///
/// The function uses the source's `mysqldump` binary to export each database
/// and Servisan's `mysql` client to import the resulting SQL.
pub fn run_import(
    paths: &Paths,
    cfg: &Config,
    source: &DetectedSource,
    opts: &ImportOptions,
) -> Result<MigrateReport> {
    let source_label = format!("{} ({})", source.kind, source.root.display());

    // Validate that Servisan's MySQL is reachable.
    ensure_servisan_mysql_ready(cfg)?;

    // Resolve Servisan's mysql client binary.
    let servisan_mysql = resolve_servisan_mysql(paths)?;

    // Resolve mysqldump binary — prefer source's own, fall back to Servisan's.
    let mysqldump = resolve_mysqldump(paths, source)?;

    // Discover databases in the source.
    let databases = discover_databases(source)?;
    if databases.is_empty() {
        println!("No user databases found in {source_label}.");
        return Ok(MigrateReport {
            source_label,
            results: Vec::new(),
        });
    }

    // Filter to requested databases if specified.
    let databases: Vec<&DiscoveredDatabase> = if opts.filter.is_empty() {
        databases.iter().collect()
    } else {
        databases
            .iter()
            .filter(|db| opts.filter.iter().any(|f| f.eq_ignore_ascii_case(&db.name)))
            .collect()
    };

    if databases.is_empty() {
        println!("No matching databases found for the specified filter.");
        return Ok(MigrateReport {
            source_label,
            results: Vec::new(),
        });
    }

    // Get existing databases in Servisan to detect conflicts.
    let existing = list_servisan_databases(&servisan_mysql, cfg)?;

    println!("Migrating databases from {source_label}...\n");

    // If source MySQL is currently running, use its port directly.
    // If NOT running, spin up a silent temporary headless instance on an ephemeral port!
    let (effective_port, _temp_server) = if source.is_running {
        (source.mysql_port, None)
    } else {
        let s = start_temporary_source_mysqld(paths, source)?;
        let p = s.port;
        (p, Some(s))
    };

    let total = databases.len();
    let mut results = Vec::with_capacity(total);

    for (i, db) in databases.iter().enumerate() {
        let idx = format!("[{}/{}]", i + 1, total);

        // Check for conflict.
        if existing.iter().any(|e| e.eq_ignore_ascii_case(&db.name)) {
            match opts.on_conflict {
                ConflictStrategy::Skip => {
                    println!(
                        "  {idx} {:<24} {:>10}  SKIPPED (already exists)",
                        db.name,
                        db.size_display()
                    );
                    results.push(ImportResult {
                        name: db.name.clone(),
                        status: ImportStatus::Skipped,
                    });
                    continue;
                }
                ConflictStrategy::Overwrite => {
                    // Drop existing database before import.
                    if let Err(e) = drop_database(&servisan_mysql, cfg, &db.name) {
                        let msg = format!("failed to drop existing database: {e}");
                        println!("  {idx} {:<24} {:>10}  FAILED", db.name, db.size_display());
                        results.push(ImportResult {
                            name: db.name.clone(),
                            status: ImportStatus::Failed(msg),
                        });
                        continue;
                    }
                }
            }
        }

        // Perform the dump + import using effective_port.
        match import_single_database(
            &mysqldump,
            &servisan_mysql,
            paths,
            cfg,
            effective_port,
            &db.name,
        ) {
            Ok(()) => {
                println!("  {idx} {:<24} {:>10}  OK", db.name, db.size_display());
                results.push(ImportResult {
                    name: db.name.clone(),
                    status: ImportStatus::Ok {
                        size_bytes: db.estimated_size,
                    },
                });
            }
            Err(e) => {
                let msg = format!("{e}");
                println!("  {idx} {:<24} {:>10}  FAILED", db.name, db.size_display());
                results.push(ImportResult {
                    name: db.name.clone(),
                    status: ImportStatus::Failed(msg),
                });
            }
        }
    }

    let report = MigrateReport {
        source_label,
        results,
    };
    report.print_summary();
    Ok(report)
}

/// A temporary instance of the source's MySQL server running on an ephemeral port.
/// Automatically terminated and cleaned up on drop.
pub struct TemporarySourceServer {
    child: Option<std::process::Child>,
    pub port: u16,
}

impl Drop for TemporarySourceServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            println!(
                "  Stopping temporary source MySQL instance (port {})...",
                self.port
            );
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Start a temporary headless mysqld instance from the source binary on an ephemeral free port.
pub fn start_temporary_source_mysqld(
    paths: &Paths,
    source: &DetectedSource,
) -> Result<TemporarySourceServer> {
    let mysqld = if let Some(ref exe) = source.mysqld_exe {
        exe.clone()
    } else {
        let servisan_mysqld = paths.mysql_dir().join("bin").join("mysqld.exe");
        if servisan_mysqld.is_file() {
            servisan_mysqld
        } else {
            bail!(
                "could not find mysqld.exe in source ({}) or Servisan to start temporary server",
                source.root.display()
            );
        }
    };

    // Find an unused local port
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("finding available port for temporary source server")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    println!("  Source MySQL is offline; starting silent headless instance on port {port} ...");

    let mut cmd = Command::new(&mysqld);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.arg(format!("--datadir={}", source.data_dir.display()))
        .arg(format!("--port={port}"))
        .arg("--bind-address=127.0.0.1")
        .arg("--skip-grant-tables")
        .arg("--skip-networking=0");

    let child = cmd
        .spawn()
        .with_context(|| format!("spawning temporary source mysqld from {}", mysqld.display()))?;

    // Wait up to 15 seconds for the server to accept connections
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let mut ready = false;

    while std::time::Instant::now() < deadline {
        if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    if !ready {
        let mut c = child;
        let _ = c.kill();
        let _ = c.wait();
        bail!("temporary source MySQL did not become ready on port {port} within 15 seconds");
    }

    println!("  Temporary source MySQL ready on port {port}.\n");

    Ok(TemporarySourceServer {
        child: Some(child),
        port,
    })
}

/// Import a single database: mysqldump from source port → pipe into Servisan mysql.
fn import_single_database(
    mysqldump: &Path,
    servisan_mysql: &Path,
    paths: &Paths,
    cfg: &Config,
    source_port: u16,
    db_name: &str,
) -> Result<()> {
    // Ensure temporary directory exists for dump files.
    let tmp_dir = paths.tmp_dir().join("migrate");
    std::fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("creating temp dir {}", tmp_dir.display()))?;

    let dump_file = tmp_dir.join(format!("{db_name}.sql"));

    // Step 1: mysqldump from source.
    dump_database(mysqldump, source_port, db_name, &dump_file)?;

    // Step 2: Create the database in Servisan.
    create_database(servisan_mysql, cfg, db_name)?;

    // Step 3: Import the dump into Servisan.
    import_dump_file(servisan_mysql, cfg, db_name, &dump_file)?;

    // Step 4: Cleanup temporary file.
    let _ = std::fs::remove_file(&dump_file);

    Ok(())
}

/// Run mysqldump to export a single database to a SQL file.
fn dump_database(
    mysqldump: &Path,
    source_port: u16,
    db_name: &str,
    output_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new(mysqldump);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.arg("--host=127.0.0.1")
        .arg(format!("--port={source_port}"))
        .arg("--user=root")
        .arg("--single-transaction")
        .arg("--routines")
        .arg("--triggers")
        .arg("--events")
        .arg("--set-gtid-purged=OFF")
        .arg("--result-file")
        .arg(output_path)
        .arg(db_name);

    let output = cmd
        .output()
        .with_context(|| format!("running mysqldump for database '{db_name}'"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // MariaDB does not support --set-gtid-purged; retry without it.
        if stderr.contains("unknown option") && stderr.contains("gtid") {
            return dump_database_mariadb(mysqldump, source_port, db_name, output_path);
        }
        bail!("mysqldump failed for '{db_name}': {}", stderr.trim());
    }

    Ok(())
}

/// MariaDB-compatible mysqldump (without --set-gtid-purged).
fn dump_database_mariadb(
    mysqldump: &Path,
    source_port: u16,
    db_name: &str,
    output_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new(mysqldump);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.arg("--host=127.0.0.1")
        .arg(format!("--port={source_port}"))
        .arg("--user=root")
        .arg("--single-transaction")
        .arg("--routines")
        .arg("--triggers")
        .arg("--events")
        .arg("--result-file")
        .arg(output_path)
        .arg(db_name);

    let output = cmd
        .output()
        .with_context(|| format!("running mysqldump (MariaDB mode) for '{db_name}'"))?;

    if !output.status.success() {
        bail!(
            "mysqldump failed for '{db_name}': {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

/// Create a database in Servisan's MySQL (if it does not exist).
fn create_database(mysql_client: &Path, cfg: &Config, db_name: &str) -> Result<()> {
    let query = format!("CREATE DATABASE IF NOT EXISTS `{db_name}`;");
    run_mysql_query(mysql_client, cfg, None, &query)
        .with_context(|| format!("creating database '{db_name}'"))
}

/// Drop a database in Servisan's MySQL.
fn drop_database(mysql_client: &Path, cfg: &Config, db_name: &str) -> Result<()> {
    let query = format!("DROP DATABASE IF EXISTS `{db_name}`;");
    run_mysql_query(mysql_client, cfg, None, &query)
        .with_context(|| format!("dropping database '{db_name}'"))
}

/// Import a SQL dump file into a specific database.
fn import_dump_file(
    mysql_client: &Path,
    cfg: &Config,
    db_name: &str,
    dump_path: &Path,
) -> Result<()> {
    let mut cmd = Command::new(mysql_client);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.arg("--host=127.0.0.1")
        .arg(format!("--port={}", cfg.mysql.port))
        .arg("--user=root")
        .arg(db_name);

    // Redirect the dump file as stdin.
    let dump_file = std::fs::File::open(dump_path)
        .with_context(|| format!("opening dump file {}", dump_path.display()))?;
    cmd.stdin(dump_file);

    let output = cmd
        .output()
        .with_context(|| format!("importing dump into '{db_name}'"))?;

    if !output.status.success() {
        bail!(
            "import failed for '{db_name}': {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

/// Run a single SQL query against Servisan's MySQL.
fn run_mysql_query(
    mysql_client: &Path,
    cfg: &Config,
    database: Option<&str>,
    query: &str,
) -> Result<()> {
    let mut cmd = Command::new(mysql_client);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.arg("--host=127.0.0.1")
        .arg(format!("--port={}", cfg.mysql.port))
        .arg("--user=root")
        .arg("-e")
        .arg(query);

    if let Some(db) = database {
        cmd.arg(db);
    }

    let output = cmd.output().context("running mysql query")?;

    if !output.status.success() {
        bail!(
            "query failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

/// List all existing databases in Servisan's MySQL instance.
fn list_servisan_databases(mysql_client: &Path, cfg: &Config) -> Result<Vec<String>> {
    let mut cmd = Command::new(mysql_client);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.arg("--host=127.0.0.1")
        .arg(format!("--port={}", cfg.mysql.port))
        .arg("--user=root")
        .arg("--batch")
        .arg("--skip-column-names")
        .arg("-e")
        .arg("SHOW DATABASES;");

    let output = cmd.output().context("listing Servisan databases")?;

    if !output.status.success() {
        bail!(
            "failed to list Servisan databases: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let databases: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|name| !name.is_empty() && !is_system_database(name))
        .collect();

    Ok(databases)
}

/// Verify that Servisan's MySQL server is accepting connections.
fn ensure_servisan_mysql_ready(cfg: &Config) -> Result<()> {
    let addr = format!("127.0.0.1:{}", cfg.mysql.port);
    let addr: std::net::SocketAddr = addr.parse().context("parsing Servisan MySQL address")?;

    if std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_err() {
        bail!(
            "Servisan's MySQL is not running on port {}.\n\
             Start Servisan first with 'servisan start' or via the system tray, then retry.",
            cfg.mysql.port
        );
    }
    Ok(())
}

/// Resolve the mysql client binary from Servisan's bin directory.
fn resolve_servisan_mysql(paths: &Paths) -> Result<PathBuf> {
    let bin = paths.mysql_dir().join("bin");
    for name in ["mysql.exe", "mariadb.exe"] {
        let p = bin.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }
    bail!(
        "MySQL client binary not found in {}.\n\
         Run scripts/install-binaries.ps1 to install MySQL/MariaDB.",
        bin.display()
    );
}

/// Resolve the mysqldump binary — prefer the source's own binary,
/// fall back to Servisan's bundled one.
fn resolve_mysqldump(paths: &Paths, source: &DetectedSource) -> Result<PathBuf> {
    // Prefer source's own mysqldump for best compatibility.
    if let Some(ref dump) = source.mysqldump_exe {
        if dump.is_file() {
            return Ok(dump.clone());
        }
    }

    // Fall back to Servisan's bundled mysqldump.
    let bin = paths.mysql_dir().join("bin");
    for name in ["mysqldump.exe", "mariadb-dump.exe"] {
        let p = bin.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }

    bail!(
        "mysqldump binary not found.\n\
         Neither the source ({}) nor Servisan has a mysqldump executable.",
        source.root.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_import_options() {
        let opts = ImportOptions::default();
        assert!(opts.filter.is_empty());
        assert_eq!(opts.on_conflict, ConflictStrategy::Skip);
    }

    #[test]
    fn migrate_report_counts() {
        let report = MigrateReport {
            source_label: "Test".to_string(),
            results: vec![
                ImportResult {
                    name: "db1".to_string(),
                    status: ImportStatus::Ok { size_bytes: 100 },
                },
                ImportResult {
                    name: "db2".to_string(),
                    status: ImportStatus::Skipped,
                },
                ImportResult {
                    name: "db3".to_string(),
                    status: ImportStatus::Failed("error".to_string()),
                },
                ImportResult {
                    name: "db4".to_string(),
                    status: ImportStatus::Ok { size_bytes: 200 },
                },
            ],
        };
        assert_eq!(report.imported_count(), 2);
        assert_eq!(report.skipped_count(), 1);
        assert_eq!(report.failed_count(), 1);
    }
}
