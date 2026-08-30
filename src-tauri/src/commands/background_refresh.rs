use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ai_limits::notifications::PreviousRemainingStore;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::time::Instant;

use super::collect::collect_single_provider_limits;
use super::provider_limits::ProviderLimitsQuery;

const ALLOWED_INTERVAL_SECONDS: [u64; 5] = [60, 300, 600, 1_800, 3_600];

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundRefreshConfig {
    query: ProviderLimitsQuery,
    interval_seconds: Option<u64>,
}

impl BackgroundRefreshConfig {
    fn interval(&self) -> Result<Option<Duration>, String> {
        match self.interval_seconds {
            None => Ok(None),
            Some(seconds) if ALLOWED_INTERVAL_SECONDS.contains(&seconds) => {
                Ok(Some(Duration::from_secs(seconds)))
            }
            Some(_) => Err("Unsupported background refresh interval".to_string()),
        }
    }

    fn enabled_provider_ids(&self) -> HashSet<&'static str> {
        [
            ("codex", self.query.enabled_codex),
            ("claude", self.query.enabled_claude),
            ("cursor", self.query.enabled_cursor),
        ]
        .into_iter()
        .filter_map(|(id, enabled)| enabled.then_some(id))
        .collect()
    }
}

enum SchedulerMessage {
    Configure(BackgroundRefreshConfig),
    CollectionFinished {
        provider_id: String,
        started_at: Instant,
    },
}

pub struct BackgroundRefreshScheduler {
    sender: mpsc::UnboundedSender<SchedulerMessage>,
    receiver: Mutex<Option<mpsc::UnboundedReceiver<SchedulerMessage>>>,
}

impl BackgroundRefreshScheduler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Mutex::new(Some(receiver)),
        }
    }

    pub fn start(&self, app: tauri::AppHandle) {
        let receiver = self
            .receiver
            .lock()
            .expect("background refresh receiver lock should not be poisoned")
            .take()
            .expect("background refresh scheduler must start exactly once");
        tauri::async_runtime::spawn(run_scheduler(app, receiver));
    }

    pub fn configure(&self, config: BackgroundRefreshConfig) -> Result<(), String> {
        config.interval()?;
        self.sender
            .send(SchedulerMessage::Configure(config))
            .map_err(|_| "Background refresh scheduler is unavailable".to_string())
    }

    pub fn collection_finished(&self, provider_id: String, started_at: Instant) {
        let _ = self.sender.send(SchedulerMessage::CollectionFinished {
            provider_id,
            started_at,
        });
    }
}

struct SchedulerState {
    config: Option<BackgroundRefreshConfig>,
    last_attempts: HashMap<String, Instant>,
    deadlines: HashMap<String, Instant>,
}

impl SchedulerState {
    fn new() -> Self {
        Self {
            config: None,
            last_attempts: HashMap::new(),
            deadlines: HashMap::new(),
        }
    }

    fn configure(&mut self, config: BackgroundRefreshConfig, now: Instant) {
        let interval = config
            .interval()
            .expect("configure validates the interval before sending it");
        let enabled = config.enabled_provider_ids();
        self.config = Some(config);
        self.deadlines.clear();

        let Some(interval) = interval else {
            return;
        };

        for provider_id in enabled {
            let deadline = self
                .last_attempts
                .get(provider_id)
                .map(|last_attempt| *last_attempt + interval)
                .unwrap_or(now);
            self.deadlines.insert(provider_id.to_string(), deadline);
        }
    }

    fn collection_finished(&mut self, provider_id: String, now: Instant) {
        self.last_attempts.insert(provider_id.clone(), now);
        let Some(config) = &self.config else {
            return;
        };
        let enabled = config.enabled_provider_ids();
        let interval = config
            .interval()
            .expect("stored scheduler config must contain a validated interval");
        match (enabled.contains(provider_id.as_str()), interval) {
            (true, Some(interval)) => {
                self.deadlines.insert(provider_id, now + interval);
            }
            _ => {
                self.deadlines.remove(&provider_id);
            }
        }
    }

    fn next_deadline(&self) -> Option<Instant> {
        self.deadlines.values().copied().min()
    }

