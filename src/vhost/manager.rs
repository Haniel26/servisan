use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::paths::Paths;
use crate::vhost::hosts::{sync_hosts_file, DEFAULT_HOSTS_PATH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualHost {
    pub name: String,
    pub domain: String,
    pub target_path: PathBuf,
    pub doc_root: PathBuf,
    pub framework: String,
    pub link_path: PathBuf,
    pub hosts_synced: bool,
    pub created_at: String,
}

/// Detect the document root and framework type from the given target directory.
pub fn detect_doc_root(target: &Path) -> (PathBuf, String) {
    let public_dir = target.join("public");
    if public_dir.is_dir() {
        let has_index = public_dir.join("index.php").is_file();
        let is_laravel = target.join("artisan").is_file();
        let label = if is_laravel {
            "Laravel (public/)"
        } else if has_index {
            "Modern Framework (public/)"
        } else {
            "Public Directory (public/)"
        };
        (public_dir, label.to_string())
    } else {
        let has_wp = target.join("wp-config.php").is_file() || target.join("wp-load.php").is_file();
        let label = if has_wp {
            "WordPress (root)"
        } else {
            "Standard PHP (root)"
        };
        (target.to_path_buf(), label.to_string())
    }
}

/// Slugify folder name into a valid vhost name and suggested `.test` domain.
pub fn suggest_name_and_domain(target: &Path) -> (String, String) {
    let raw_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let mut slug = String::new();
    let mut last_dash = false;
    for c in raw_name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if (c == '-' || c == '_' || c == ' ') && !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    let name = if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    };
    let domain = format!("{}.test", name);
    (name, domain)
}

/// Strip Windows extended verbatim prefix (`\\?\`) from paths.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

/// Path to `config/vhosts.json`.
pub fn vhosts_config_path(paths: &Paths) -> PathBuf {
    paths.config_dir().join("vhosts.json")
}

/// Load list of registered virtual hosts from `config/vhosts.json`.
pub fn load_vhosts(paths: &Paths) -> Vec<VirtualHost> {
    let p = vhosts_config_path(paths);
    if !p.is_file() {
        return Vec::new();
    }
    match std::fs::read_to_string(&p) {
        Ok(text) => {
            let mut list: Vec<VirtualHost> = serde_json::from_str(&text).unwrap_or_default();
            for item in &mut list {
                item.target_path = strip_verbatim_prefix(&item.target_path);
                item.doc_root = strip_verbatim_prefix(&item.doc_root);
                item.link_path = strip_verbatim_prefix(&item.link_path);
            }
            list
        }
        Err(_) => Vec::new(),
    }
}

/// Save list of registered virtual hosts to `config/vhosts.json`.
pub fn save_vhosts(paths: &Paths, vhosts: &[VirtualHost]) -> Result<()> {
    let p = vhosts_config_path(paths);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(vhosts)?;
    std::fs::write(&p, json).with_context(|| format!("writing {}", p.display()))?;
    Ok(())
}

/// Add an external project directory as a Servisan Virtual Host.
///
/// 1. Auto-detects `public/` (Laravel/Modern) or root document root.
/// 2. Creates a Windows Directory Junction at `www/<name>`.
/// 3. Registers the domain in Windows `hosts` file (`C:\Windows\System32\drivers\etc\hosts`).
/// 4. Stores configuration in `config/vhosts.json`.
pub fn add_virtual_host(
    paths: &Paths,
    target_path: &Path,
    custom_name: Option<&str>,
    custom_domain: Option<&str>,
) -> Result<VirtualHost> {
    if !target_path.is_dir() {
        return Err(anyhow!(
            "Target project path does not exist or is not a directory: {}",
            target_path.display()
        ));
    }

    let raw_target = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path.to_path_buf());
    let abs_target = strip_verbatim_prefix(&raw_target);

    let (suggested_name, suggested_domain) = suggest_name_and_domain(&abs_target);
    let name = custom_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(suggested_name);
    let domain = custom_domain
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(suggested_domain);

    let (raw_doc_root, framework) = detect_doc_root(&abs_target);
    let doc_root = strip_verbatim_prefix(&raw_doc_root);
    let link_path = strip_verbatim_prefix(&paths.www_dir().join(&name));

    let mut vhosts = load_vhosts(paths);

    // If an existing entry matches the name, remove old junction link first
    if let Some(pos) = vhosts
        .iter()
        .position(|v| v.name.eq_ignore_ascii_case(&name))
    {
        let old = &vhosts[pos];
        let _ = remove_junction(&old.link_path);
        vhosts.remove(pos);
    }

    // Create Windows Directory Junction at www/<name>
    create_junction(&link_path, &doc_root)?;

    // Sync hosts file
    let mut all_domains: Vec<String> = vhosts.iter().map(|v| v.domain.clone()).collect();
    if !all_domains.contains(&domain) {
        all_domains.push(domain.clone());
    }
    let hosts_synced =
        sync_hosts_file(Path::new(DEFAULT_HOSTS_PATH), &all_domains).unwrap_or(false);

    let vhost = VirtualHost {
        name,
        domain,
        target_path: abs_target,
        doc_root,
        framework,
        link_path,
        hosts_synced,
        created_at: current_timestamp(),
    };

    vhosts.push(vhost.clone());
    save_vhosts(paths, &vhosts)?;

    Ok(vhost)
}

