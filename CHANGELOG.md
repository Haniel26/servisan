# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2026-10-16

### Fixed
- **Database Initialization Conflict with OPcache (`data` directory not empty)**:
  - Fixed `ERROR : Data directory ... is not empty. Only new or empty existing directories are accepted for --datadir` on fresh setups.
  - Relocated PHP OPcache disk cache (`opcache.file_cache`) from `data/opcache` to `tmp/opcache` (`tmp_dir()`), ensuring MySQL `datadir` remains 100% reserved for database tables and files.
  - Added recursive cleanup of uninitialized `data/` directory before running bootstrap, guaranteeing a clean slate even after previous failed initialization attempts.

## [0.1.3] - 2026-10-16

### Fixed
- **MariaDB Portable Initialization (`mariadb-install-db.exe` `--basedir` bug fix)**:
  - Fixed database initialization failure `(exit code: 7): unknown variable 'basedir=...'` when initializing MariaDB/MySQL on non-standard drives or clean portable paths.
  - Windows `mariadb-install-db.exe` automatically resolves `basedir` relative to its binary folder (`bin\..`) and rejects `--basedir` as an unrecognized option.
  - Added hidden window creation flag (`CREATE_NO_WINDOW`) to database initialization processes to prevent console flashing during initial startup.

## [0.1.2] - 2026-10-16

### Fixed
- **App-Local Visual C++ Runtime Bundling (`vcruntime140.dll` fix)**:
  - Fixed `vcruntime140.dll was not found` error on clean/fresh Windows machines without Microsoft Visual C++ Redistributable installed.
  - Automatically bundles Microsoft C/C++ runtime DLLs (`vcruntime140.dll`, `vcruntime140_1.dll`, `msvcp140.dll`, `msvcp140_1.dll`, `msvcp140_2.dll`, `msvcp140_atomic_wait.dll`, `msvcp140_codecvt_ids.dll`, `vcomp140.dll`) app-locally into `bin/php/`, `bin/mysql/bin/`, and the package root in both portable `.zip` and setup `.exe` packages.
  - Guarantees 100% zero-dependency execution out-of-the-box on any Windows 10/11 system.

## [0.1.1] - 2026-10-16

### Added
- **Laragon-Style Web Dashboard (`www/index.php`)**:
  - Full redesign replicating Laragon's clean desktop aesthetic in the browser.
  - Live PHP and MySQL/MariaDB service status indicators and ports.
  - Interactive top-right Menu dropdown with phpinfo, loaded extensions, phpMyAdmin, and documentation.
  - 4 native bottom docked buttons: Logs console modal (50 lines tail, PHP/MySQL/Servisan tabs), Database (phpMyAdmin new tab), Terminal (launches `servisan shell` in Command Prompt), and Root (opens `www/` in File Explorer).
  - Clean monochrome developer UI with zero emojis and external footer credit.
- **System Tray "Open Dashboard" Shortcut**:
  - Added "Open Dashboard" to system tray menu with helper methods `web_url()` and `open_dashboard()`.
- **Windows Setup Installer (`servisan-setup.exe`)**:
  - Added Inno Setup script (`scripts/installer.iss`) and build script (`scripts/build-installer.ps1`).
  - Solid LZMA2 ultra compression (47.5 MB installer containing full 376 MB portable runtimes).
  - Zero-UAC installation to `C:\Servisan`, desktop shortcut options, and auto-start on boot task.
- **CI/CD Workflow Upgrades**:
  - Automated release workflow in `.github/workflows/release.yml` publishing both portable `.zip` and setup `.exe` installers.

## [0.1.0] - 2026-10-16

### Initial Public Release

#### Added
- **Core Process Engine & Safe Cleanup:**
  - Portable PHP built-in web server (`php -S`) with concurrent worker support via `PHP_CLI_SERVER_WORKERS`.
  - OPcache acceleration enabled by default with immediate timestamp validation (`opcache.validate_timestamps=1`, `opcache.revalidate_freq=0`).
  - Native MySQL / MariaDB server (`mysqld`) lifecycle management with graceful shutdown via `mysqladmin shutdown` prior to process termination.
  - Native Windows Job Object integration (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) ensuring zero lingering orphan/zombie processes upon exit or crash.
- **Silent System Tray & Developer CLI:**
  - Lightweight Windows System Tray UI using `tray-icon` and `tao` without desktop window clutter or console popups.
  - Dynamic status indicators (🟢 running, ⚪ stopped) and right-click context menu (Start All, Stop All, Open www, Config, Logs, phpMyAdmin, Exit).
  - CLI commands: `tray`, `start`, `stop`, `status`, `init`, `shell`, and `phpmyadmin`.
  - Built-in developer shell (`servisan shell`) injecting portable `php`, `mysql`, and `mysqladmin` into runtime `PATH`.
- **Zero-Dependency Dual-Mode Installer:**
  - `scripts/install-binaries.ps1` with smart local Laragon auto-detection across `C:\laragon` and `D:\laragon`.
  - Automated fallback downloading clean standalone portable runtimes (PHP 8.3 Thread-Safe and MariaDB 11.4 LTS) for users without Laragon/XAMPP.
- **phpMyAdmin Integration:**
  - One-command phpMyAdmin provisioning (`servisan phpmyadmin install`) with automatic `config.inc.php` generation and CSPRNG blowfish secret creation.
- **Packaging & Portability:**
  - `scripts/package-portable.ps1` producing self-contained, offline-ready `.zip` release archives with SHA-256 verification.
  - Conflict-free default ports (PHP `8080`, MySQL `3307`) avoiding clashes with existing Laragon, Apache, or IIS services.