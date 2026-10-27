use std::{sync::Mutex, time::Duration};

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{
    models::{AppStatusState, GitHubStatusSummary, OverallStatus},
    tray,
};

const SUMMARY_URL: &str = "https://www.githubstatus.com/api/v2/summary.json";
const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub type SharedMonitor = Mutex<AppStatusState>;

async fn fetch(client: &reqwest::Client) -> reqwest::Result<GitHubStatusSummary> {
    client
        .get(SUMMARY_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

/// Opens the status page when a notification is clicked.
pub fn init_notifications(app: &AppHandle) -> tauri_plugin_notification::Result<()> {
    let handle = app.clone();
    app.notification()
        .on_action(move |_| tray::open_status_page(&handle))
}

fn notify(app: &AppHandle, to: OverallStatus) {
    let result = app
        .notification()
        .builder()
        .title("GitHub Status Changed")
        .body(format!("Service is now {}.", to.label()))
        .show();
    if let Err(e) = result {
        log::error!("failed to show notification: {e}");
    }
}

/// Polls the status API forever, updating shared state and the tray.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        loop {
            let result = fetch(&client).await;
            if let Err(e) = &result {
                log::warn!("status fetch failed: {e}");
            }
            let monitor = app.state::<SharedMonitor>();
            let (change, current) = {
                let mut state = monitor.lock().unwrap();
                (state.apply(result, Utc::now()), state.current)
            };
            if let Some((from, to)) = change {
                log::info!("status changed: {} -> {}", from.label(), to.label());
                notify(&app, to);
            }
            log::debug!("status: {}", current.label());
            tray::set_status(&app, current);
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}
