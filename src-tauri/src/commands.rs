use ai_limits::get_limits::{default_source_plan, get_source_plan_limits};
use ai_limits::presentation::{
    format_user_timestamp, normalize_percent, remaining_percent_for_display,
    source_label_for_display, window_label_for_display, TimeContext,
};
use ai_limits::types::StructuredSourceInfo;

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
pub fn get_provider_limits() -> Vec<ProviderLimits> {
    default_source_plan()
        .into_iter()
        .map(|plan| {
            let id = plan.label().to_string();
            match get_source_plan_limits(plan) {
                Ok(report) => provider_limits_from_structured(&id, &report.data.structured),
                Err(error) => provider_error(&id, error.to_string()),
            }
        })
        .collect()
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
