use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use ai_limits::get_limits::{get_source_plan_limits, ui_source_plan, UiSourcePlanOptions};
use ai_limits::notifications;
use ai_limits::presentation::{
    format_user_timestamp, normalize_percent, remaining_percent_for_display,
    source_label_for_display, window_label_for_display, TimeContext,
};
use ai_limits::types::{SourceReport, StructuredSourceInfo};

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitsQuery {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
    pub use_cli_fallback: bool,
    pub notifications_enabled: bool,
}

impl Default for ProviderLimitsQuery {
    fn default() -> Self {
        let defaults = UiSourcePlanOptions::default();
        Self {
            enabled_codex: defaults.enabled_codex,
            enabled_claude: defaults.enabled_claude,
            enabled_cursor: defaults.enabled_cursor,
            use_cli_fallback: defaults.use_cli_fallback,
            notifications_enabled: true,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimits {
    id: String,
    label: String,
    source_id: Option<String>,
    data_timestamp: Option<String>,
    selected_update_frequency: String,
    limits: Vec<ProviderLimitRow>,
    error_message: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitRow {
    label: String,
    remaining_percentage: f64,
    reset_time: Option<String>,
}

#[tauri::command]
pub async fn get_provider_limits(
    query: ProviderLimitsQuery,
    sent_notifications: tauri::State<'_, Arc<Mutex<HashSet<String>>>>,
) -> Result<Vec<ProviderLimits>, String> {
    let sent_notifications = Arc::clone(sent_notifications.inner());

    tauri::async_runtime::spawn_blocking(move || collect_provider_limits(&query, sent_notifications))
        .await
        .map_err(|error| error.to_string())
}

fn collect_provider_limits(
    query: &ProviderLimitsQuery,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
) -> Vec<ProviderLimits> {
    let plan = ui_source_plan(UiSourcePlanOptions {
        enabled_codex: query.enabled_codex,
        enabled_claude: query.enabled_claude,
        enabled_cursor: query.enabled_cursor,
        use_cli_fallback: query.use_cli_fallback,
    });

    plan.into_iter()
        .map(|source_plan| {
            let id = source_plan.label().to_string();
            match get_source_plan_limits(source_plan) {
                Ok(report) => {
                    if query.notifications_enabled {
                        notify_for_report(&report, sent_notifications.clone());
                    }
                    provider_limits_from_structured(&id, &report.data.structured)
                }
                Err(error) => provider_error(&id, error.to_string()),
            }
        })
        .collect()
}

fn notify_for_report(report: &SourceReport, sent_notifications: Arc<Mutex<HashSet<String>>>) {
    let Ok(mut sent) = sent_notifications.lock() else {
        return;
    };

    for notification in notifications::notifications_for_report(report) {
        if sent.insert(notification.dedupe_key.clone()) {
            let _ = notifications::notify(&notification);
        }
    }
}

fn provider_limits_from_structured(id: &str, info: &StructuredSourceInfo) -> ProviderLimits {
    let time_context = TimeContext::from_structured(info);
    let limits = info
        .limits
        .iter()
        .filter_map(|limit| {
            let remaining = normalize_percent(remaining_percent_for_display(limit)?);
            Some(ProviderLimitRow {
                label: window_label_for_display(limit),
                remaining_percentage: remaining,
                reset_time: limit
                    .resets_at
                    .as_deref()
                    .map(|value| format_user_timestamp(value, &time_context)),
            })
        })
        .collect();

    let error_message = if info.status.access_available && info.status.data_available {
        None
    } else {
        info.status
            .message
            .clone()
            .or_else(|| Some("No usable limit data".to_string()))
    };

    ProviderLimits {
        id: id.to_string(),
        label: provider_label(id),
        source_id: Some(source_label_for_display(&info.source)),
        data_timestamp: Some(
            info.data_as_of
                .as_deref()
                .map(|value| format_user_timestamp(value, &time_context))
                .unwrap_or_else(|| "unknown".to_string()),
        ),
        selected_update_frequency: "5 min".to_string(),
        limits,
        error_message,
    }
}

fn provider_error(id: &str, message: String) -> ProviderLimits {
    ProviderLimits {
        id: id.to_string(),
        label: provider_label(id),
        source_id: None,
        data_timestamp: None,
        selected_update_frequency: "5 min".to_string(),
        limits: Vec::new(),
        error_message: Some(message),
    }
}

fn provider_label(id: &str) -> String {
    let mut characters = id.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => id.to_string(),
    }
}
