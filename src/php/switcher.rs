use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::paths::Paths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhpVersionCandidate {
    pub id: String,
    pub version: String,
    pub path: PathBuf,
    pub source: String,
    pub is_active: bool,
    pub has_security_warning: bool,
    pub security_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpDownloadTarget {
    pub id: &'static str,
    pub version: &'static str,
    pub label: &'static str,
    pub url: &'static str,
    pub note: &'static str,
    pub is_installed: bool,
}

pub const OFFICIAL_DOWNLOAD_TARGETS: &[PhpDownloadTarget] = &[
    PhpDownloadTarget {
        id: "8.5",
        version: "8.5.12",
        label: "PHP 8.5 (Latest)",
        url: "https://windows.php.net/downloads/releases/php-8.5.12-Win32-vs17-x64.zip",
        note: "Latest official release from php.net (VS17 x64 Thread-Safe)",
        is_installed: false,
    },
    PhpDownloadTarget {
        id: "8.4",
        version: "8.4.27",
        label: "PHP 8.4 (Secured)",
        url: "https://windows.php.net/downloads/releases/php-8.4.27-Win32-vs17-x64.zip",
        note: "Latest stable release with property hooks & asymmetric visibility",
        is_installed: false,
    },
    PhpDownloadTarget {
        id: "8.3",
        version: "8.3.36",
        label: "PHP 8.3 (Secured)",
        url: "https://windows.php.net/downloads/releases/php-8.3.36-Win32-vs16-x64.zip",
        note: "Critical security patch fixing CVE-2026-109618 & CVE-2026-109619 RCE",
        is_installed: false,
    },
    PhpDownloadTarget {
        id: "8.2",
        version: "8.2.35",
        label: "PHP 8.2 (Secured)",
        url: "https://windows.php.net/downloads/releases/php-8.2.35-Win32-vs16-x64.zip",
        note: "Patched LTS release for stable production & Laravel 10/11",
        is_installed: false,
    },
    PhpDownloadTarget {
        id: "8.1",
        version: "8.1.31",
        label: "PHP 8.1 (Legacy LTS)",
        url: "https://windows.php.net/downloads/releases/archives/php-8.1.31-Win32-vs16-x64.zip",
        note: "Legacy LTS build for Laravel 9 / older frameworks",
        is_installed: false,
    },
    PhpDownloadTarget {
        id: "7.4",
        version: "7.4.33",
        label: "PHP 7.4 (Legacy)",
        url: "https://windows.php.net/downloads/releases/archives/php-7.4.33-Win32-vc15-x64.zip",
        note: "Legacy archive build for legacy projects",
        is_installed: false,
    },
];

/// Returns the official download targets with `is_installed` populated based on current installed candidates.
pub fn get_available_download_targets(installed: &[PhpVersionCandidate]) -> Vec<PhpDownloadTarget> {
    OFFICIAL_DOWNLOAD_TARGETS
        .iter()
        .map(|target| {
            let mut item = target.clone();
            item.is_installed = installed.iter().any(|c| c.version == target.version);
            item
        })
        .collect()
}

/// Detect PHP version string from directory name or `php.exe -v`.
pub fn detect_php_version_string(php_dir: &Path) -> String {
    let dirname = php_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    // Check directory name first (e.g. php-8.3.36-Win32-vs16-x64, php-8.3.36, php8.3.30)
    for part in dirname.split(&['-', '_'][..]) {
        if let Some(rest) = part.strip_prefix("php") {
            if is_version_number(rest) {
                return rest.to_string();
            }
        }
        if is_version_number(part) {
            return part.to_string();
        }
    }

    // Try executing php.exe -v silently
    let exe = php_dir.join("php.exe");
    if exe.is_file() {
        if let Ok(output) = run_silent_cmd(&exe, &["-v"]) {
            if let Some(first_line) = output.lines().next() {
                // e.g. "PHP 8.3.30 (cli) (built: ...)"
                let words: Vec<&str> = first_line.split_whitespace().collect();
                if words.len() >= 2 && words[0].eq_ignore_ascii_case("PHP") {
                    return words[1].to_string();
                }
            }
        }
    }

    if !dirname.is_empty() && !dirname.eq_ignore_ascii_case("php") {
        dirname.to_string()
    } else {
        "Unknown".to_string()
    }
}

fn is_version_number(s: &str) -> bool {
    let dots = s.chars().filter(|&c| c == '.').count();
    dots >= 1 && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

fn check_security_advisory(version: &str) -> (bool, Option<String>) {
    // Check CVE-2026-109618 / CVE-2026-109619 security advisory
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse::<u32>().unwrap_or(0);
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        let patch = if parts.len() >= 3 {
            parts[2].parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        if major == 8 {
            if minor == 3 && patch < 36 {
                return (
                    true,
                    Some(
                        "Vulnerable to CVE-2026-109618/19 RCE. Upgrade to PHP 8.3.36 recommended."
                            .to_string(),
                    ),
                );
            }
            if minor == 2 && patch < 35 {
                return (
                    true,
                    Some(
                        "Vulnerable to CVE-2026-109618/19 RCE. Upgrade to PHP 8.2.35 recommended."
                            .to_string(),
                    ),
                );
            }
            if minor <= 1 {
                return (
                    true,
                    Some(format!(
                        "PHP 8.{minor} is End-Of-Life and vulnerable to CVE-2026-109618/19 RCE."
                    )),
                );
            }
        } else if major == 7 {
            return (
                true,
                Some(
                    "PHP 7.x is End-Of-Life and vulnerable to CVE-2026-109618/19 RCE.".to_string(),
                ),
            );
        }
    }

    (false, None)
}

/// Discover all installed PHP versions in Servisan and Laragon installations.
pub fn discover_installed_versions(paths: &Paths, config: &Config) -> Vec<PhpVersionCandidate> {
    let mut candidates = Vec::new();
    let active_dir = if let Some(ref vpath) = config.php.version_path {
        let p = PathBuf::from(vpath);
        let resolved = if p.is_absolute() {
            p
        } else {
            paths.root().join(p)
        };
        normalize_path(&resolved)
    } else {
        normalize_path(&paths.php_dir())
    };

    // 1. Scan internal Servisan `bin/php` directory
    let base_php = paths.bin_dir().join("php");
    if base_php.is_dir() {
        // Direct flat installation: bin/php/php.exe
        if base_php.join("php.exe").is_file() {
            let version = detect_php_version_string(&base_php);
            let (has_warn, warn_msg) = check_security_advisory(&version);
            let is_active = normalize_path(&base_php) == active_dir;

            candidates.push(PhpVersionCandidate {
                id: "servisan-bundled".to_string(),
                version: if version == "Unknown" {
                    "Bundled (bin/php)".to_string()
                } else {
                    version
                },
                path: base_php.clone(),
                source: "Servisan".to_string(),
                is_active,
                has_security_warning: has_warn,
                security_message: warn_msg,
            });
        }

        // Subdirectories under bin/php/
        if let Ok(entries) = std::fs::read_dir(&base_php) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join("php.exe").is_file() {
                    let dirname = p.file_name().unwrap().to_string_lossy().to_string();
                    let version = detect_php_version_string(&p);
                    let (has_warn, warn_msg) = check_security_advisory(&version);
                    let is_active = normalize_path(&p) == active_dir;

                    candidates.push(PhpVersionCandidate {
                        id: dirname,
                        version,
                        path: p,
                        source: "Servisan".to_string(),
                        is_active,
                        has_security_warning: has_warn,
                        security_message: warn_msg,
                    });
                }
            }
        }
    }

    // 2. Scan Laragon installations (D:\laragon, C:\laragon, and all detected drives)
    let drive_letters = crate::migrate::detect::available_drives();
    for drive in drive_letters {
        let laragon_php = PathBuf::from(format!(r"{}:\laragon\bin\php", drive));
        if laragon_php.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&laragon_php) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && p.join("php.exe").is_file() {
                        let dirname = p.file_name().unwrap().to_string_lossy().to_string();
                        let version = detect_php_version_string(&p);
                        let (has_warn, warn_msg) = check_security_advisory(&version);
                        let is_active = normalize_path(&p) == active_dir;

                        // Avoid duplicate paths
                        if !candidates
                            .iter()
                            .any(|c| normalize_path(&c.path) == normalize_path(&p))
                        {
                            candidates.push(PhpVersionCandidate {
                                id: format!("laragon-{dirname}"),
                                version,
                                path: p,
                                source: "Laragon".to_string(),
                                is_active,
                                has_security_warning: has_warn,
                                security_message: warn_msg,
                            });
                        }
                    }
                }
            }
        }
    }

    candidates
}

