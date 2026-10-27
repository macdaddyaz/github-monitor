use std::{sync::Mutex, time::Duration};

use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::{
    models::{AppStatusState, GitHubStatusSummary},
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
            }
            log::debug!("status: {}", current.label());
            tray::set_status(&app, current);
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}
