use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

use servisan::app::{App, ServiceKind};
use servisan::signal;

#[derive(Parser)]
#[command(
    name = "servisan",
    version,
    about = "Lightweight, portable local PHP + MySQL server manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the system-tray application (default when no command is given).
    Tray,
    /// Initialize config, generated ini files and the MySQL data directory.
    Init,
    /// Start services and run until Ctrl+C.
    Start {
        /// Start only one service.
        #[arg(long, value_enum)]
        only: Option<ServiceArg>,
    },
    /// Stop services started by another Servisan process.
    Stop,
    /// Show the status of PHP and MySQL.
    Status,
    /// Open a shell with `php`, `mysql` and `mysqladmin` on PATH.
    Shell {
        /// Shell program to launch. Defaults to %ComSpec% (cmd.exe).
        #[arg(long)]
        program: Option<String>,
    },
    /// Manage the optional bundled phpMyAdmin.
    #[command(name = "phpmyadmin")]
    PhpMyAdmin {
        #[command(subcommand)]
        command: PhpMyAdminCommand,
    },
    /// Manage PHP versions and switch active runtime.
    #[command(name = "php")]
    Php {
        #[command(subcommand)]
        command: PhpCommand,
    },
    /// Migrate databases from Laragon, XAMPP, WAMP, or other MySQL installations.
    #[command(name = "migrate")]
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    /// Manage Virtual Hosts and linked external projects.
    #[command(name = "vhost")]
    Vhost {
        #[command(subcommand)]
        command: VhostCommand,
    },
}

#[derive(Subcommand)]
enum PhpMyAdminCommand {
    /// Copy Laragon's phpMyAdmin into Servisan's www folder and generate config.
    Install {
        /// Override the Laragon source folder.
        #[arg(long)]
        source: Option<String>,
        /// Replace an existing phpMyAdmin copy.
        #[arg(long)]
        force: bool,
    },
    /// Show whether phpMyAdmin is installed and its URL.
    Status,
    /// Open phpMyAdmin in the browser.
    Open,
}

