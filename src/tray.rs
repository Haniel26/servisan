//! System-tray UI (Phase 2).
//!
//! A tray-only interface: the manager lives in the notification area with a
//! right-click menu to control services and open common folders. No webview,
//! no window — keeps the binary tiny and the footprint low.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

use crate::app::{App, Running};

const ID_START: &str = "servisan.start";
const ID_STOP: &str = "servisan.stop";
const ID_WWW: &str = "servisan.www";
const ID_CONFIG: &str = "servisan.config";
const ID_LOGS: &str = "servisan.logs";
const ID_PMA: &str = "servisan.phpmyadmin";
const ID_EXIT: &str = "servisan.exit";

/// A user action triggered from the tray menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartAll,
    StopAll,
    OpenWww,
    OpenConfig,
    OpenLogs,
    OpenPhpMyAdmin,
    Exit,
}

/// Map a menu item id to its action.
fn action_for(id: &str) -> Option<Action> {
    Some(match id {
        ID_START => Action::StartAll,
        ID_STOP => Action::StopAll,
        ID_WWW => Action::OpenWww,
        ID_CONFIG => Action::OpenConfig,
        ID_LOGS => Action::OpenLogs,
        ID_PMA => Action::OpenPhpMyAdmin,
        ID_EXIT => Action::Exit,
        _ => return None,
    })
}

/// Generate a simple circular status icon: green when running, grey otherwise.
fn status_icon(running: bool) -> Result<Icon> {
    const SIZE: u32 = 32;
    let rgba = if running {
        include_bytes!("../assets/tray-running.rgba")
    } else {
        include_bytes!("../assets/tray-stopped.rgba")
    };

    Icon::from_rgba(rgba.to_vec(), SIZE, SIZE)
        .map_err(|e| anyhow!("building tray icon: {e}"))
}

/// Open a file or folder in the shell's default handler.
fn open_path(path: &Path) {
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

/// Result of handling a tray action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Exit,
}

/// Report a tray failure: to stderr, and to `logs/servisan.log` so it survives
/// once the console is gone.
fn report_error(app: &App, msg: &str) {
    eprintln!("{msg}");

    use std::io::Write;
    let dir = app.paths.logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("servisan.log"))
    {
        let _ = writeln!(file, "{msg}");
    }
}

/// Apply a tray action to the running services. Shared by the event loop and
/// tests so the control flow is exercised by both.
pub fn apply_action(app: &App, running: &mut Option<Running>, action: Action) -> Outcome {
    match action {
        Action::StartAll => match app.start(None) {
            Ok(r) => *running = Some(r),
            Err(e) => report_error(app, &format!("failed to start services: {e}")),
        },
        Action::StopAll => {
            if let Some(mut r) = running.take() {
                if let Err(e) = app.stop_running(&mut r) {
                    report_error(app, &format!("failed to stop services: {e}"));
                }
            }
        }
        Action::OpenWww => open_path(&app.paths.www_dir()),
        Action::OpenConfig => open_path(&app.paths.config_file()),
        Action::OpenLogs => open_path(&app.paths.logs_dir()),
        Action::OpenPhpMyAdmin => {
            if let Err(e) = app.open_phpmyadmin() {
                report_error(app, &format!("could not open phpMyAdmin: {e}"));
            }
        }
        Action::Exit => {
            if let Some(mut r) = running.take() {
                if let Err(e) = app.stop_running(&mut r) {
                    report_error(app, &format!("failed to stop services on exit: {e}"));
                }
            }
            return Outcome::Exit;
        }
    }
    Outcome::Continue
}

/// Detach this process from its console.
///
/// The binary is built as a console application so the CLI subcommands can
/// print. Tray mode has no console UI, so the window would otherwise just sit
/// there blank ("blank hitam" on a double-click). Freeing it closes the console
/// this process owns and leaves any other console untouched.
#[cfg(windows)]
fn detach_console() {
    use windows::Win32::System::Console::FreeConsole;
    unsafe {
        let _ = FreeConsole();
    }
}

#[cfg(not(windows))]
fn detach_console() {}

/// The actionable menu items, each created with an explicit id.
///
/// `MenuItem::new` mints a counter-based id ("0", "1", ...), which `action_for`
/// could never match: the menu would open but every item would be dead.
fn actionable_items() -> [MenuItem; 6] {
    [
        MenuItem::with_id(ID_START, "Start All", true, None),
        MenuItem::with_id(ID_STOP, "Stop All", false, None),
        MenuItem::with_id(ID_WWW, "Open www", true, None),
        MenuItem::with_id(ID_CONFIG, "Open config", true, None),
        MenuItem::with_id(ID_LOGS, "Open logs", true, None),
        MenuItem::with_id(ID_EXIT, "Exit", true, None),
    ]
}

/// Build the context menu with a variable item enabling phpMyAdmin integration.
pub fn build_menu(installed: bool) -> Result<(Menu, MenuItem, MenuItem)> {
    let [start, stop, www, config, logs, exit] = actionable_items();

    let menu = Menu::new();
    menu.append_items(&[
        &start,
        &stop,
        &PredefinedMenuItem::separator(),
        &www,
        &config,
        &logs,
        &PredefinedMenuItem::separator(),
        &exit,
    ])?;

    if installed {
        let phpmyadmin = MenuItem::with_id(
            ID_PMA,
            "Open phpMyAdmin",
            true,
            None,
        );
        menu.append_items(&[&PredefinedMenuItem::separator(), &phpmyadmin])?;
    }

    Ok((menu, start, stop))
}

