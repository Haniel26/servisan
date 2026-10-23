use anyhow::{Context, Result};
use std::path::Path;

pub const DEFAULT_HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
pub const START_MARKER: &str = "# --- Servisan Virtual Hosts Start ---";
pub const END_MARKER: &str = "# --- Servisan Virtual Hosts End ---";

/// Read the current contents of the hosts file.
pub fn read_hosts(hosts_path: &Path) -> Result<String> {
    std::fs::read_to_string(hosts_path)
        .with_context(|| format!("reading hosts file from {}", hosts_path.display()))
}

/// Generate new hosts file content replacing the Servisan section with the given domains.
pub fn render_hosts_content(existing_content: &str, domains: &[String]) -> String {
    let mut cleaned_lines = Vec::new();
    let mut in_servisan_block = false;

    for line in existing_content.lines() {
        let trimmed = line.trim();
        if trimmed == START_MARKER {
            in_servisan_block = true;
            continue;
        }
        if trimmed == END_MARKER {
            in_servisan_block = false;
            continue;
        }
        if !in_servisan_block {
            cleaned_lines.push(line);
        }
    }

    // Trim trailing empty lines
    while let Some(last) = cleaned_lines.last() {
        if last.trim().is_empty() {
            cleaned_lines.pop();
        } else {
            break;
        }
    }

    let mut result = cleaned_lines.join("\r\n");

    if !domains.is_empty() {
        if !result.is_empty() {
            result.push_str("\r\n\r\n");
        }
        result.push_str(START_MARKER);
        result.push_str("\r\n");
        for domain in domains {
            let d = domain.trim();
            if !d.is_empty() {
                result.push_str(&format!("127.0.0.1  {}\r\n", d));
            }
        }
        result.push_str(END_MARKER);
        result.push_str("\r\n");
    } else if !result.is_empty() {
        result.push_str("\r\n");
    }

    result
}

/// Update the Windows hosts file with the given list of domains.
///
/// Returns `Ok(true)` if written directly, `Ok(true)` if written via UAC elevation,
/// or `Ok(false)` if the user rejected elevation or write permission was denied.
pub fn sync_hosts_file(hosts_path: &Path, domains: &[String]) -> Result<bool> {
    let existing = read_hosts(hosts_path).unwrap_or_default();

    let new_content = render_hosts_content(&existing, domains);

    // Try direct write first (succeeds if running as Administrator or writeable)
    if std::fs::write(hosts_path, &new_content).is_ok() {
        return Ok(true);
    }

    // If direct write fails (PermissionDenied), attempt elevated write via ShellExecuteW runas
    #[cfg(windows)]
    {
        let temp_file = std::env::temp_dir().join("servisan_hosts_sync.tmp");
        if std::fs::write(&temp_file, &new_content).is_ok() {
            let script = format!(
                "Set-Content -LiteralPath '{}' -Value (Get-Content -LiteralPath '{}' -Raw) -Encoding ASCII; Remove-Item -LiteralPath '{}' -Force",
                hosts_path.to_string_lossy(),
                temp_file.to_string_lossy(),
                temp_file.to_string_lossy()
            );

            let params = format!("-NoProfile -Command \"{}\"", script.replace('"', "`\""));
            if crate::platform::shell_execute_runas("powershell.exe", &params).is_ok() {
                // Poll briefly (up to 3 seconds) for the elevated PowerShell process to write
                for _ in 0..15 {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    if let Ok(updated) = read_hosts(hosts_path) {
                        if updated.contains(START_MARKER) || domains.is_empty() {
                            let _ = std::fs::remove_file(&temp_file);
                            return Ok(true);
                        }
                    }
                }
            }

            let _ = std::fs::remove_file(&temp_file);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_domains_cleans_block() {
        let original = "127.0.0.1 localhost\r\n# --- Servisan Virtual Hosts Start ---\r\n127.0.0.1 blog.test\r\n# --- Servisan Virtual Hosts End ---\r\n";
        let rendered = render_hosts_content(original, &[]);
        assert!(!rendered.contains("blog.test"));
        assert!(!rendered.contains(START_MARKER));
        assert!(rendered.contains("127.0.0.1 localhost"));
    }

    #[test]
    fn renders_new_domains() {
        let original = "127.0.0.1 localhost\r\n";
        let domains = vec!["app.test".to_string(), "shop.test".to_string()];
        let rendered = render_hosts_content(original, &domains);
        assert!(rendered.contains(START_MARKER));
        assert!(rendered.contains("127.0.0.1  app.test"));
        assert!(rendered.contains("127.0.0.1  shop.test"));
        assert!(rendered.contains(END_MARKER));
    }
}
