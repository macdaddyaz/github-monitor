use std::{sync::Mutex, time::Duration};

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{
    models::{AppStatusState, GitHubStatusSummary, OverallStatus},
    tray,
};

const DEFAULT_SUMMARY_URL: &str = "https://www.githubstatus.com/api/v2/summary.json";
/// Overrides the status endpoint, e.g. to point at a local mock when testing notifications.
const SUMMARY_URL_ENV: &str = "GITHUB_MONITOR_SUMMARY_URL";
const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub type SharedMonitor = Mutex<AppStatusState>;

async fn fetch(client: &reqwest::Client, url: &str) -> reqwest::Result<GitHubStatusSummary> {
    client
        .get(url)
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
        let url = std::env::var(SUMMARY_URL_ENV).unwrap_or_else(|_| DEFAULT_SUMMARY_URL.into());
        if url != DEFAULT_SUMMARY_URL {
            log::warn!("using status endpoint override: {url}");
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        loop {
            let result = fetch(&client, &url).await;
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
