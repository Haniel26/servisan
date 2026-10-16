use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, ensure, Context, Result};

use crate::config::Config;
use crate::paths::Paths;

const SECRET_FILE: &str = "phpmyadmin.secret";

#[cfg(windows)]
fn random_hex_bytes(len: usize) -> Result<Vec<u8>> {
    use windows::Win32::Security::Cryptography::{
        BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG,
    };

    let mut bytes = vec![0u8; len];
    unsafe {
        BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG)
            .ok()
            .context("generating cryptographically secure bytes")?;
    }
    Ok(bytes)
}

pub fn phpmyadmin_dir(paths: &Paths) -> PathBuf {
    paths.phpmyadmin_dir()
}

pub fn is_installed(paths: &Paths) -> bool {
    paths
        .phpmyadmin_dir()
        .join("index.php")
        .is_file()
        && paths
            .phpmyadmin_dir()
            .join("libraries")
            .join("constants.php")
            .is_file()
}

pub fn configured_source(cfg: &Config) -> PathBuf {
    let source = cfg.phpmyadmin.source.trim();
    if source.is_empty() {
        PathBuf::from("D:\\laragon\\etc\\apps\\phpmyadmin")
    } else {
        PathBuf::from(source)
    }
}

pub fn secret_path(paths: &Paths) -> PathBuf {
    paths.config_dir().join("generated").join(SECRET_FILE)
}

pub fn generate_secret() -> Result<String> {
    let bytes = random_hex_bytes(32)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub fn ensure_secret(paths: &Paths) -> Result<String> {
    let path = secret_path(paths);
    if let Ok(text) = fs::read_to_string(&path) {
        let trimmed = text.trim();
        if trimmed.len() >= 32 {
            return Ok(trimmed.to_string());
        }
    }

    let bytes = random_hex_bytes(32)?;
    let secret = bytes.iter().map(|byte| format!("{byte:02x}")).collect();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, format!("{secret}\n"))
        .with_context(|| format!("writing phpMyAdmin secret {}", path.display()))?;
    Ok(secret)
}

pub fn install(paths: &Paths, cfg: &Config, source: Option<PathBuf>, force: bool) -> Result<()> {
    if !cfg.phpmyadmin.enabled {
        bail!("phpMyAdmin is disabled in config/app.toml");
    }

    let source = source.unwrap_or_else(|| configured_source(cfg));
    ensure!(
        source.is_dir(),
        "phpMyAdmin source not found: {}\n\
         Install or configure 'phpmyadmin.source', or pass --source.",
        source.display()
    );

    let destination = paths.phpmyadmin_dir();
    let destination_parent = destination
        .parent()
        .ok_or_else(|| anyhow!("phpMyAdmin destination has no parent"))?;
    fs::create_dir_all(destination_parent)
        .with_context(|| format!("creating {}", destination_parent.display()))?;

    if force && destination.exists() {
        fs::remove_dir_all(&destination)
            .with_context(|| format!("removing existing {}", destination.display()))?;
    }

    if destination.exists() && !is_empty_dir(&destination) {
        bail!(
            "phpMyAdmin destination is not empty: {}.\n\
             Use 'servisan phpmyadmin install --force' to replace it.",
            destination.display()
        );
    }

    fs::create_dir_all(&destination)
        .with_context(|| format!("creating {}", destination.display()))?;
    copy_dir_recursive(&source, &destination)?;
    fs::create_dir_all(destination.join("tmp"))
        .with_context(|| format!("creating {}\\tmp", destination.display()))?;

    ensure_secret(paths)?;
    configure(paths, cfg)?;
    Ok(())
}

pub fn configure(paths: &Paths, cfg: &Config) -> Result<()> {
    if !is_installed(paths) {
        return Ok(());
    }

    let secret = ensure_secret(paths)?;
    let config = render_config(paths, cfg, &secret);
    fs::write(paths.phpmyadmin_dir().join("config.inc.php"), config)
        .context("writing phpMyAdmin config.inc.php")?;
    Ok(())
}

