use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use tauri::async_runtime::{channel, Sender};

use ai_limits::types::StructuredSourceInfo;

/// One current `StructuredSourceInfo` snapshot per provider, shared by every
/// surface (Main Window, Menu Bar Popover). Written only on a successful
/// collection, after the provider's source chain has selected its result and
/// applied account-field backfill — never with raw provider output, stderr,
/// the UI-facing `ProviderLimits` model, or a failed collection's data.
pub type StructuredInfoCache = Arc<Mutex<HashMap<String, StructuredSourceInfo>>>;

pub fn new_structured_info_cache() -> StructuredInfoCache {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Reads the current shared snapshot for `provider_id`, if any, without
/// starting or joining a collection. Backs `get_cached_provider_limits`, used
/// by a surface that is initializing (or was just re-enabled) so it can
/// render the snapshot another surface already collected instead of always
/// waiting on its own fetch.
pub fn read_cached_structured_info(
    cache: &StructuredInfoCache,
    provider_id: &str,
) -> Option<StructuredSourceInfo> {
    sync_cursor_cache(cache);
    cache.lock().ok()?.get(provider_id).cloned()
}

/// Credential changes invalidate account-bound values before either surface can read them.
pub fn sync_cursor_cache(cache: &StructuredInfoCache) {
    static EPOCH: AtomicU64 = AtomicU64::new(0);
    let current = ai_limits::providers::cursor_api2::cache_epoch();
    if EPOCH.swap(current, Ordering::SeqCst) != current {
        if let Ok(mut cache) = cache.lock() {
            cache.remove("cursor");
        }
    }
}

pub type CollectResult = (Result<StructuredSourceInfo, String>, Option<u64>);

#[derive(Default)]
struct InFlightEntry {
    waiters: Vec<Sender<CollectResult>>,
}

/// Coordinates concurrent collection requests for the same provider so that
/// only one actual collection (source-chain walk + notification evaluation)
/// runs at a time per provider id; concurrent callers join the in-flight
/// collection and receive its result instead of starting their own.
#[derive(Clone, Default)]
pub struct CollectionCoordinator {
    inflight: Arc<Mutex<HashMap<String, InFlightEntry>>>,
}

impl CollectionCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn collect_once<F, Fut>(&self, key: &str, run: F) -> CollectResult
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = CollectResult> + Send + 'static,
    {
        let mut waiter_rx = None;
        {
            let mut inflight = self.inflight.lock().unwrap();
            if let Some(entry) = inflight.get_mut(key) {
                let (tx, rx) = channel(1);
                entry.waiters.push(tx);
                waiter_rx = Some(rx);
            } else {
                inflight.insert(key.to_string(), InFlightEntry::default());
            }
        }

        if let Some(mut rx) = waiter_rx {
            return rx.recv().await.unwrap_or_else(|| {
                (
                    Err("collection coordinator channel closed".to_string()),
                    None,
                )
            });
        }

        // Detached from this call's own future so a caller being dropped
        // (e.g. the request that triggered the collection getting cancelled)
        // cannot strand the waiters that joined it.
        let result = tauri::async_runtime::spawn(async move { run().await })
            .await
            .unwrap_or_else(|error| (Err(error.to_string()), None));

        let waiters = {
            let mut inflight = self.inflight.lock().unwrap();
            inflight
                .remove(key)
                .map(|entry| entry.waiters)
                .unwrap_or_default()
        };
        for waiter in waiters {
            let _ = waiter.send(result.clone()).await;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use ai_limits::types::{AccountInfo, SourceStatus, UsageInfo};

    fn structured(provider: &str) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: provider.to_string(),
            source: "codex_local".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-08-06T00:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn concurrent_calls_for_the_same_key_share_one_collection() {
        tauri::async_runtime::block_on(async {
            let coordinator = CollectionCoordinator::new();
            let runs = Arc::new(AtomicUsize::new(0));

            let mut handles = Vec::new();
            for _ in 0..5 {
                let coordinator = coordinator.clone();
                let runs = Arc::clone(&runs);
                handles.push(tauri::async_runtime::spawn(async move {
                    coordinator
                        .collect_once("codex", move || async move {
                            runs.fetch_add(1, Ordering::SeqCst);
                            // Gives the other spawned callers a chance to
                            // register as waiters before this one finishes.
                            tauri::async_runtime::spawn_blocking(|| {
                                std::thread::sleep(Duration::from_millis(20));
                            })
                            .await
                            .unwrap();
                            (Ok(structured("codex")), None)
                        })
                        .await
                }));
            }

            for handle in handles {
                let result = handle.await.expect("task should not panic");
                assert_eq!(
                    result.0.expect("collection should succeed").provider,
                    "codex"
                );
            }

            assert_eq!(runs.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn read_cached_structured_info_returns_none_before_any_collection() {
        let cache = new_structured_info_cache();
        assert!(read_cached_structured_info(&cache, "codex").is_none());
    }

    #[test]
    fn read_cached_structured_info_returns_the_last_successful_snapshot() {
        let cache = new_structured_info_cache();
        cache
            .lock()
            .unwrap()
            .insert("codex".to_string(), structured("codex"));

        let snapshot =
            read_cached_structured_info(&cache, "codex").expect("snapshot should be cached");
        assert_eq!(snapshot.provider, "codex");
        assert!(read_cached_structured_info(&cache, "claude").is_none());
    }

    #[test]
    fn distinct_keys_each_run_their_own_collection() {
        tauri::async_runtime::block_on(async {
            let coordinator = CollectionCoordinator::new();
            let runs = Arc::new(AtomicUsize::new(0));

            let codex = {
                let coordinator = coordinator.clone();
                let runs = Arc::clone(&runs);
                coordinator
                    .collect_once("codex", move || async move {
                        runs.fetch_add(1, Ordering::SeqCst);
                        (Ok(structured("codex")), None)
                    })
                    .await
            };
            let claude = {
                let runs = Arc::clone(&runs);
                coordinator
                    .collect_once("claude", move || async move {
                        runs.fetch_add(1, Ordering::SeqCst);
                        (Ok(structured("claude")), None)
                    })
                    .await
            };

            assert_eq!(codex.0.unwrap().provider, "codex");
            assert_eq!(claude.0.unwrap().provider, "claude");
            assert_eq!(runs.load(Ordering::SeqCst), 2);
        });
    }
}
