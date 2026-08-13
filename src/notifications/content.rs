use serde::{Deserialize, Serialize};

use crate::presentation::{format_user_timestamp, limit_type_label, TimeContext};

use super::kinds::{LimitNotificationKind, NotificationColor};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Notification {
    pub dedupe_key: String,
    pub title: String,
    pub subtitle: String,
    pub message: String,
    pub color: Option<NotificationColor>,
    /// When true, `send_for_report_with_delivery` delivers this notification
    /// unconditionally instead of gating it behind the process-lifetime
    /// `sent` set. Edge-triggered candidates (100% again) already have their
    /// own persistent-store check for "is this a fresh transition"; a static
    /// `dedupe_key` would otherwise wrongly suppress a later, genuinely new
    /// occurrence of the same transition within one run.
    pub always_deliver: bool,
}

impl Notification {
    pub fn limit(
        provider: &str,
        source: &str,
        limit_name: &str,
        kind: LimitNotificationKind,
        remaining_percent: f64,
        resets_at: Option<&str>,
        time_context: &TimeContext,
    ) -> Self {
        let provider = provider_label(provider);
        let type_label = limit_type_label(limit_name);
        let remaining_percent = display_percent(remaining_percent);
        Self {
            dedupe_key: format!(
                "{}|{}|{}|{}",
                provider,
                source,
                type_label,
                kind.remaining_percent()
            ),
            title: format!("{} AI Limits", kind.emoji()),
            subtitle: format!("{provider} {type_label} - {remaining_percent}% left"),
            message: format!("reset {}", reset_label(resets_at, time_context)),
            color: kind.color(),
            always_deliver: false,
        }
    }

    /// "100% again": fired when a limit's remaining percent returns to
    /// exactly 100 after a stored value below 100. See the "100% again"
    /// section of `docs/notifications/overview.md` for trigger rules.
    pub fn replenished(
        provider: &str,
        limit_name: &str,
        resets_at: Option<&str>,
        time_context: &TimeContext,
    ) -> Self {
        let provider = provider_label(provider);
        let type_label = limit_type_label(limit_name);
        Self {
            dedupe_key: format!("{provider}|{type_label}|100-again"),
            title: format!("{} AI Limits", LimitNotificationKind::Replenished.emoji()),
            subtitle: format!("{provider} {type_label} - 100% again"),
            message: format!("reset {}", reset_label(resets_at, time_context)),
            color: LimitNotificationKind::Replenished.color(),
            always_deliver: true,
        }
    }
}

fn provider_label(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "codex" => "Codex".to_string(),
        "claude" => "Claude".to_string(),
        "cursor" => "Cursor".to_string(),
        "" => "AI Limits".to_string(),
        _ => title_case(provider),
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_uppercase().collect::<String>() + chars.as_str()
}

fn display_percent(remaining_percent: f64) -> u8 {
    remaining_percent.clamp(0.0, 100.0).round() as u8
}

fn reset_label(value: Option<&str>, time_context: &TimeContext) -> String {
    value
        .map(|value| format_user_timestamp(value, time_context))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
