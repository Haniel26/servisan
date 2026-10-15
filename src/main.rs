use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

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
        Some(Command::Start { only }) => run_foreground(&app, only.map(ServiceKind::from)),
    }
}

/// Start services and block until Ctrl+C, then shut everything down.
fn run_foreground(app: &App, only: Option<ServiceKind>) -> Result<()> {
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
