use std::collections::HashSet;
use std::process::Command;
use std::sync::{Arc, Mutex};

use ai_limits::get_limits::{
    get_source_plan_limits, ui_source_plan, SourcePlan, UiSourcePlanOptions,
};
use ai_limits::notifications as core_notifications;
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
    credits_remaining: Option<f64>,
    error_message: Option<String>,
    no_fresh_data: bool,
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
    app: tauri::AppHandle,
    sent_notifications: tauri::State<'_, Arc<Mutex<HashSet<String>>>>,
) -> Result<Vec<ProviderLimits>, String> {
    let sent_notifications = Arc::clone(sent_notifications.inner());

    tauri::async_runtime::spawn_blocking(move || {
        collect_provider_limits(&query, app, sent_notifications)
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_single_provider_limits(
    provider_id: String,
    query: ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: tauri::State<'_, Arc<Mutex<HashSet<String>>>>,
) -> Result<ProviderLimits, String> {
    let sent_notifications = Arc::clone(sent_notifications.inner());

    tauri::async_runtime::spawn_blocking(move || {
        collect_single_provider_limits(&provider_id, &query, app, sent_notifications)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    if !is_allowed_external_url(&url) {
        return Err("External URL is not allowed".to_string());
    }

    open_url_with_system(&url).map_err(|error| error.to_string())
}

fn collect_provider_limits(
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
) -> Vec<ProviderLimits> {
    ui_source_plan(source_plan_options(query))
        .into_iter()
        .map(|source_plan| {
            collect_provider_limits_for_plan(
                source_plan,
                query,
                app.clone(),
                Arc::clone(&sent_notifications),
            )
        })
        .collect()
}

fn collect_single_provider_limits(
    provider_id: &str,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
) -> Result<ProviderLimits, String> {
    let source_plan = ui_source_plan(source_plan_options(query))
        .into_iter()
        .find(|plan| plan.label() == provider_id)
        .ok_or_else(|| format!("Provider '{provider_id}' is disabled or unknown"))?;

    Ok(collect_provider_limits_for_plan(
        source_plan,
        query,
        app,
        sent_notifications,
    ))
}

fn collect_provider_limits_for_plan(
    source_plan: SourcePlan,
    query: &ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: Arc<Mutex<HashSet<String>>>,
) -> ProviderLimits {
    let id = source_plan.label().to_string();
    match get_source_plan_limits(source_plan) {
        Ok(report) => {
            if query.notifications_enabled {
                notify_for_report(&report, app, &sent_notifications);
            }
            provider_limits_from_structured(&id, &report.data.structured)
        }
        Err(error) => provider_error(&id, error.to_string()),
    }
}

fn source_plan_options(query: &ProviderLimitsQuery) -> UiSourcePlanOptions {
    UiSourcePlanOptions {
        enabled_codex: query.enabled_codex,
        enabled_claude: query.enabled_claude,
        enabled_cursor: query.enabled_cursor,
        use_cli_fallback: query.use_cli_fallback,
    }
}

fn notify_for_report(
    report: &SourceReport,
    app: tauri::AppHandle,
    sent_notifications: &Arc<Mutex<HashSet<String>>>,
) {
    let Ok(mut sent) = sent_notifications.lock() else {
        return;
    };

    let delivery = crate::notifications::TauriNotificationDelivery::new(app);
    core_notifications::send_for_report_with_delivery(report, &mut sent, &delivery);
}

fn provider_limits_from_structured(id: &str, info: &StructuredSourceInfo) -> ProviderLimits {
    let time_context = TimeContext::from_structured(info);
    let limits: Vec<ProviderLimitRow> = info
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

    let no_fresh_data = info.status.access_available && limits.is_empty();

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
        credits_remaining: info.account.credits_remaining,
        error_message,
        no_fresh_data,
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
        credits_remaining: None,
        error_message: Some(message),
        no_fresh_data: false,
    }
}

fn provider_label(id: &str) -> String {
    let mut characters = id.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => id.to_string(),
    }
}

fn is_allowed_external_url(url: &str) -> bool {
    matches!(
        url,
        "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md"
            | "https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md"
    )
}

#[cfg(target_os = "macos")]
fn open_url_with_system(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn()?.wait()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_url_with_system(url: &str) -> std::io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_url_with_system(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn()?.wait()?;
    Ok(())
}