pub fn phpmyadmin_item() -> MenuItem {
    MenuItem::with_id(ID_PMA, "Open phpMyAdmin", true, None)
}

/// Run the tray event loop until the user chooses Exit. Diverges.
pub fn run(app: App) -> Result<()> {
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();

    let installed = app.phpmyadmin_installed();
    let (menu, start, stop) = build_menu(installed)?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Servisan - stopped")
        .with_icon(status_icon(false)?)
        .with_menu_on_left_click(true)
        .build()?;

    let mut running: Option<Running> = None;
    let mut last_active: Option<bool> = None;

    if app.config.general.auto_start_all {
        match app.start(None) {
            Ok(r) => running = Some(r),
            Err(e) => report_error(&app, &format!("failed to start services: {e}")),
        }
    }

    let active = running.as_mut().map(Running::any_alive).unwrap_or(false);
    if last_active != Some(active) {
        last_active = Some(active);
        let _ = tray.set_icon(Some(status_icon(active).unwrap_or_else(|_| {
            status_icon(false).expect("fallback icon")
        })));
        let _ = tray.set_tooltip(Some(if active {
            "Servisan - running"
        } else {
            "Servisan - stopped"
        }));
        start.set_enabled(!active);
        stop.set_enabled(active);
    }

    detach_console();

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(250));

        if let Event::NewEvents(_) = event {
            while let Ok(menu_event) = MenuEvent::receiver().try_recv() {
                // An id that maps to nothing means the menu wiring drifted; the
                // console is gone, so record it where "Open logs" can show it.
                let Some(action) = action_for(menu_event.id.as_ref()) else {
                    report_error(
                        &app,
                        &format!("unhandled tray menu id {:?}", menu_event.id.as_ref()),
                    );
                    continue;
                };

                if apply_action(&app, &mut running, action) == Outcome::Exit {
                    *control_flow = ControlFlow::ExitWithCode(0);
                    return;
                }
            }

            // Refresh status indicator / menu state on change.
            let active = running.as_mut().map(Running::any_alive).unwrap_or(false);
            if last_active != Some(active) {
                last_active = Some(active);
                let _ = tray.set_icon(Some(status_icon(active).unwrap_or_else(|_| {
                    status_icon(false).expect("fallback icon")
                })));
                let _ = tray.set_tooltip(Some(if active {
                    "Servisan - running"
                } else {
                    "Servisan - stopped"
                }));
                start.set_enabled(!active);
                stop.set_enabled(active);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_menu_ids_to_actions() {
        assert_eq!(action_for(ID_START), Some(Action::StartAll));
        assert_eq!(action_for(ID_STOP), Some(Action::StopAll));
        assert_eq!(action_for(ID_WWW), Some(Action::OpenWww));
        assert_eq!(action_for(ID_CONFIG), Some(Action::OpenConfig));
        assert_eq!(action_for(ID_LOGS), Some(Action::OpenLogs));
        assert_eq!(action_for(ID_EXIT), Some(Action::Exit));
        assert_eq!(action_for("unknown"), None);
    }

    #[test]
    fn builds_status_icons() {
        assert!(status_icon(false).is_ok());
        assert!(status_icon(true).is_ok());
    }

    #[test]
    fn every_actionable_menu_item_maps_to_an_action() {
        for item in actionable_items() {
            let id = item.id().as_ref().to_string();
            assert!(
                action_for(&id).is_some(),
                "menu item with id {id:?} has no action"
            );
        }
    }

    #[test]
    fn built_menu_items_reach_the_expected_actions() {
        let (_menu, start, stop) = build_menu(false).unwrap();
        assert_eq!(action_for(start.id().as_ref()), Some(Action::StartAll));
        assert_eq!(action_for(stop.id().as_ref()), Some(Action::StopAll));
    }

    #[test]
    fn report_error_appends_to_log() {
        let dir = std::env::temp_dir().join("servisan_report_error");
        let _ = std::fs::remove_dir_all(&dir);

        let app = App {
            paths: crate::paths::Paths::new(&dir),
            config: crate::config::Config::default(),
        };
        report_error(&app, "boom one");
        report_error(&app, "boom two");

        let text = std::fs::read_to_string(dir.join("logs").join("servisan.log")).unwrap();
        assert!(text.contains("boom one"));
        assert!(text.contains("boom two"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[ignore = "starts real PHP + MySQL services on the configured ports"]
    fn start_and_stop_cycle() {
        let _guard = crate::HEAVY_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let app = match App::load() {
            Ok(a) => a,
            Err(_) => return,
        };
        if !app.paths.php_exe().is_file() || !app.paths.mysqld_exe().is_file() {
            eprintln!("skipping: binaries not installed");
            return;
        }

        let mut running = None;
        assert_eq!(
            apply_action(&app, &mut running, Action::StartAll),
            Outcome::Continue
        );
        assert!(running.is_some(), "services should be running after StartAll");

        assert_eq!(
            apply_action(&app, &mut running, Action::StopAll),
            Outcome::Continue
        );
        assert!(running.is_none(), "services should be cleared after StopAll");
    }
}
