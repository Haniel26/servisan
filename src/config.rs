use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhpMyAdmin {
    /// Whether Servisan should discover and manage phpMyAdmin under `www`.
    pub enabled: bool,
    /// Laragon source folder copied during `servisan phpmyadmin install`.
    #[serde(default = "default_phpmyadmin_source")]
    pub source: String,
}
///
/// Every section and field has a default, so a partial (or empty) file still
/// produces a usable configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub php: Php,
    pub mysql: Mysql,
    pub phpmyadmin: PhpMyAdmin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    /// Web root, relative to the Servisan root (default `www`).
    pub web_root: String,
    /// Host interface the HTTP server binds to.
    pub host: String,
    /// Open the browser after services start (StartAll / `servisan start`).
    #[serde(default = "default_auto_open_browser")]
    pub auto_open_browser: bool,
    /// Restart all services automatically when Servisan starts.
    #[serde(default = "default_auto_start_all")]
    pub auto_start_all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Php {
    /// Informational build name, e.g. `php-8.3.30-Win32-vs16-x64`.
    pub version: String,
    /// HTTP engine used to serve PHP.
    pub engine: EngineKind,
    /// HTTP port (default 8080 to avoid clashing with Laragon's port 80).
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mysql {
    /// Informational build name, e.g. `mysql-8.4.3-winx64`.
    pub version: String,
    /// MySQL port (default 3307 to avoid clashing with Laragon's 3306).
    pub port: u16,
}

/// Selectable HTTP engine. Only `builtin` is implemented in the MVP; the
/// others are reserved so the switch is a config change, not a refactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    #[default]
    Builtin,
    Nginx,
    Apache,
}

impl Default for PhpMyAdmin {
    fn default() -> Self {
        Self {
            enabled: true,
            source: default_phpmyadmin_source(),
        }
    }
}

fn default_phpmyadmin_source() -> String {
    "D:\\laragon\\etc\\apps\\phpmyadmin".to_string()
}

fn default_auto_open_browser() -> bool {
    true
}

fn default_auto_start_all() -> bool {
    true
}

impl Default for General {
    fn default() -> Self {
        Self {
            web_root: "www".to_string(),
            host: "127.0.0.1".to_string(),
            auto_open_browser: true,
            auto_start_all: true,
        }
    }
}

impl Default for Php {
    fn default() -> Self {
        Self {
            version: "php-8.3.30-Win32-vs16-x64".to_string(),
            engine: EngineKind::default(),
            port: 8080,
        }
    }
}

impl Default for Mysql {
    fn default() -> Self {
        Self {
            version: "mysql-8.4.3-winx64".to_string(),
            port: 3307,
        }
    }
}

impl Config {
    /// Load configuration from `path`. Returns defaults if the file is missing.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Serialize to TOML and write to `path`, creating parent directories.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("serializing config to TOML")?;
        std::fs::write(path, text).with_context(|| format!("writing config {}", path.display()))?;
        Ok(())
    }

    /// Basic sanity checks beyond what serde enforces.
    pub fn validate(&self) -> Result<()> {
        if self.php.port == 0 {
            anyhow::bail!("php.port must not be 0");
        }
        if self.mysql.port == 0 {
            anyhow::bail!("mysql.port must not be 0");
        }
        if self.php.port == self.mysql.port {
            anyhow::bail!(
                "php.port and mysql.port must differ (both are {})",
                self.php.port
            );
        }
        if self.general.web_root.trim().is_empty() {
            anyhow::bail!("general.web_root must not be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let cfg = Config::default();
        cfg.validate().unwrap();
        assert_eq!(cfg.php.port, 8080);
        assert_eq!(cfg.mysql.port, 3307);
        assert_eq!(cfg.php.engine, EngineKind::Builtin);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let cfg: Config = toml::from_str("[mysql]\nport = 3399\n").unwrap();
        assert_eq!(cfg.mysql.port, 3399);
        assert_eq!(cfg.php.port, 8080);
        assert_eq!(cfg.general.web_root, "www");
    }

    #[test]
    fn roundtrip_preserves_values() {
        let mut cfg = Config::default();
        cfg.php.engine = EngineKind::Nginx;
        cfg.php.port = 9000;
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn rejects_equal_ports() {
        let mut cfg = Config::default();
        cfg.mysql.port = cfg.php.port;
        assert!(cfg.validate().is_err());
    }
}
