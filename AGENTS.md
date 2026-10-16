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