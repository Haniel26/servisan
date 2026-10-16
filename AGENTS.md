# AGENTS.md

Project memory for opencode and maintainers working in `D:\Project\ServiSan`.

## Purpose

Servisan is a Windows-only portable PHP + MySQL/MariaDB local server manager.
It prefers the Laragon binary reuse strategy, runs PHP with `php -S`, controls MySQL
via `mysqld`, and can optionally install phpMyAdmin.

## Canonical Commands

Build and test:

```powershell
cargo build
cargo test
cargo test -- --ignored
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release
```

Package an offline portable release:

```powershell
.\scripts\build-release.ps1
.\scripts\package-portable.ps1
```

Run from source:

```powershell
cargo run -- status
cargo run -- init
cargo run -- start
cargo run -- phpmyadmin install
```

Product binary entry points:

- `servisan.exe` or `servisan tray`: tray app
- `servisan start`: foreground server
- `servisan stop`: stop managed services
- `servisan status`: inspect services
- `servisan shell`: shell with `php`, `mysql`, and `mysqladmin` on PATH

## Project Notes

- `bin/`, `data/`, `logs/`, `www/*`, `target/`, and `config/generated/` are gitignored.
- Default resources are governor by `config/app.toml`.
- Default ports are PHP 8080 and MySQL 3307 to avoid conflict with Laragon.
- Tray app auto-starts and auto-opens the browser after successful service startup.
- PHP/MySQL process windows stay hidden using `CREATE_NO_WINDOW`.
- Child cleanup uses Windows Job Object with `KILL_ON_JOB_CLOSE`.
- PHP and MySQL binaries remain separate native processes; they are not compiled into Rust.
- phpMyAdmin installation copies Laragon's app to `www/phpmyadmin`; generated `config.inc.php` refreshes on startup.
- Branded assets are copied under `www/` during `init` and `start`.
- Todo/task tracking is session-only; commit history is the durable record.

## Key Files

- `src/app.rs`: app lifecycle and service orchestration
- `src/tray.rs`: tray icon, context menu, status indicators
- `src/process.rs`: child process spawning and cleanup
- `src/config.rs`: configuration schema
- `src/paths.rs`: portable root discovery and path helpers
- `src/main.rs`: CLI commands
- `src/phpmyadmin.rs`: optional phpMyAdmin install and configuration
- `src/services/php.rs`, `src/services/mysql.rs`: service lifecycle
- `scripts/build-release.ps1`, `scripts/package-portable.ps1`: release packaging

## Release Repository Policy

Keep repository source-focused. Do not commit runtime binaries, generated MySQL data, logs, user projects, or extracted phpMyAdmin files. Release artifacts belong in GitHub Releases.

## Recent Updates

- **Automated GitHub Actions Release Workflow**: Added `.github/workflows/release.yml` triggered on version tag pushes (`v*`). The workflow compiles the release binary on `windows-latest`, stages official portable runtimes and phpMyAdmin, builds the portable `.zip` package with SHA-256 checksums, and automatically publishes the release assets.
- **Changelog Standardization & Version Tagging**: Restructured `CHANGELOG.md` following the Keep a Changelog standard for initial public release `[0.1.0]`. Added release badge and navigation link to `README.md`.
- **Standalone Runtime Auto-Download & Dual-Mode Installer**: Upgraded `scripts/install-binaries.ps1` to support both Laragon binary reuse (auto-detecting `C:\laragon` and `D:\laragon`) and automatic fallback to downloading official portable runtimes (PHP 8.3 Thread-Safe from windows.php.net and MariaDB 11.4 LTS portable zip from official archives). Updated `src/phpmyadmin.rs` to automatically discover Laragon across multiple drive letters. Clarified standalone 100% dependency-free portable package flow in `README.md`.
- **PHP Concurrency & OPcache Optimization**: Added configurable `workers` option to `[php]` section in `config/app.toml` (defaulting to 4). Propagated `PHP_CLI_SERVER_WORKERS` into `BuiltinServer` spawner to allow concurrent static asset/AJAX handling on Windows. Enabled OPcache in generated `php.ini` (`opcache.enable=1`, `opcache.enable_cli=1`, `opcache.validate_timestamps=1`, `opcache.revalidate_freq=0`) for instant execution without sacrificing real-time code reload.
- **Documentation & Showcase Preparation (README.md)**: Revamped `README.md` with official brand logo, badges, value proposition, competitive comparison table (Servisan vs Laragon vs XAMPP vs Docker), architecture Mermaid diagram, detailed CLI reference, configuration guide, explicit clarification of silent system tray behavior (no window/no console popup), and future roadmap for GitHub publication (including Laragon database migration tooling).