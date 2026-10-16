# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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