use std::io;

use chrono::{DateTime, Local, TimeZone, Utc};

use crate::types::{LimitInfo, SourceReport, StructuredSourceInfo};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    not(any(target_os = "macos", target_os = "windows", target_os = "linux"))
))]
mod noop;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NotificationColor {
    Green,
    Yellow,
    Orange,
    Red,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LimitNotificationKind {
    Remaining75,
    Remaining50,
    Remaining25,
    Remaining10,
}

impl LimitNotificationKind {
    pub const ALL: [Self; 4] = [
        Self::Remaining75,
        Self::Remaining50,
        Self::Remaining25,
        Self::Remaining10,
    ];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "75" => Ok(Self::Remaining75),
            "50" => Ok(Self::Remaining50),
            "25" => Ok(Self::Remaining25),
            "10" => Ok(Self::Remaining10),
            _ => Err("expected one of: 75, 50, 25, 10".to_string()),
        }
    }

    pub fn remaining_percent(self) -> u8 {
        match self {
            Self::Remaining75 => 75,
            Self::Remaining50 => 50,
            Self::Remaining25 => 25,
            Self::Remaining10 => 10,
        }
    }

    pub fn color(self) -> NotificationColor {
        match self {
            Self::Remaining75 => NotificationColor::Green,
            Self::Remaining50 => NotificationColor::Yellow,
            Self::Remaining25 => NotificationColor::Orange,
            Self::Remaining10 => NotificationColor::Red,
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Self::Remaining75 => "🟢",
            Self::Remaining50 => "🟡",
            Self::Remaining25 => "🟠",
            Self::Remaining10 => "🔴",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Notification {
    pub dedupe_key: String,
    pub title: String,
    pub subtitle: String,
    pub message: String,
    pub color: NotificationColor,
}

impl Notification {
    pub fn limit(
        provider: &str,
        source: &str,
        limit_name: &str,
        kind: LimitNotificationKind,
        remaining_percent: f64,
        resets_at: Option<&str>,
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
            message: format!("reset {}", reset_label(resets_at)),
            color: kind.color(),
        }
    }

    pub fn test(kind: LimitNotificationKind) -> Self {
        let remaining_percent = kind.remaining_percent();
        Self {
            dedupe_key: format!("test|{remaining_percent}"),
            title: format!("{} AI Limits", kind.emoji()),
            subtitle: format!("AI Limits test - {remaining_percent}% left"),
            message: "reset unknown".to_string(),
            color: kind.color(),
        }
    }
}

pub fn notify(notification: &Notification) -> io::Result<()> {
    platform_notify(notification)
}

pub fn notify_test(kind: LimitNotificationKind) -> io::Result<()> {
    notify(&Notification::test(kind))
}

pub fn notifications_for_report(report: &SourceReport) -> Vec<Notification> {
    notifications_for_structured(&report.data.structured)
}

pub fn notifications_for_structured(info: &StructuredSourceInfo) -> Vec<Notification> {
    if !info.status.access_available || !info.status.data_available {
        return Vec::new();
    }

    info.limits
        .iter()
        .filter_map(|limit| {
            let remaining = remaining_percent(limit)?;
            let kind = matching_kind(remaining)?;
            Some(Notification::limit(
                &info.provider,
                &info.source,
                &limit.name,
                kind,
                remaining,
                limit.resets_at.as_deref(),
            ))
        })
        .collect()
}

fn matching_kind(remaining_percent: f64) -> Option<LimitNotificationKind> {
    let remaining = remaining_percent.clamp(0.0, 100.0);

    if remaining <= 10.0 {
        Some(LimitNotificationKind::Remaining10)
    } else if remaining <= 25.0 {
        Some(LimitNotificationKind::Remaining25)
    } else if remaining <= 50.0 {
        Some(LimitNotificationKind::Remaining50)
    } else if remaining <= 75.0 {
        Some(LimitNotificationKind::Remaining75)
    } else {
        None
    }
}

fn remaining_percent(limit: &LimitInfo) -> Option<f64> {
    limit
        .remaining_percent
        .or_else(|| limit.used_percent.map(|used| 100.0 - used))
        .map(|remaining| remaining.clamp(0.0, 100.0))
}

fn provider_label(provider: &str) -> String {
    match provider.trim().to_ascii_lowercase().as_str() {
        "codex" => "Codex".to_string(),
        "claude" => "Claude".to_string(),
        "cursor" => "Cursor".to_string(),
        value if value.is_empty() => "AI Limits".to_string(),
        _ => title_case(provider),
    }
}

fn limit_type_label(limit_name: &str) -> String {
    match limit_name.trim().to_ascii_lowercase().as_str() {
        "5h" | "five_hour" | "five hour" | "session" | "primary" => "5h".to_string(),
        "weekly" | "week" | "7d" | "seven_day" | "seven day" | "secondary" => "weekly".to_string(),
        "auto" => "auto".to_string(),
        "plan" | "total" => "plan".to_string(),
        "api" | "api_models" | "api models" => "api".to_string(),
        value if value.is_empty() => "limit".to_string(),
        value => value.replace('_', " "),
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

fn reset_label(value: Option<&str>) -> String {
    value
        .and_then(parse_reset)
        .map(format_reset)
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_reset(value: &str) -> Option<DateTime<Local>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Local));
    }

    if trimmed.chars().all(|character| character.is_ascii_digit()) {
        let seconds = trimmed.parse::<i64>().ok()?;
        return Utc
            .timestamp_opt(seconds, 0)
            .single()
            .map(|parsed| parsed.with_timezone(&Local));
    }

    None
}