/// Switch active PHP version to the specified candidate ID, path, or directory name.
pub fn switch_php_version(
    paths: &Paths,
    config: &mut Config,
    target_id_or_path: &str,
) -> Result<PathBuf> {
    let installed = discover_installed_versions(paths, config);

    let target_path = if let Some(candidate) = installed.iter().find(|c| {
        c.id.eq_ignore_ascii_case(target_id_or_path)
            || c.version.eq_ignore_ascii_case(target_id_or_path)
            || c.path
                .to_string_lossy()
                .eq_ignore_ascii_case(target_id_or_path)
            || c.path
                .file_name()
                .map(|n| n.to_string_lossy().eq_ignore_ascii_case(target_id_or_path))
                .unwrap_or(false)
    }) {
        candidate.path.clone()
    } else {
        let direct = PathBuf::from(target_id_or_path);
        if direct.is_dir() && direct.join("php.exe").is_file() {
            direct
        } else {
            return Err(anyhow!(
                "PHP version '{}' not found. Run 'servisan php list' to view available versions.",
                target_id_or_path
            ));
        }
    };

    // Store in config
    let root = paths.root();
    let stored_path = if target_path.starts_with(root) {
        // Store relative path if inside Servisan root
        target_path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    } else {
        target_path.to_string_lossy().replace('\\', "/")
    };

    let version_str = detect_php_version_string(&target_path);
    config.php.version_path = Some(stored_path);
    config.php.version = format!("PHP {version_str}");
    config
        .save(&paths.config_file())
        .context("saving updated config/app.toml")?;

    // Create an updated Paths instance pointing to the newly selected php_dir
    let updated_paths = paths.clone().with_php_dir(&target_path);

    // Regenerate php.ini to point extension_dir to new PHP's ext/
    crate::ini::write_all(&updated_paths, config)
        .context("regenerating php.ini with new extension_dir")?;

    Ok(target_path)
}

