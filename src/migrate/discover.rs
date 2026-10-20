use std::path::Path;

use anyhow::Result;

use super::detect::DetectedSource;

/// System databases that should never be migrated.
const SYSTEM_DATABASES: &[&str] = &[
    "mysql",
    "information_schema",
    "performance_schema",
    "sys",
    "phpmyadmin",
    "test",
    "#innodb_redo",
    "#innodb_temp",
    "#innodb_undo",
];

/// A database discovered inside a detected source installation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscoveredDatabase {
    /// Database name (matches the sub-folder name in the data directory).
    pub name: String,
    /// Estimated size in bytes (sum of all files in the sub-folder).
    pub estimated_size: u64,
}

impl DiscoveredDatabase {
    /// Human-friendly size string (e.g. "2.3 MB", "850 KB").
    pub fn size_display(&self) -> String {
        format_size(self.estimated_size)
    }
}

/// Discover user databases inside a detected source's data directory.
///
/// Uses filesystem scanning (offline — the source MySQL does not need to be
/// running). Each sub-folder in the data directory that is not a system
/// database is treated as a user database.
pub fn discover_databases(source: &DetectedSource) -> Result<Vec<DiscoveredDatabase>> {
    discover_databases_in(&source.data_dir)
}

/// Discover databases from a raw data directory path.
pub fn discover_databases_in(data_dir: &Path) -> Result<Vec<DiscoveredDatabase>> {
    let mut databases = Vec::new();

    if !data_dir.is_dir() {
        return Ok(databases);
    }

    let entries = std::fs::read_dir(data_dir)?;
    for entry in entries.flatten() {
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip system databases and hidden/internal directories.
        if is_system_database(&name) || name.starts_with('.') {
            continue;
        }

        let estimated_size = dir_size_recursive(&entry.path());

        databases.push(DiscoveredDatabase {
            name,
            estimated_size,
        });
    }

    databases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(databases)
}

/// Returns true if the database name is a MySQL/MariaDB system database.
pub fn is_system_database(name: &str) -> bool {
    SYSTEM_DATABASES
        .iter()
        .any(|s| s.eq_ignore_ascii_case(name))
}

/// Recursively compute the total file size under `dir`.
fn dir_size_recursive(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += dir_size_recursive(&entry.path());
                }
            }
        }
    }
    total
}

/// Format a byte count into a human-readable string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_databases_are_excluded() {
        assert!(is_system_database("mysql"));
        assert!(is_system_database("information_schema"));
        assert!(is_system_database("performance_schema"));
        assert!(is_system_database("sys"));
        assert!(is_system_database("phpmyadmin"));
        assert!(is_system_database("#innodb_redo"));
        assert!(!is_system_database("blog"));
        assert!(!is_system_database("ecommerce"));
    }

    #[test]
    fn discovers_databases_from_filesystem() {
        let dir = std::env::temp_dir().join("servisan_discover_test");
        let _ = std::fs::remove_dir_all(&dir);

        // Create mock database directories.
        std::fs::create_dir_all(dir.join("blog")).unwrap();
        std::fs::write(dir.join("blog").join("table1.ibd"), "x".repeat(1024)).unwrap();

        std::fs::create_dir_all(dir.join("shop")).unwrap();
        std::fs::write(dir.join("shop").join("products.ibd"), "x".repeat(2048)).unwrap();

        // System databases should be excluded.
        std::fs::create_dir_all(dir.join("mysql")).unwrap();
        std::fs::create_dir_all(dir.join("performance_schema")).unwrap();
        std::fs::create_dir_all(dir.join("sys")).unwrap();

        let dbs = discover_databases_in(&dir).unwrap();
        let names: Vec<&str> = dbs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"blog"));
        assert!(names.contains(&"shop"));
        assert!(!names.contains(&"mysql"));
        assert!(!names.contains(&"performance_schema"));
        assert!(!names.contains(&"sys"));
        assert_eq!(dbs.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_size_values() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(2_500_000), "2.4 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn empty_dir_returns_no_databases() {
        let dir = std::env::temp_dir().join("servisan_discover_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let dbs = discover_databases_in(&dir).unwrap();
        assert!(dbs.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
