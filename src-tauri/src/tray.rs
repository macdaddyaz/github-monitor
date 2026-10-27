use std::sync::Mutex;

use tauri::{
    image::Image,
    menu::{IconMenuItem, Menu, MenuEvent, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Wry,
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
    status: Mutex<OverallStatus>,
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
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
        status: Mutex::new(OverallStatus::Unknown),
    });
    apply(app);
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        ID_STATUS => open_status_page(app),
        ID_QUIT => app.exit(0),
        _ => {}
    }
}

pub fn open_status_page(app: &AppHandle) {
    if let Err(e) = app.opener().open_url(STATUS_PAGE, None::<&str>) {
        log::error!("failed to open status page: {e}");
    }
}

pub fn set_status(app: &AppHandle, status: OverallStatus) {
    if app.try_state::<Tray>().is_some() {
        *app.state::<Tray>().status.lock().unwrap() = status;
        apply(app);
    }
}

/// Tray icons are template images so macOS tints them to match the menubar; the badge
/// shape carries the status. Color lives on the menu item's dot instead.
fn apply(app: &AppHandle) {
    let tray = app.state::<Tray>();
    let status = *tray.status.lock().unwrap();
    let (icon, dot) = match status {
        OverallStatus::Healthy => (png!("tray-healthy"), png!("dot-green")),
        OverallStatus::Degraded => (png!("tray-degraded"), png!("dot-yellow")),
        OverallStatus::Unhealthy => (png!("tray-unhealthy"), png!("dot-red")),
        OverallStatus::Unknown => (png!("tray-unknown"), png!("dot-gray")),
    };
    let _ = tray.icon.set_icon(Some(icon));
    let _ = tray.icon.set_icon_as_template(true);
    let _ = tray.status_item.set_icon(Some(dot));
    let _ = tray
        .status_item
        .set_text(format!("GitHub status: {}", status.label()));
}