/// Download and install an official PHP release from windows.php.net into `bin/php/php-<version>`.
pub fn download_and_install_php(paths: &Paths, target_key: &str) -> Result<PathBuf> {
    let target = OFFICIAL_DOWNLOAD_TARGETS
        .iter()
        .find(|t| {
            t.id.eq_ignore_ascii_case(target_key) || t.version.eq_ignore_ascii_case(target_key)
        })
        .ok_or_else(|| {
            anyhow!(
                "Unknown target '{}'. Available targets: {}",
                target_key,
                OFFICIAL_DOWNLOAD_TARGETS
                    .iter()
                    .map(|t| t.id)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let dest_dir = paths
        .bin_dir()
        .join("php")
        .join(format!("php-{}", target.version));
    if dest_dir.is_dir() && dest_dir.join("php.exe").is_file() {
        return Ok(dest_dir);
    }

    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("creating {}", dest_dir.display()))?;

    let tmp_zip = paths
        .tmp_dir()
        .join(format!("php-{}-download.zip", target.version));
    if let Some(parent) = tmp_zip.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let status_file = paths.tmp_dir().join("php-install.status");
    let _ = std::fs::write(&status_file, format!("downloading:{}:0", target.version));

    // 1. Download using native curl.exe with silent progress
    let curl_res = Command::new("curl.exe")
        .args(["-fSL", target.url, "-o"])
        .arg(&tmp_zip)
        .status()
        .context("running curl.exe to download PHP archive")?;

    if !curl_res.success() {
        let _ = std::fs::remove_file(&tmp_zip);
        let _ = std::fs::remove_dir_all(&dest_dir);
        let _ = std::fs::remove_file(&status_file);
        return Err(anyhow!(
            "Failed to download PHP {} from {} (curl exited with code {:?})",
            target.version,
            target.url,
            curl_res.code()
        ));
    }

    let _ = std::fs::write(&status_file, format!("extracting:{}", target.version));

    // 2. Extract using native tar.exe (bsdtar built into Windows)
    let tar_res = Command::new("tar.exe")
        .args(["-xf"])
        .arg(&tmp_zip)
        .args(["-C"])
        .arg(&dest_dir)
        .status()
        .context("running tar.exe to extract PHP archive")?;

    let _ = std::fs::remove_file(&tmp_zip);

    if !tar_res.success() {
        let _ = std::fs::remove_dir_all(&dest_dir);
        let _ = std::fs::remove_file(&status_file);
        return Err(anyhow!(
            "Failed to extract PHP {} (tar exited with code {:?})",
            target.version,
            tar_res.code()
        ));
    }

    let _ = std::fs::write(&status_file, format!("configuring:{}", target.version));

    // 3. Ensure Visual C++ runtime DLLs are copied into destination
    ensure_vc_dlls(&dest_dir);

    let _ = std::fs::write(&status_file, format!("done:{}", target.version));
    let _ = std::fs::remove_file(&status_file);

    Ok(dest_dir)
}

/// Copy Visual C++ runtime DLLs (vcruntime140.dll, etc.) from System32 into the PHP directory.
fn ensure_vc_dlls(dest_dir: &Path) {
    if let Ok(sys32) = std::env::var("SystemRoot") {
        let sys32_dir = PathBuf::from(sys32).join("System32");
        let dlls = [
            "vcruntime140.dll",
            "vcruntime140_1.dll",
            "msvcp140.dll",
            "msvcp140_1.dll",
            "msvcp140_2.dll",
            "msvcp140_atomic_wait.dll",
            "msvcp140_codecvt_ids.dll",
            "vcomp140.dll",
        ];
        for dll in dlls {
            let src = sys32_dir.join(dll);
            let dst = dest_dir.join(dll);
            if src.is_file() && !dst.is_file() {
                let _ = std::fs::copy(&src, &dst);
            }
        }
    }
}

/// Uninstall and delete an installed PHP runtime from `bin/php/<target>`.
///
/// Guarded against removing the currently active PHP runtime or directories outside `bin/php/*`.
pub fn uninstall_php_version(paths: &Paths, config: &Config, target_key: &str) -> Result<PathBuf> {
    let installed = discover_installed_versions(paths, config);
    let target_key_norm = target_key.trim();

    let candidate = installed
        .iter()
        .find(|c| {
            c.id.eq_ignore_ascii_case(target_key_norm)
                || c.version.eq_ignore_ascii_case(target_key_norm)
                || c.path
                    .to_string_lossy()
                    .eq_ignore_ascii_case(target_key_norm)
        })
        .ok_or_else(|| {
            anyhow!(
                "PHP version '{}' was not found in installed versions.",
                target_key
            )
        })?;

    if candidate.is_active || candidate.version.eq_ignore_ascii_case(&config.php.version) {
        return Err(anyhow!(
            "Cannot remove PHP {} because it is currently active. Switch to another version first.",
            candidate.version
        ));
    }

    let base_php = normalize_path(&paths.bin_dir().join("php"));
    let target_norm = normalize_path(&candidate.path);

    if !target_norm.starts_with(&base_php) {
        return Err(anyhow!(
            "Only Servisan-managed runtimes in bin/php can be removed. External installations are protected."
        ));
    }

    if target_norm == base_php {
        // Bundled flat runtime in bin/php: clean files directly in base_php and bundled folders (ext, lib)
        // while strictly preserving subdirectories (such as other installed PHP versions).
        if let Ok(entries) = std::fs::read_dir(&base_php) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = std::fs::remove_file(&path);
                } else if path.is_dir() {
                    let dirname = path.file_name().unwrap_or_default().to_string_lossy();
                    if dirname.eq_ignore_ascii_case("ext") || dirname.eq_ignore_ascii_case("lib") {
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }
        }
    } else if candidate.path.is_dir() {
        std::fs::remove_dir_all(&candidate.path)
            .with_context(|| format!("removing directory {}", candidate.path.display()))?;
    }

    Ok(candidate.path.clone())
}

fn normalize_path(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

#[cfg(windows)]
fn run_silent_cmd(exe: &Path, args: &[&str]) -> Result<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("spawning silent command")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(not(windows))]
fn run_silent_cmd(exe: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(exe).args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_strings() {
        assert_eq!(
            detect_php_version_string(Path::new(r"C:\php\php-8.3.36-Win32-vs16-x64")),
            "8.3.36"
        );
        assert_eq!(
            detect_php_version_string(Path::new(r"C:\php\php-8.2.35")),
            "8.2.35"
        );
        assert_eq!(
            detect_php_version_string(Path::new(r"C:\php\php7.4.33")),
            "7.4.33"
        );
    }

    #[test]
    fn identifies_security_advisories() {
        let (warn, msg) = check_security_advisory("8.3.30");
        assert!(warn);
        assert!(msg.unwrap().contains("CVE-2026-109618"));

        let (warn, _) = check_security_advisory("8.3.36");
        assert!(!warn);

        let (warn, msg) = check_security_advisory("8.2.25");
        assert!(warn);
        assert!(msg.unwrap().contains("8.2.35"));

        let (warn, _) = check_security_advisory("8.2.35");
        assert!(!warn);

        let (warn, _) = check_security_advisory("8.4.27");
        assert!(!warn);
    }

    #[test]
    fn official_targets_contain_required_versions() {
        assert!(OFFICIAL_DOWNLOAD_TARGETS
            .iter()
            .any(|t| t.id == "8.5" && t.version == "8.5.12"));
        assert!(OFFICIAL_DOWNLOAD_TARGETS
            .iter()
            .any(|t| t.id == "8.3" && t.version == "8.3.36"));
        assert!(OFFICIAL_DOWNLOAD_TARGETS
            .iter()
            .any(|t| t.id == "8.4" && t.version == "8.4.27"));
        assert!(OFFICIAL_DOWNLOAD_TARGETS
            .iter()
            .any(|t| t.id == "8.2" && t.version == "8.2.35"));
    }

    #[test]
    fn uninstall_rejects_active_php() {
        let root = std::env::temp_dir().join("servisan_uninstall_active_test");
        let _ = std::fs::remove_dir_all(&root);
        let paths = Paths::new(&root);
        let php_dir = paths.bin_dir().join("php").join("php-8.3.36");
        std::fs::create_dir_all(&php_dir).unwrap();
        std::fs::write(php_dir.join("php.exe"), b"dummy").unwrap();

        let mut config = Config::default();
        config.php.version_path = Some(php_dir.to_string_lossy().to_string());
        config.php.version = "8.3.36".to_string();

        let err = uninstall_php_version(&paths, &config, "8.3.36").unwrap_err();
        assert!(err.to_string().contains("currently active"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn uninstall_removes_inactive_php() {
        let root = std::env::temp_dir().join("servisan_uninstall_remove_test");
        let _ = std::fs::remove_dir_all(&root);
        let paths = Paths::new(&root);
        let php_dir = paths.bin_dir().join("php").join("php-8.2.35");
        std::fs::create_dir_all(&php_dir).unwrap();
        std::fs::write(php_dir.join("php.exe"), b"dummy").unwrap();

        let config = Config::default();

        assert!(php_dir.exists());
        let res = uninstall_php_version(&paths, &config, "8.2.35").unwrap();
        assert_eq!(res, php_dir);
        assert!(!php_dir.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn uninstall_removes_bundled_flat_php_preserving_subdirs() {
        let root = std::env::temp_dir().join("servisan_uninstall_bundled_test");
        let _ = std::fs::remove_dir_all(&root);
        let paths = Paths::new(&root);
        let base_php = paths.bin_dir().join("php");
        let sub_php = base_php.join("php-8.4.27");
        std::fs::create_dir_all(&sub_php).unwrap();
        std::fs::create_dir_all(base_php.join("ext")).unwrap();
        std::fs::write(base_php.join("php.exe"), b"dummy-bundled").unwrap();
        std::fs::write(base_php.join("php8ts.dll"), b"dummy-dll").unwrap();
        std::fs::write(base_php.join("ext").join("curl.dll"), b"dummy-ext").unwrap();
        std::fs::write(sub_php.join("php.exe"), b"dummy-sub").unwrap();

        let mut config = Config::default();
        config.php.version_path = Some(sub_php.to_string_lossy().to_string());
        config.php.version = "8.4.27".to_string();

        assert!(base_php.join("php.exe").exists());
        assert!(sub_php.join("php.exe").exists());

        let res = uninstall_php_version(&paths, &config, "servisan-bundled").unwrap();
        assert_eq!(res, base_php);
        assert!(!base_php.join("php.exe").exists());
        assert!(!base_php.join("php8ts.dll").exists());
        assert!(!base_php.join("ext").exists());
        assert!(sub_php.join("php.exe").exists());

        let servisan_candidates: Vec<_> = discover_installed_versions(&paths, &config)
            .into_iter()
            .filter(|c| c.source == "Servisan")
            .collect();
        assert_eq!(servisan_candidates.len(), 1);
        assert_eq!(servisan_candidates[0].id, "php-8.4.27");

        let _ = std::fs::remove_dir_all(&root);
    }
}
