use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubStatusSummary {
    pub page: Page,
    pub status: Status,
    pub components: Vec<Component>,
    pub incidents: Vec<Incident>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub name: String,
    pub url: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub indicator: StatusIndicator,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusIndicator {
    None,
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub status: String,
    pub description: Option<String>,
    /// True for group containers, which the UI filters out.
    #[serde(default)]
    pub group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OverallStatus {
    Healthy,
    Degraded,
    Unhealthy,
    /// No data yet, or the last fetch failed.
    Unknown,
}

impl OverallStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Degraded => "Degraded",
            Self::Unhealthy => "Unhealthy",
            Self::Unknown => "Unknown",
        }
    }
}

impl From<StatusIndicator> for OverallStatus {
    fn from(i: StatusIndicator) -> Self {
        match i {
            StatusIndicator::None => Self::Healthy,
            StatusIndicator::Minor => Self::Degraded,
            StatusIndicator::Major | StatusIndicator::Critical => Self::Unhealthy,
        }
    }
}

#[derive(Debug)]
pub struct AppStatusState {
    pub current: OverallStatus,
    /// Last status from a successful fetch; `Unknown` never overwrites it.
    pub last_known: Option<OverallStatus>,
    /// Last good payload, kept for display while offline.
    pub summary: Option<GitHubStatusSummary>,
    /// Fallback downtime start when no open incident is reported.
    pub first_non_healthy_at: Option<DateTime<Utc>>,
}

impl Default for AppStatusState {
    fn default() -> Self {
        Self {
            current: OverallStatus::Unknown,
            last_known: None,
            summary: None,
            first_non_healthy_at: None,
        }
    }
}

impl AppStatusState {
    /// Records a fetch result. Returns `(from, to)` when the known status
    /// changed; the first successful fetch only sets the baseline.
    pub fn apply<E>(
        &mut self,
        result: Result<GitHubStatusSummary, E>,
        now: DateTime<Utc>,
    ) -> Option<(OverallStatus, OverallStatus)> {
        let Ok(summary) = result else {
            self.current = OverallStatus::Unknown;
            return None;
        };
        let status = OverallStatus::from(summary.status.indicator);
        self.first_non_healthy_at = match status {
            OverallStatus::Healthy => None,
            _ => self.first_non_healthy_at.or(Some(now)),
        };
        self.summary = Some(summary);
        self.current = status;
        let prev = self.last_known.replace(status);
        prev.filter(|p| *p != status).map(|p| (p, status))
    }

    #[allow(dead_code)] // used by the dashboard in Phase 2
    /// Start of the current downtime: earliest open incident, else first observation.
    pub fn incident_start(&self) -> Option<DateTime<Utc>> {
        if self.current == OverallStatus::Healthy {
            return None;
        }
        let from_incidents = self
            .summary
            .as_ref()
            .and_then(|s| s.incidents.iter().map(|i| i.created_at).min());
        from_incidents.or(self.first_non_healthy_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(indicator: &str, incidents: &str) -> GitHubStatusSummary {
        serde_json::from_str(&format!(
            r#"{{"page":{{"name":"GitHub","url":"https://www.githubstatus.com","updated_at":"2026-01-01T00:00:00Z"}},
                "status":{{"indicator":"{indicator}","description":"x"}},
                "components":[{{"id":"a","name":"Git Operations","status":"operational","description":null,"group":false}}],
                "incidents":[{incidents}]}}"#
        ))
        .unwrap()
    }

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn maps_indicators() {
        assert_eq!(
            OverallStatus::from(StatusIndicator::None),
            OverallStatus::Healthy
        );
        assert_eq!(
            OverallStatus::from(StatusIndicator::Minor),
            OverallStatus::Degraded
        );
        assert_eq!(
            OverallStatus::from(StatusIndicator::Critical),
            OverallStatus::Unhealthy
        );
    }

    #[test]
    fn first_fetch_is_silent_and_unknown_never_notifies() {
        let mut s = AppStatusState::default();
        let t = at("2026-01-01T00:00:00Z");
        assert_eq!(s.apply::<()>(Ok(summary("none", "")), t), None);
        assert_eq!(s.apply::<()>(Err(()), t), None);
        assert_eq!(s.current, OverallStatus::Unknown);
        // recovering to the same known status is not a change
        assert_eq!(s.apply::<()>(Ok(summary("none", "")), t), None);
        assert_eq!(
            s.apply::<()>(Ok(summary("minor", "")), t),
            Some((OverallStatus::Healthy, OverallStatus::Degraded))
        );
    }

    #[test]
    fn incident_start_prefers_earliest_incident_then_first_seen() {
        let mut s = AppStatusState::default();
        let t = at("2026-01-02T00:00:00Z");
        s.apply::<()>(Ok(summary("minor", "")), t);
        assert_eq!(s.incident_start(), Some(t));
        let inc = |id: &str, ts: &str| {
            format!(r#"{{"id":"{id}","name":"n","status":"investigating","created_at":"{ts}"}}"#)
        };
        let incidents = format!(
            "{},{}",
            inc("1", "2026-01-01T10:00:00Z"),
            inc("2", "2026-01-01T09:00:00Z")
        );
        s.apply::<()>(Ok(summary("minor", &incidents)), t);
        assert_eq!(s.incident_start(), Some(at("2026-01-01T09:00:00Z")));
        s.apply::<()>(Ok(summary("none", "")), t);
        assert_eq!(s.incident_start(), None);
    }
}
