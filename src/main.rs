use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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
    /// Migrate databases from Laragon, XAMPP, WAMP, or other MySQL installations.
    #[command(name = "migrate")]
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
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
    let app = App::load()?;

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
        Some(Command::Migrate { command }) => run_migrate(&app, command),
        Some(Command::Start { only }) => run_foreground(&app, only.map(ServiceKind::from)),
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