fn format_reset(value: DateTime<Local>) -> String {
    format!(
        "{} {}",
        value.format("%Y-%m-%d %H:%M"),
        format_utc_offset(value.offset().local_minus_utc())
    )
}

fn format_utc_offset(offset_seconds: i32) -> String {
    let sign = if offset_seconds >= 0 { '+' } else { '-' };
    let absolute = offset_seconds.abs();
    let hours = absolute / 3600;
    let minutes = (absolute % 3600) / 60;

    if offset_seconds == 0 {
        "UTC".to_string()
    } else if minutes == 0 {
        format!("UTC{sign}{hours}")
    } else {
        format!("UTC{sign}{hours}:{minutes:02}")
    }
}

#[cfg(target_os = "macos")]
fn platform_notify(notification: &Notification) -> io::Result<()> {
    macos::notify(notification)
}

#[cfg(target_os = "windows")]
fn platform_notify(notification: &Notification) -> io::Result<()> {
    windows::notify(notification)
}

#[cfg(target_os = "linux")]
fn platform_notify(notification: &Notification) -> io::Result<()> {
    linux::notify(notification)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_notify(notification: &Notification) -> io::Result<()> {
    noop::notify(notification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AccountInfo, Source, SourceData, SourceStatus, UsageInfo};

    fn structured_with_limit(remaining_percent: Option<f64>) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_local".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: false,
            collected_at: None,
            data_as_of: None,
            account: AccountInfo::default(),
            limits: vec![LimitInfo {
                name: "5h".to_string(),
                remaining_percent,
                ..Default::default()
            }],
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn creates_notification_for_threshold_remaining_percent() {
        let notifications = notifications_for_structured(&structured_with_limit(Some(75.0)));

        assert_eq!(
            notifications,
            vec![Notification {
                dedupe_key: "Codex|codex_local|5h|75".to_string(),
                title: "🟢 AI Limits".to_string(),
                subtitle: "Codex 5h - 75% left".to_string(),
                message: "reset unknown".to_string(),
                color: NotificationColor::Green,
            }]
        );
    }

    #[test]
    fn creates_notification_when_remaining_is_below_threshold() {
        let notifications = notifications_for_structured(&structured_with_limit(Some(74.0)));

        assert_eq!(notifications[0].color, NotificationColor::Green);
        assert_eq!(notifications[0].title, "🟢 AI Limits");
        assert_eq!(notifications[0].subtitle, "Codex 5h - 74% left");
        assert_eq!(notifications[0].message, "reset unknown");
    }

    #[test]
    fn dedupe_key_uses_threshold_not_exact_remaining_percent() {
        let first = notifications_for_structured(&structured_with_limit(Some(75.0)));
        let second = notifications_for_structured(&structured_with_limit(Some(74.0)));

        assert_eq!(first[0].dedupe_key, second[0].dedupe_key);
    }

    #[test]
    fn ignores_remaining_above_first_threshold() {
        assert!(notifications_for_structured(&structured_with_limit(Some(76.0))).is_empty());
    }

    #[test]
    fn derives_remaining_percent_from_used_percent() {
        let mut info = structured_with_limit(None);
        info.limits[0].used_percent = Some(50.0);

        let notifications = notifications_for_structured(&info);

        assert_eq!(notifications[0].color, NotificationColor::Yellow);
        assert_eq!(notifications[0].title, "🟡 AI Limits");
        assert_eq!(notifications[0].subtitle, "Codex 5h - 50% left");
    }

    #[test]
    fn ignores_unavailable_data() {
        let mut info = structured_with_limit(Some(25.0));
        info.status.data_available = false;

        assert!(notifications_for_structured(&info).is_empty());
    }

    #[test]
    fn evaluates_source_report_structured_data() {
        let report = SourceReport {
            source: Source::CodexLocal,
            data: SourceData {
                raw: None,
                structured: structured_with_limit(Some(10.0)),
                stderr: String::new(),
            },
        };

        assert_eq!(
            notifications_for_report(&report)[0].color,
            NotificationColor::Red
        );
    }
}