    fn take_due(&mut self, now: Instant) -> Vec<(String, ProviderLimitsQuery)> {
        let Some(config) = &self.config else {
            return Vec::new();
        };
        let interval = config
            .interval()
            .expect("stored scheduler config must contain a validated interval");
        let Some(interval) = interval else {
            return Vec::new();
        };
        let query = config.query.clone();
        let due: Vec<String> = self
            .deadlines
            .iter()
            .filter_map(|(provider_id, deadline)| (*deadline <= now).then_some(provider_id.clone()))
            .collect();

        for provider_id in &due {
            self.deadlines.insert(provider_id.clone(), now + interval);
        }

        due.into_iter()
            .map(|provider_id| (provider_id, query.clone()))
            .collect()
    }
}

async fn run_scheduler(
    app: tauri::AppHandle,
    mut receiver: mpsc::UnboundedReceiver<SchedulerMessage>,
) {
    let mut state = SchedulerState::new();

    loop {
        let message = match state.next_deadline() {
            Some(deadline) => match tokio::time::timeout_at(deadline, receiver.recv()).await {
                Ok(message) => message,
                Err(_) => {
                    start_due_collections(&app, state.take_due(Instant::now()));
                    continue;
                }
            },
            None => receiver.recv().await,
        };

        match message {
            Some(SchedulerMessage::Configure(config)) => state.configure(config, Instant::now()),
            Some(SchedulerMessage::CollectionFinished {
                provider_id,
                started_at,
            }) => {
                state.collection_finished(provider_id, started_at);
            }
            None => return,
        }
    }
}

fn start_due_collections(app: &tauri::AppHandle, collections: Vec<(String, ProviderLimitsQuery)>) {
    for (provider_id, query) in collections {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let sent_notifications = app.state::<Arc<Mutex<HashSet<String>>>>();
            let remaining_store = app.state::<Arc<dyn PreviousRemainingStore>>();
            let structured_cache = app.state::<super::StructuredInfoCache>();
            let coordinator = app.state::<super::CollectionCoordinator>();
            let _ = collect_single_provider_limits(
                &provider_id,
                &query,
                app.clone(),
                Arc::clone(sent_notifications.inner()),
                Arc::clone(remaining_store.inner()),
                Arc::clone(structured_cache.inner()),
                coordinator.inner().clone(),
            )
            .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(interval_seconds: Option<u64>) -> BackgroundRefreshConfig {
        BackgroundRefreshConfig {
            query: ProviderLimitsQuery::default(),
            interval_seconds,
        }
    }

    #[test]
    fn only_supported_intervals_are_accepted() {
        for seconds in ALLOWED_INTERVAL_SECONDS {
            assert_eq!(
                config(Some(seconds)).interval().unwrap(),
                Some(Duration::from_secs(seconds))
            );
        }
        assert!(config(None).interval().unwrap().is_none());
        assert!(config(Some(2)).interval().is_err());
    }

    #[test]
    fn enabled_providers_are_due_immediately_before_their_first_attempt() {
        let now = Instant::now();
        let mut state = SchedulerState::new();
        state.configure(config(Some(600)), now);
        let due = state.take_due(now);
        assert_eq!(due.len(), 3);
        assert_eq!(
            due.iter()
                .map(|(id, _)| id.as_str())
                .collect::<HashSet<_>>(),
            HashSet::from(["codex", "claude", "cursor"])
        );
    }

    #[test]
    fn a_finished_collection_moves_only_that_provider_one_interval_forward() {
        let now = Instant::now();
        let mut state = SchedulerState::new();
        state.configure(config(Some(600)), now);
        state.collection_finished("codex".to_string(), now);
        assert_eq!(
            state.deadlines.get("codex"),
            Some(&(now + Duration::from_secs(600)))
        );
        assert_eq!(state.deadlines.get("claude"), Some(&now));
    }

    #[test]
    fn manual_only_clears_every_deadline() {
        let now = Instant::now();
        let mut state = SchedulerState::new();
        state.configure(config(Some(600)), now);
        state.configure(config(None), now);
        assert!(state.deadlines.is_empty());
    }
}