#[derive(Subcommand)]
enum PhpCommand {
    /// List installed PHP versions and available official downloads.
    List {
        /// Output results in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Switch active PHP version to a candidate ID, version, or folder.
    Switch {
        /// Version ID (e.g. 'php-8.3.36'), version string ('8.3.36'), or full directory path.
        target: String,
    },
    /// Download and install an official PHP runtime (e.g. 8.4, 8.3, 8.2, 8.1, 7.4).
    Install {
        /// Target version key (e.g. '8.4', '8.3', '8.2', '8.1', '7.4').
        version: String,
        /// Automatically switch to the newly installed version.
        #[arg(long)]
        switch: bool,
    },
    /// Remove an installed PHP version from bin/php/ (cannot remove the active version).
    #[command(alias = "remove")]
    Uninstall {
        /// Version ID (e.g. 'php-8.3.36') or version string ('8.3.36') to remove.
        target: String,
    },
}

#[derive(Subcommand)]
enum MigrateCommand {
    /// Scan for local server installations (Laragon, XAMPP, WAMP) or a specific directory.
    Scan {
        /// Specific folder to scan (skip auto-detection).
        #[arg(long)]
        from: Option<String>,

        /// Source MySQL port (only used with --from for a custom source).
        #[arg(long, default_value = "3306")]
        port: u16,

        /// Output scan results in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// List databases found in detected installations or a specific directory.
    List {
        /// Specific folder to list databases from (skip auto-detection).
        #[arg(long)]
        from: Option<String>,

        /// Source MySQL port (only used with --from for a custom source).
        #[arg(long, default_value = "3306")]
        port: u16,

        /// Output discovered databases in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Import databases from detected installations into Servisan's MySQL.
    Import {
        /// Specific database names to import. Imports all if omitted.
        databases: Vec<String>,

        /// Import from a specific data directory path (skip auto-detection).
        #[arg(long)]
        from: Option<String>,

        /// Source MySQL port (only used with --from for a custom source).
        #[arg(long, default_value = "3306")]
        port: u16,

        /// Skip databases that already exist in Servisan (default behavior).
        #[arg(long)]
        skip_existing: bool,

        /// Overwrite databases that already exist in Servisan.
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
enum VhostCommand {
    /// List all registered virtual hosts and linked projects.
    List {
        /// Output discovered virtual hosts in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Link an external project directory as a virtual host.
    Add {
        /// Path to the project directory.
        path: String,

        /// Custom project name / subfolder slug (default: auto-detected from directory name).
        #[arg(long)]
        name: Option<String>,

        /// Custom local domain (default: <name>.test).
        #[arg(long)]
        domain: Option<String>,
    },
    /// Remove a virtual host by name or domain.
    #[command(alias = "remove")]
    Delete {
        /// Name or domain of the virtual host to remove.
        target: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ServiceArg {
    Php,
    Mysql,
}

impl From<ServiceArg> for ServiceKind {
    fn from(value: ServiceArg) -> Self {
        match value {
            ServiceArg::Php => ServiceKind::Php,
            ServiceArg::Mysql => ServiceKind::Mysql,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut app = App::load()?;

    match cli.command {
        None | Some(Command::Tray) => servisan::tray::run(app),
        Some(Command::Init) => app.init(),
        Some(Command::Status) => app.status(),
        Some(Command::Stop) => app.stop_external(),
        Some(Command::Shell { program }) => app.shell(program.as_deref()),
        Some(Command::PhpMyAdmin { command }) => match command {
            PhpMyAdminCommand::Install { source, force } => {
                app.install_phpmyadmin(source.map(PathBuf::from), force)
            }
            PhpMyAdminCommand::Status => {
                app.status()?;
                if let Some(url) = app.phpmyadmin_url() {
                    println!("  phpMyAdmin URL : {url}");
                }
                Ok(())
            }
            PhpMyAdminCommand::Open => app.open_phpmyadmin(),
        },
        Some(Command::Php { command }) => run_php(&mut app, command),
        Some(Command::Migrate { command }) => run_migrate(&app, command),
        Some(Command::Vhost { command }) => run_vhost(&app, command),
        Some(Command::Start { only }) => run_foreground(&app, only.map(ServiceKind::from)),
    }
}

/// Handle `servisan php` subcommands.
fn run_php(app: &mut App, command: PhpCommand) -> Result<()> {
    match command {
        PhpCommand::List { json } => {
            let installed = app.list_php_versions();
            let available = servisan::php::get_available_download_targets(&installed);

            if json {
                let out = serde_json::json!({
                    "installed": installed,
                    "available": available,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }

            println!("=== Installed PHP Versions ===\n");
            if installed.is_empty() {
                println!("  No PHP versions found. Run scripts/install-binaries.ps1 or 'servisan php install 8.3'.");
            } else {
                for c in &installed {
                    let active_str = if c.is_active { "* (active)" } else { "  " };
                    let warn_str = if c.has_security_warning {
                        format!(
                            " [SECURITY WARNING: {}]",
                            c.security_message.as_deref().unwrap_or("Patches available")
                        )
                    } else {
                        String::new()
                    };
                    println!(
                        "  {:10} {:14} {:<8} {}{}",
                        active_str,
                        c.version,
                        c.source,
                        c.path.display(),
                        warn_str
                    );
                }
            }

            println!("\n=== Available Official Releases (1-Click Download) ===\n");
            for a in &available {
                let status = if a.is_installed {
                    "[Installed]"
                } else {
                    "[Download Ready]"
                };
                println!("  {:<6} {:<10} {:<18} {}", a.id, a.version, status, a.note);
            }
            println!("\nSwitch active version with: servisan php switch <version-or-id>");
            println!("Download new version with : servisan php install <id> [--switch]");
            Ok(())
        }
        PhpCommand::Switch { target } => {
            let new_path = app.switch_php(&target)?;
            println!(
                "Successfully switched active PHP to: {}",
                new_path.display()
            );
            println!("Regenerated config/generated/php.ini with updated extension paths.");
            println!(
                "Note: If Servisan is running, reload or restart to apply the new PHP runtime."
            );
            Ok(())
        }
        PhpCommand::Install { version, switch } => {
            println!("Downloading and installing official PHP {version} from windows.php.net...");
            let installed_path = servisan::php::download_and_install_php(&app.paths, &version)?;
            println!(
                "Successfully installed PHP to: {}",
                installed_path.display()
            );
            if switch {
                app.switch_php(&installed_path.to_string_lossy())?;
                println!("Switched active PHP runtime to newly installed version.");
            } else {
                println!(
                    "To activate this version, run: servisan php switch {}",
                    installed_path.display()
                );
            }
            Ok(())
        }
        PhpCommand::Uninstall { target } => {
            let removed_path = app.uninstall_php(&target)?;
            println!(
                "Successfully removed PHP version from: {}",
                removed_path.display()
            );
            Ok(())
        }
    }
}

/// Handle `servisan migrate` subcommands.
fn run_migrate(app: &App, command: MigrateCommand) -> Result<()> {
    use servisan::migrate::detect;
    use servisan::migrate::discover;
    use servisan::migrate::import::{ConflictStrategy, ImportOptions};

    match command {
        MigrateCommand::Scan { from, port, json } => {
            let sources = if let Some(from_path) = from {
                vec![detect::custom_source(PathBuf::from(from_path), port)]
            } else {
                detect::scan_sources()?
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&sources)?);
                return Ok(());
            }
            if sources.is_empty() {
                println!("No local server installations found.");
                println!("Looked for Laragon, XAMPP, and WAMP on all available drives.");
            } else {
                println!("Found {} local server installation(s):\n", sources.len());
                for (i, src) in sources.iter().enumerate() {
                    println!("  [{}] {}", i + 1, src);
                    println!("      Data: {}", src.data_dir.display());
                    if let Some(ref dump) = src.mysqldump_exe {
                        println!("      Dump: {}", dump.display());
                    }
                    println!();
                }
            }
            Ok(())
        }

        MigrateCommand::List { from, port, json } => {
            let sources = if let Some(from_path) = from {
                vec![detect::custom_source(PathBuf::from(from_path), port)]
            } else {
                detect::scan_sources()?
            };

            if json {
                #[derive(serde::Serialize)]
                struct SourceEntry {
                    source: String,
                    root: String,
                    port: u16,
                    is_running: bool,
                    databases: Vec<discover::DiscoveredDatabase>,
                }
                let mut output = Vec::new();
                for src in &sources {
                    let dbs = discover::discover_databases(src)?;
                    output.push(SourceEntry {
                        source: src.kind.to_string(),
                        root: src.root.display().to_string(),
                        port: src.mysql_port,
                        is_running: src.is_running,
                        databases: dbs,
                    });
                }
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }
            if sources.is_empty() {
                println!("No local server installations found.");
                return Ok(());
            }

            for src in &sources {
                println!("{} ({}):", src.kind, src.root.display());
                let dbs = discover::discover_databases(src)?;
                if dbs.is_empty() {
                    println!("  (no user databases found)\n");
                } else {
                    for db in &dbs {
                        println!("  {:<28} {:>10}", db.name, db.size_display());
                    }
                    println!();
                }
            }
            Ok(())
        }

        MigrateCommand::Import {
            databases,
            from,
            port,
            skip_existing: _,
            overwrite,
        } => {
            let conflict = if overwrite {
                ConflictStrategy::Overwrite
            } else {
                ConflictStrategy::Skip
            };

            let opts = ImportOptions {
                filter: databases,
                on_conflict: conflict,
            };

            if let Some(from_path) = from {
                // Custom source from a specific path.
                let source = detect::custom_source(PathBuf::from(from_path), port);
                servisan::migrate::import::run_import(&app.paths, &app.config, &source, &opts)?;
            } else {
                // Auto-detect all sources and import from each.
                let sources = detect::scan_sources()?;
                if sources.is_empty() {
                    println!("No local server installations found.");
                    println!(
                        "Use --from <path> --port <port> to specify a custom MySQL data directory."
                    );
                    return Ok(());
                }

                for source in &sources {
                    servisan::migrate::import::run_import(&app.paths, &app.config, source, &opts)?;
                    println!();
                }
            }

            Ok(())
        }
    }
}

/// Start services and block until Ctrl+C, then shut everything down.
fn run_foreground(app: &App, only: Option<ServiceKind>) -> Result<()> {
    let mutex_name = app.instance_mutex_name();
    let _instance = match servisan::platform::SingleInstance::acquire(&mutex_name)? {
        Some(guard) => guard,
        None => {
            eprintln!("Servisan is already running in another process.");
            eprintln!("Use 'servisan status' or check the system tray.");
            return Ok(());
        }
    };

    let mut running = app.start(only)?;
    let shutdown = signal::install_shutdown_flag()?;

    println!();
    println!("Servisan is running. Press Ctrl+C to stop.");

    while !shutdown.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(500));
        if !running.any_alive() {
            eprintln!("a service exited unexpectedly; shutting down.");
            break;
        }
    }

    println!();
    app.stop_running(&mut running)?;
    println!("stopped.");
    Ok(())
}

/// Handle `servisan vhost` subcommands.
fn run_vhost(app: &App, command: VhostCommand) -> Result<()> {
    match command {
        VhostCommand::List { json } => {
            let vhosts = app.list_vhosts();
            if json {
                println!("{}", serde_json::to_string_pretty(&vhosts)?);
                return Ok(());
            }

            println!("=== Servisan Virtual Hosts & Linked Projects ===\n");
            if vhosts.is_empty() {
                println!("No virtual hosts registered yet.");
                println!("Link an external project with: servisan vhost add <path>");
                return Ok(());
            }

            for v in &vhosts {
                let hosts_status = if v.hosts_synced {
                    "[hosts synced]"
                } else {
                    "[hosts pending]"
                };
                println!(
                    "• {:<16} -> http://{}  (or http://localhost:{}/{})",
                    v.name, v.domain, app.config.php.port, v.name
                );
                println!("    Path     : {}", v.target_path.display());
                println!("    DocRoot  : {} ({})", v.doc_root.display(), v.framework);
                println!("    Status   : {}\n", hosts_status);
            }
            Ok(())
        }
        VhostCommand::Add { path, name, domain } => {
            let target = Path::new(&path);
            let vhost = app.add_vhost(target, name.as_deref(), domain.as_deref())?;
            println!("Successfully registered virtual host:");
            println!("  Name       : {}", vhost.name);
            println!("  Domain     : http://{}", vhost.domain);
            println!(
                "  Localhost  : http://localhost:{}/{}",
                app.config.php.port, vhost.name
            );
            println!(
                "  DocRoot    : {} ({})",
                vhost.doc_root.display(),
                vhost.framework
            );
            if vhost.hosts_synced {
                println!(
                    "  Hosts file : 127.0.0.1 {} registered in Windows hosts",
                    vhost.domain
                );
            } else {
                println!("  Hosts file : (Admin permission needed to update C:\\Windows\\System32\\drivers\\etc\\hosts)");
                println!(
                    "               You can access it immediately at: http://localhost:{}/{}",
                    app.config.php.port, vhost.name
                );
            }
            Ok(())
        }
        VhostCommand::Delete { target } => {
            let removed = app.remove_vhost(&target)?;
            println!(
                "Successfully removed virtual host: {} ({})",
                removed.name, removed.domain
            );
            println!("Removed junction link from www/{}", removed.name);
            Ok(())
        }
    }
}
