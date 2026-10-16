# Servisan

Servisan is a lightweight, portable Windows manager for local PHP and MySQL/MariaDB development.

It is intentionally small:

- starts PHP through its built-in web server (`php -S`)
- runs MySQL/MariaDB from its configured data directory
- keeps all runtime state under the Servisan folder
- stores MySQL on the loopback network only
- cleans child processes on normal exit, Ctrl+C and process crash, through a Windows Job Object
- exposes a tray icon and a small CLI
- optionally installs phpMyAdmin from Laragon

## Status

Windows-only MVP is functional. Verified in this repository:

- build: `cargo test`, `cargo clippy`
- tray starts and stops PHP and MySQL
- `START` returns HTTP 200 on `http://127.0.0.1:8080/`
- phpMyAdmin renders after installation
- no child process leaks on stop or crash cleanup

## Quick Start from Source

1. Install Rust and an existing Laragon (or equivalent) tree at `D:\laragon`.
2. Copy the portable runtimes without entering the repository index:

   ```powershell
   .\scripts\install-binaries.ps1
   ```

   This copies `php-8.3.30-Win32-vs16-x64` and `mysql-8.4.3-winx64`; change versions with `-PhpVersion` and `-MysqlVersion`.
3. Initialize Servisan:

   ```powershell
   cargo run -- init
   ```

4. Test the CLI flow:

   ```powershell
   cargo run -- start
   ```

   Then visit `http://127.0.0.1:8080/`. Press `Ctrl+C` to stop.
5. Build the release binary:

   ```powershell
   .\scripts\build-release.ps1
   .\servisan.exe status
   ```
6. Start the tray app:

   ```powershell
   .\servisan.exe
   ```

By default all services start automatically when the tray app starts. Open the hidden notification-area icon if needed.

## CLI

```text
servisan [COMMAND]
```

| Command | Description |
|---|---|
| `tray`, or no command | Run the tray application |
| `init` | Create folders, write `config/app.toml`, render `my.ini` / `php.ini`, initialize MySQL |
| `start` | Start PHP + MySQL and block until `Ctrl+C` |
| `stop` | Stop Servisan-owned services |
| `status` | Show PHP and MySQL state |
| `shell` | Open a terminal with Servisan's `php`, `mysql` and `mysqladmin` on `PATH` |
| `phpmyadmin install [--source PATH] [--force]` | Copy or refresh phpMyAdmin |
| `phpmyadmin status` | Show whether phpMyAdmin is installed |
| `phpmyadmin open` | Open phpMyAdmin in the browser |

## Default Configuration

`config/app.toml` uses:

- PHP: `127.0.0.1:8080`
- MySQL: `127.0.0.1:3307`
- auto-start: `true`
- auto open browser after start: `true`
- MySQL root, TCP loopback only, `AllowNoPassword`

Ports avoid a default Laragon installation on `80` / `3306`.

## phpMyAdmin

phpMyAdmin is optional. It is reused from the Laragon tree:

```powershell
.\servisan.exe phpmyadmin install
```

The manager refreshes `www/phpmyadmin/config.inc.php` on startup so host and port point at the Servisan MySQL instance. The blowfish secret is generated under `config/generated/phpmyadmin.secret` (gitignored).

## Portable Full-Package Release

This repository is source-oriented; runtime binaries and `www/*` are gitignored. A full portable package can be built locally by running `scripts/package-portable.ps1`. It stages:

- `servisan.exe`
- `bin/php`, `bin/mysql`
- `assets`, `config`, `www`
- empty `data` and `logs`
- README and LICENSE

The resulting zip is suitable as a GitHub Release asset, but should not be committed to the repository.

## Security Notes

This is a development server only.

- MySQL binds `127.0.0.1`
- Port range defaults avoid common Laragon ports
- Root has no password, which is acceptable only for local work
- `config/generated/`, logs, data, and user projects are ignored by Git

## Known Limits

- Windows-only
- No virtual hosts or `*.test` domains yet
- No Run-at-logon integration beyond tray autostart
- phpMyAdmin storage tables are not pre-created; basic browsing works
- `EngineKind::Nginx` and `EngineKind::Apache` are reserved, but only the builtin PHP server is implemented

## License

MIT. See [LICENSE](LICENSE).

Reusing Laragon-provided runtime binaries is the responsibility of anyone redistributing them. Keep the appropriate PHP and MySQL/MariaDB license files and notices.