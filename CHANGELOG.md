# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- **CSRF protection for dashboard actions**: the Rust manager injects a random `SERVISAN_DASHBOARD_TOKEN` on every start and the dashboard echoes it in the `X-Servisan-Token` header. Actions without the token are rejected, so other websites can no longer trigger local commands through `127.0.0.1`.
- **POST-only state changes**: opening the shell, Explorer folders, and database migration now require `POST`; `GET` requests are answered with `405`.
- **Loopback-only actions and `phpinfo()`**: requests from other machines cannot run dashboard actions, and the page shows a warning banner when it is opened from the network.
- **Command hardening**: user-supplied folder paths are normalized and validated, database names are checked against an allowlist (blocking CLI-flag injection such as `--overwrite`), and the terminal command is reduced to a single escaped executable call instead of string-concatenated `cmd.exe /k`.
- **No shell for migration commands**: `servisan migrate list` and `servisan migrate import` now run through `proc_open()` with an argument array, so database names and folders reach the CLI as literal argv entries and can never be parsed as shell syntax.
- **Empty migration selection rejected**: an empty database list can no longer fall through to "import every database".
- Added `Cache-Control: no-store`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` and a restrictive `Content-Security-Policy` to dashboard responses.
- Added `tests/dashboard_security.php`, `tests/dashboard_cli.php` plus Rust coverage for the token, loopback detection, the action guards and the shell-free command runner.

## [0.2.0] - 2026-10-20

### Added
- **Automated Database Migration Engine (`servisan migrate`)**:
  - Automatically scans Windows drive letters (`C:`, `D:`, etc.) detecting existing Laragon, XAMPP, and WAMP installations.
  - Resolves server binary (`mysqld.exe` / `mariadbd.exe`), dump tool (`mysqldump.exe`), and client (`mysql.exe`).
  - Offline filesystem inspection of user databases with formatted storage footprint estimates.
  - Piped zero-loss database streaming and import into Servisan MySQL with conflict options (`Skip` or `Overwrite`).
  - **Zero-Touch Autopilot Headless Spawner**: If source local server is stopped/offline, Servisan automatically spawns the source's `mysqld` on an ephemeral background port with `--skip-grant-tables --bind-address=127.0.0.1`, streams data silently, and cleans up the process upon completion without requiring user to start or touch Laragon/XAMPP.
  - **Custom Folder Scanner (`--from <path>`)**: Allows users to point to external hard drives or moved local server installations.
  - Added CLI subcommands `servisan migrate scan`, `servisan migrate list`, and `servisan migrate import [DATABASES...]` with `--json` support.
- **Interactive Database Migration Wizard in Web Dashboard**:
  - Top bar Menu dropdown "Database Migration" shortcut and system tray integration (`/?page=migrate`).
  - Auto-detected server cards, manual folder path input with live scanning, checklist database selection, and live execution console.
  - UI state guarding: prevents accidental modal dismissal during migration and replaces controls with a clean return button once finished.
- **Full SVG Icon Standardization & Strict Zero-Emoji UI**:
  - Standardized all items in the top-bar dropdown menu with 15x15 monochrome developer SVG icons (`PHP Info`, `Loaded Extensions`, `Database Migration`, `Open phpMyAdmin`, `Terminal Shell`, `Server Logs`, `Check for Updates...`, `GitHub Repository`).
  - Replaced all unicode emojis across dashboard alerts with clean inline monochrome SVGs.

### Fixed
- **PHP 8.1+ MySQL Offline Exception Handling**:
  - Caught unhandled `mysqli_sql_exception` when MySQL is stopped, ensuring dashboard cleanly renders `stopped` instead of a fatal PHP exception. (Thanks @mteguhfh in #1)
- **Dynamic Port & Version Manager Injection**:
  - Replaced hardcoded ports and version numbers with dynamic environment variables (`SERVISAN_MYSQL_PORT`, `SERVISAN_VERSION`) injected directly from `app.toml` and `Cargo.toml`. (Thanks @mteguhfh in #1)
- **Selectable Log Output**:
  - Restored text selection on server log console and modal body (`user-select: text`). (Thanks @mteguhfh in #1)
- **Offline Dashboard Regression Tests**:
  - Added automated Rust and PHP regression tests (`tests/dashboard_offline.php`) ensuring dashboard renders reliably when database is offline. (Thanks @mteguhfh in #1)

## [0.1.5] - 2026-10-16

### Added
- **Automatic Update Detection & Interactive Release Modal**:
  - Automatic background detection of new GitHub releases whenever the Web Dashboard is loaded online.
  - Interactive pulsating update badge on the navbar when a newer version is available.
  - "Check for Updates..." menu item in the dashboard dropdown with instant manual checks.
  - Interactive Update Modal displaying version comparison, release changelog notes, direct 1-click downloads for Windows Installer (`.exe`) and Portable Archive (`.zip`), and direct GitHub release links.

### Fixed
- **Single-Instance Enforcement & Duplicate Tray Prevention**:
  - Prevented multiple running instances of `servisan.exe` from spawning duplicate tray icons in the Windows notification area.
  - Implemented Windows named mutex (`Local\Servisan_Instance_<hash>`) scoped to the installation root in `src/platform/windows.rs`.
  - When a user opens Servisan while it is already running (e.g. clicking desktop/start shortcuts or re-running the executable), the second instance automatically brings up the Web Dashboard in the default browser and exits cleanly without spawning redundant processes or tray icons.

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