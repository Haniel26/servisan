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
        Some(Command::Start { only }) => run_foreground(&app, only.map(ServiceKind::from)),
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