/// Remove a virtual host by name or domain.
pub fn remove_virtual_host(paths: &Paths, target_key: &str) -> Result<VirtualHost> {
    let key = target_key.trim();
    let mut vhosts = load_vhosts(paths);

    let pos = vhosts
        .iter()
        .position(|v| v.name.eq_ignore_ascii_case(key) || v.domain.eq_ignore_ascii_case(key))
        .ok_or_else(|| anyhow!("Virtual host '{}' was not found.", key))?;

    let removed = vhosts.remove(pos);

    // Remove Directory Junction from www/
    let _ = remove_junction(&removed.link_path);

    // Sync hosts file without this domain
    let remaining_domains: Vec<String> = vhosts.iter().map(|v| v.domain.clone()).collect();
    let _ = sync_hosts_file(Path::new(DEFAULT_HOSTS_PATH), &remaining_domains);

    save_vhosts(paths, &vhosts)?;

    Ok(removed)
}

/// Create a Windows Directory Junction pointing `link` to `target`.
pub fn create_junction(link: &Path, target: &Path) -> Result<()> {
    if link.exists() {
        let _ = remove_junction(link);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let status = std::process::Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()
            .with_context(|| {
                format!("running mklink /J {} {}", link.display(), target.display())
            })?;

        if !status.success() {
            return Err(anyhow!(
                "Failed to create junction link (cmd exited with code {:?})",
                status.code()
            ));
        }
    }

    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link)?;
    }

    Ok(())
}

/// Remove a Windows Directory Junction or link.
pub fn remove_junction(link: &Path) -> Result<()> {
    if !link.exists() {
        return Ok(());
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let status = std::process::Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/C", "rmdir"])
            .arg(link)
            .status()
            .with_context(|| format!("running rmdir {}", link.display()))?;

        if !status.success() {
            // Fallback to std::fs::remove_dir
            let _ = std::fs::remove_dir(link);
        }
    }

    #[cfg(not(windows))]
    {
        let _ = std::fs::remove_file(link);
    }

    Ok(())
}

fn current_timestamp() -> String {
    // Basic human-readable timestamp
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_laravel_public_dir() {
        let root = std::env::temp_dir().join("servisan_vhost_laravel_test");
        let _ = std::fs::remove_dir_all(&root);
        let public_dir = root.join("public");
        std::fs::create_dir_all(&public_dir).unwrap();
        std::fs::write(root.join("artisan"), b"dummy").unwrap();

        let (doc_root, framework) = detect_doc_root(&root);
        assert_eq!(doc_root, public_dir);
        assert!(framework.contains("Laravel"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn detects_standard_php_root() {
        let root = std::env::temp_dir().join("servisan_vhost_standard_test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("index.php"), b"dummy").unwrap();

        let (doc_root, framework) = detect_doc_root(&root);
        assert_eq!(doc_root, root);
        assert!(framework.contains("Standard"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn suggests_clean_slug_and_domain() {
        let p = Path::new(r"D:\Projects\My Cool-App_2026");
        let (name, domain) = suggest_name_and_domain(p);
        assert_eq!(name, "my-cool-app-2026");
        assert_eq!(domain, "my-cool-app-2026.test");
    }

    #[test]
    fn strips_windows_verbatim_prefix() {
        let p = Path::new(r"\\?\D:\Project\PenaReview");
        let stripped = strip_verbatim_prefix(p);
        assert_eq!(stripped, PathBuf::from(r"D:\Project\PenaReview"));

        let normal = Path::new(r"D:\Project\PenaReview");
        assert_eq!(
            strip_verbatim_prefix(normal),
            PathBuf::from(r"D:\Project\PenaReview")
        );
    }
}
