use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

use ai_limits::get_limits::{
    get_source_plan_limits, ui_source_plan, SourcePlan, UiSourcePlanOptions,
};
use ai_limits::notifications as core_notifications;
use ai_limits::notifications::PreviousRemainingStore;
use ai_limits::types::{SourceReport, StructuredSourceInfo};

use super::provider_limits::{
    provider_error, provider_limits_from_structured, ProviderLimits, ProviderLimitsQuery,
};
use super::structured_cache::{CollectionCoordinator, StructuredInfoCache};

/// Tauri app event emitted after every successful actual collection, payload
/// the same `ProviderLimits` shape a direct `get_single_provider_limits`
/// response carries. Lets every open surface (Main Window, Popover) pick up
/// a result collected for another surface without starting its own
/// collection — see docs/desktop/ui/frontend-state.md and
/// docs/desktop/mac-popover.md#cross-window-sync. Forwarded to the Popover's
/// non-Tauri-managed webview by `popover_panel::install_event_forwarding`.
pub const PROVIDER_UPDATED_EVENT: &str = "provider-updated";

/// Tauri app event emitted right as an actual collection begins (before the
/// source chain runs), payload `{ "id": <provider id> }`. `CollectionCoordinator`
/// guarantees `run_collection` runs at most once per real collection — a
/// concurrent caller for the same provider joins it instead of starting a
/// second one — so this fires exactly once per collection regardless of how
/// many surfaces requested it. Lets every open surface show the same
/// in-flight refresh animation for a card, even when the collection was
/// started by a different surface. Forwarded to the Popover the same way as
/// `PROVIDER_UPDATED_EVENT`.
pub const PROVIDER_REFRESH_STARTED_EVENT: &str = "provider-refresh-started";

/// Tauri app event emitted after a failed actual collection, payload the same
/// `ProviderLimits` shape `PROVIDER_UPDATED_EVENT` carries (built via
/// `provider_error`, so `errorMessage` is set and `limits` is empty). Lets
/// every open surface show the same error state for a card whose collection
/// failed on another surface, without a second collection attempt. The
/// shared structured-data cache is left untouched on failure, same as
/// before — this event only carries the failure to other surfaces' UI state.
pub const PROVIDER_REFRESH_FAILED_EVENT: &str = "provider-refresh-failed";

pub(super) async fn collect_single_provider_limits(
    provider_id: &str,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
    structured_cache: StructuredInfoCache,
    coordinator: CollectionCoordinator,
) -> Result<ProviderLimits, String> {
    let source_plan = ui_source_plan(source_plan_options(query))
        .into_iter()
        .find(|plan| plan.label() == provider_id)
        .ok_or_else(|| format!("Provider '{provider_id}' is disabled or unknown"))?;

    let id = source_plan.label().to_string();
    let notifications_enabled = query.notifications_enabled;

    let result = coordinator
        .collect_once(&id, move || {
            run_collection(
                source_plan,
                notifications_enabled,
                app,
                sent_notifications,
                remaining_store,
                structured_cache,
            )
        })
        .await;

    Ok(match result {
        Ok(structured) => provider_limits_from_structured(&id, &structured),
        Err(error) => provider_error(&id, error),
    })
}

/// The single actual collection for one provider: runs the source chain,
/// evaluates notifications once on success, and — only on success — updates
/// the shared structured-data cache. Always runs on a blocking thread, since
/// the source chain performs blocking file/process/network I/O.
async fn run_collection(
    source_plan: SourcePlan,
    notifications_enabled: bool,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
    remaining_store: Arc<dyn PreviousRemainingStore>,
    structured_cache: StructuredInfoCache,
) -> Result<StructuredSourceInfo, String> {
    let id = source_plan.label().to_string();
    let scheduler_app = app.clone();
    let scheduler_provider_id = id.clone();
    let collection_started_at = tokio::time::Instant::now();

    let _ = app.emit(
        PROVIDER_REFRESH_STARTED_EVENT,
        &StartedPayload { id: id.clone() },
    );

    let result =
        tauri::async_runtime::spawn_blocking(move || match get_source_plan_limits(source_plan) {
            Ok(report) => {
                if notifications_enabled {
                    notify_for_report(&report, &app, &sent_notifications, &remaining_store);
                }
                let structured = report.data.structured;
                if let Ok(mut cache) = structured_cache.lock() {
                    cache.insert(id.clone(), structured.clone());
                }
                let payload = provider_limits_from_structured(&id, &structured);
                let _ = app.emit(PROVIDER_UPDATED_EVENT, &payload);
                Ok(structured)
            }
            Err(error) => {
                let message = error.to_string();
                let payload = provider_error(&id, message.clone());
                let _ = app.emit(PROVIDER_REFRESH_FAILED_EVENT, &payload);
                Err(message)
            }
        })
        .await
        .map_err(|error| error.to_string())?;

    scheduler_app
        .state::<super::BackgroundRefreshScheduler>()
        .collection_finished(scheduler_provider_id, collection_started_at);
    result
}

#[derive(serde::Serialize)]
struct StartedPayload {
    id: String,
}

fn source_plan_options(query: &ProviderLimitsQuery) -> UiSourcePlanOptions {
    UiSourcePlanOptions {
        enabled_codex: query.enabled_codex,
        enabled_claude: query.enabled_claude,
        enabled_cursor: query.enabled_cursor,
    }
}

fn notify_for_report(
    report: &SourceReport,
    app: &tauri::AppHandle,
    sent_notifications: &Arc<Mutex<HashSet<String>>>,
    remaining_store: &Arc<dyn PreviousRemainingStore>,
) {
    let Ok(mut sent) = sent_notifications.lock() else {
        return;
    };

    let delivery = crate::notifications::TauriNotificationDelivery::new(app.clone());
    core_notifications::send_for_report_with_delivery(
        report,
        &mut sent,
        remaining_store.as_ref(),
        &delivery,
    );
}
