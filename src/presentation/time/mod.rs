mod format;
mod parse;

use chrono::{DateTime, Local};

use crate::types::StructuredSourceInfo;

use format::strip_display_timezone_suffix;
use parse::{parse_instant_reference, parse_to_local};

pub use format::{format_local_date, format_local_datetime};

pub struct TimeContext {
    reference: DateTime<Local>,
}

impl TimeContext {
    pub fn from_structured(info: &StructuredSourceInfo) -> Self {
        Self {
            reference: info
                .collected_at
                .as_deref()
                .and_then(parse_instant_reference)
                .or_else(|| info.data_as_of.as_deref().and_then(parse_instant_reference))
                .unwrap_or_else(Local::now),
        }
    }
}

pub fn format_user_timestamp(value: &str, context: &TimeContext) -> String {
    parse_to_local(value, context)
        .map(|parsed| format_local_datetime(parsed, context.reference))
        .unwrap_or_else(|| strip_display_timezone_suffix(value))
}

pub fn format_user_date(value: &str, context: &TimeContext) -> String {
    parse_to_local(value, context)
        .map(format_local_date)
        .unwrap_or_else(|| strip_display_timezone_suffix(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parse::parse_iso_or_unix;

    fn fixed_context(iso: &str) -> TimeContext {
        TimeContext {
            reference: parse_iso_or_unix(iso).expect("reference should parse"),
        }
    }

    fn expected_local_label(iso: &str, context: &TimeContext) -> String {
        format_local_datetime(
            parse_iso_or_unix(iso).expect("timestamp should parse"),
            context.reference,
        )
    }

    fn expected_local_date(iso: &str) -> String {
        format_local_date(parse_iso_or_unix(iso).expect("timestamp should parse"))
    }

    #[test]
    fn date_only_format_keeps_the_year_for_a_past_year() {
        let context = fixed_context("2026-08-02T12:00:00Z");
        let formatted = format_user_date("2024-01-12T12:00:00Z", &context);

        assert_eq!(formatted, expected_local_date("2024-01-12T12:00:00Z"));
        assert!(formatted.contains("2024"));
        assert!(!formatted.contains("2026"));
        assert!(!formatted.contains(':'));
    }

    #[test]
    fn date_only_format_keeps_the_year_within_the_current_year() {
        let context = fixed_context("2026-08-02T12:00:00Z");
        let formatted = format_user_date("2026-08-28T12:00:00Z", &context);

        assert_eq!(formatted, expected_local_date("2026-08-28T12:00:00Z"));
        assert!(formatted.contains("2026"));
        assert!(!formatted.contains(':'));
    }

    #[test]
    fn date_only_format_keeps_the_date_for_today_instead_of_the_time() {
        let context = fixed_context("2026-08-02T12:00:00Z");
        let formatted = format_user_date("2026-08-02T12:30:00Z", &context);

        assert_eq!(formatted, expected_local_date("2026-08-02T12:30:00Z"));
        assert!(formatted.contains("2026"));
        assert!(!formatted.contains(':'));
        assert_ne!(
            formatted,
            format_user_timestamp("2026-08-02T12:30:00Z", &context)
        );
    }

    #[test]
    fn date_only_format_degrades_like_timestamps_for_unparseable_input() {
        let context = fixed_context("2026-08-02T12:00:00Z");

        assert_eq!(
            format_user_date("whenever it renews (Asia/Nicosia)", &context),
            "whenever it renews"
        );
        assert_eq!(format_user_date("not a date", &context), "not a date");
    }

    #[test]
    fn parses_iso_utc_timestamp() {
        assert!(parse_iso_or_unix("2026-06-29T23:09:29Z").is_some());
    }

    #[test]
    fn formats_iso_utc_in_local_style() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("2026-06-29T20:09:29Z", &context);

        assert_eq!(
            formatted,
            expected_local_label("2026-06-29T20:09:29Z", &context)
        );
        assert!(!formatted.contains("T20:"));
        assert!(!formatted.ends_with('Z'));
    }

    #[test]
    fn formats_claude_cli_session_reset_with_source_timezone() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("2:20am (Asia/Nicosia)", &context);

        assert_eq!(
            formatted,
            expected_local_label("2026-06-29T23:20:00Z", &context)
        );
        assert!(!formatted.contains("UTC"));
    }

    #[test]
    fn formats_claude_cli_week_reset_with_date() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("Jun 30 at 1pm (Asia/Nicosia)", &context);

        assert_eq!(
            formatted,
            expected_local_label("2026-06-30T10:00:00Z", &context)
        );
        assert!(!formatted.contains("UTC"));
    }

    #[test]
    fn formats_codex_cli_weekly_reset_pattern() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("02:59 on 6 Jul", &context);

        assert!(formatted.starts_with("Jul 6, "));
        assert!(formatted.contains("02:59"));
        assert!(!formatted.contains("UTC"));
    }

    #[test]
    fn formats_today_with_explicit_label() {
        let context = fixed_context("2026-06-30T20:00:00Z");
        let formatted = format_user_timestamp("2026-06-30T20:41:00Z", &context);

        assert_eq!(
            formatted,
            expected_local_label("2026-06-30T20:41:00Z", &context)
        );
        assert!(formatted.starts_with("today "));
    }

    #[test]
    fn fallback_strips_timezone_suffixes() {
        let context = fixed_context("2026-06-29T20:00:00Z");

        assert_eq!(
            format_user_timestamp("Jul 3, 21:41 UTC-2", &context),
            "Jul 3, 21:41"
        );
        assert_eq!(
            format_user_timestamp("unknown (Asia/Nicosia)", &context),
            "unknown"
        );
    }

    #[test]
    fn formatted_timestamps_do_not_pad_single_digit_day() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let single_digit_day = format_user_timestamp("02:59 on 6 Jul", &context);

        assert!(single_digit_day.contains("Jul 6, "));
        assert!(!single_digit_day.contains("Jul  6, "));
    }
}