pub fn render_config(_paths: &Paths, cfg: &Config, secret: &str) -> String {
    let host = "127.0.0.1";
    let port = cfg.mysql.port;
    format!(
        "<?php\n\
         declare(strict_types=1);\n\
         $cfg['blowfish_secret'] = '{secret}';\n\
         $i = 0;\n\
         $i++;\n\
         $cfg['Servers'][$i]['auth_type'] = 'cookie';\n\
         $cfg['Servers'][$i]['host'] = '{host}';\n\
         $cfg['Servers'][$i]['port'] = {port};\n\
         $cfg['Servers'][$i]['compress'] = false;\n\
         $cfg['Servers'][$i]['AllowNoPassword'] = true;\n\
         $cfg['LoginCookieValidity'] = 36000;\n\
         $cfg['TempDir'] = __DIR__ . '/tmp';\n\
         $cfg['UploadDir'] = '';\n\
         $cfg['SaveDir'] = '';\n",
        secret = secret,
        host = host,
        port = port
    )
}

pub fn url(cfg: &Config) -> String {
    format!("http://127.0.0.1:{}/phpmyadmin/", cfg.php.port)
}

pub fn open_url(url: &str) -> Result<()> {
    use std::process::Command;

    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(url)
        .spawn()
        .context("launching browser for phpMyAdmin")?;
    Ok(())
}

fn is_empty_dir(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from).with_context(|| format!("reading {}", from.display()))? {
        let entry = entry.with_context(|| format!("reading {}", from.display()))?;
        let file_type = entry.file_type().with_context(|| format!("stat {}", entry.path().display()))?;
        let destination = to.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&destination)
                .with_context(|| format!("creating {}", destination.display()))?;
            copy_dir_recursive(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &destination).with_context(|| {
                format!("copying {} to {}", entry.path().display(), destination.display())
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_paths(root: &Path) -> Paths {
        Paths::new(root)
    }

    #[test]
    fn detects_missing_phpmyadmin() {
        let dir = std::env::temp_dir().join("servisan_phpmyadmin_missing_test");
        let _ = fs::remove_dir_all(&dir);
        let paths = sample_paths(&dir);
        assert!(!is_installed(&paths));
    }

    #[test]
    fn render_config_uses_mysql_port_and_secret() {
        let dir = std::env::temp_dir().join("servisan_phpmyadmin_render");
        let paths = sample_paths(&dir);
        let cfg = Config::default();
        let rendered = render_config(&paths, &cfg, "0123456789abcdef0123456789abcdef");
        assert!(rendered.contains("$cfg['blowfish_secret'] = '0123456789abcdef0123456789abcdef';"));
        assert!(rendered.contains("$cfg['Servers'][$i]['host'] = '127.0.0.1';"));
        assert!(rendered.contains("$cfg['Servers'][$i]['port'] = 3307;"));
        assert!(rendered.contains("AllowNoPassword'] = true;"));
    }

    #[test]
    fn install_copies_tree_and_writes_managed_config() {
        let root = std::env::temp_dir().join("servisan_phpmyadmin_install");
        let source = std::env::temp_dir().join("servisan_phpmyadmin_source");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(source.join("libraries")).unwrap();
        fs::write(source.join("index.php"), "<?php").unwrap();
        fs::write(source.join("libraries/constants.php"), "<?php").unwrap();
        fs::create_dir_all(source.join("tmp")).unwrap();

        let paths = sample_paths(&root);
        let mut cfg = Config::default();
        cfg.phpmyadmin.source = source.display().to_string();

        install(&paths, &cfg, None, false).unwrap();
        assert!(is_installed(&paths));
        let config = fs::read_to_string(paths.phpmyadmin_dir().join("config.inc.php")).unwrap();
        assert!(config.contains("blowfish_secret"));
        assert!(config.contains("port'] = 3307;"));

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&source);
    }

    #[test]
    fn refuses_overwrite_by_default() {
        let root = std::env::temp_dir().join("servisan_phpmyadmin_refuse");
        let source = std::env::temp_dir().join("servisan_phpmyadmin_source_refuse");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(source.join("libraries")).unwrap();
        fs::write(source.join("index.php"), "<?php").unwrap();
        fs::write(source.join("libraries/constants.php"), "<?php").unwrap();

        let paths = sample_paths(&root);
        let mut cfg = Config::default();
        cfg.phpmyadmin.source = source.display().to_string();

        install(&paths, &cfg, None, false).unwrap();
        let err = install(&paths, &cfg, None, false).unwrap_err().to_string();
        assert!(err.contains("not empty"));

        fs::remove_dir_all(paths.phpmyadmin_dir()).unwrap();
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&source);
    }

    #[cfg(windows)]
    #[test]
    fn generates_secret_of_minimum_length() {
        let got = generate_secret().unwrap();
        assert!(got.len() >= 32);
        assert!(got.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
