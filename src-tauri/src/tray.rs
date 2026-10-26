use std::sync::Mutex;

use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuEvent, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Theme, Wry,
};
use tauri_plugin_opener::OpenerExt;

use crate::models::OverallStatus;

const STATUS_PAGE: &str = "https://www.githubstatus.com";
const ID_STATUS: &str = "status";
const ID_QUIT: &str = "quit";

macro_rules! png {
    ($name:literal) => {
        Image::from_bytes(include_bytes!(concat!("../icons/", $name, ".png"))).unwrap()
    };
}

struct Tray {
    icon: TrayIcon,
    status_item: IconMenuItem<Wry>,
    /// (status, dark menubar?)
    state: Mutex<(OverallStatus, bool)>,
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let dark = app
        .get_webview_window("main")
        .and_then(|w| w.theme().ok())
        == Some(Theme::Dark);
    let status_item = IconMenuItem::with_id(
        app,
        ID_STATUS,
        "GitHub status: Unknown",
        true,
        Some(png!("dot-gray")),
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &PredefinedMenuItem::separator(app)?,
            &tauri::menu::MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;
    let icon = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .build(app)?;
    app.manage(Tray {
        icon,
        status_item,
        state: Mutex::new((OverallStatus::Unknown, dark)),
    });
    apply(app);
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        ID_STATUS => {
            if let Err(e) = app.opener().open_url(STATUS_PAGE, None::<&str>) {
                eprintln!("failed to open status page: {e}");
            }
        }
        ID_QUIT => app.exit(0),
        _ => {}
    }
}

pub fn set_status(app: &AppHandle, status: OverallStatus) {
    update(app, |s| s.0 = status);
}

pub fn set_theme(app: &AppHandle, theme: Theme) {
    update(app, |s| s.1 = theme == Theme::Dark);
}

fn update(app: &AppHandle, f: impl FnOnce(&mut (OverallStatus, bool))) {
    if let Some(tray) = app.try_state::<Tray>() {
        f(&mut tray.state.lock().unwrap());
        apply(app);
    }
}

fn apply(app: &AppHandle) {
    let tray = app.state::<Tray>();
    let (status, dark) = *tray.state.lock().unwrap();
    let (icon, dot, template) = match (status, dark) {
        (OverallStatus::Healthy, _) => (png!("tray-healthy"), png!("dot-green"), true),
        (OverallStatus::Degraded, false) => (png!("tray-degraded-light"), png!("dot-yellow"), false),
        (OverallStatus::Degraded, true) => (png!("tray-degraded-dark"), png!("dot-yellow"), false),
        (OverallStatus::Unhealthy, false) => (png!("tray-unhealthy-light"), png!("dot-red"), false),
        (OverallStatus::Unhealthy, true) => (png!("tray-unhealthy-dark"), png!("dot-red"), false),
        (OverallStatus::Unknown, false) => (png!("tray-unknown-light"), png!("dot-gray"), false),
        (OverallStatus::Unknown, true) => (png!("tray-unknown-dark"), png!("dot-gray"), false),
    };
    let _ = tray.icon.set_icon(Some(icon));
    let _ = tray.icon.set_icon_as_template(template);
    let _ = tray.status_item.set_icon(Some(dot));
    let _ = tray
        .status_item
        .set_text(format!("GitHub status: {}", status.label()));
}
