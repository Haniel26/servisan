pub mod switcher;

pub use switcher::{
    discover_installed_versions, download_and_install_php, get_available_download_targets,
    switch_php_version, uninstall_php_version, PhpDownloadTarget, PhpVersionCandidate,
};
