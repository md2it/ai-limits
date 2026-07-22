mod common;
mod limits;
mod time;
mod usage;

pub use common::{
    normalize_percent, remaining_percent_for_display, source_label_for_display,
    window_label_for_display, ColorConfig, ProviderBlock,
};
pub use limits::limits_block;
pub use time::{format_user_timestamp, TimeContext};
pub use usage::usage_block;

pub fn format_raw_output(data: &crate::types::SourceData) -> String {
    match data.raw.as_deref() {
        Some(raw) if !raw.trim().is_empty() => raw.to_string(),
        _ if !data.structured.raw_data_available => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
        _ => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
    }
}

pub fn format_structured_output(data: &crate::types::SourceData) -> String {
    serde_json::to_string_pretty(&data.structured)
        .unwrap_or_else(|error| format!("failed to serialize structured data: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AccountInfo, ActivityUsage, LimitInfo, ModelUsage, MoneyUsage, SourceStatus,
        StructuredSourceInfo, TokenUsage, UsageInfo,
    };

    fn sample_limits_info() -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_cli".to_string(),
            source_link: "docs/get-info/providers/codex.md".to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-06-30T21:41:00Z".to_string()),
            data_as_of: Some("Jul 3, 21:41 UTC-2".to_string()),
            account: AccountInfo {
                credits_remaining: Some(344.2),
                ..Default::default()
            },
            limits: vec![
                LimitInfo {
                    name: "5h limit".to_string(),
                    window_label: Some("5h".to_string()),
                    window_minutes: Some(300),
                    resets_at: Some("Jun 30, 21:41 UTC-2".to_string()),
                    used_percent: Some(92.0),
                    remaining_percent: Some(8.0),
                    ..Default::default()
                },
                LimitInfo {
                    name: "Weekly limit".to_string(),
                    window_label: Some("7d".to_string()),
                    window_minutes: Some(10080),
                    resets_at: Some("Jul 3, 21:41 UTC-2".to_string()),
                    used_percent: Some(46.0),
                    remaining_percent: Some(54.0),
                    ..Default::default()
                },
            ],
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    fn sample_usage_info() -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_local".to_string(),
            source_link: "docs/get-info/providers/codex.md".to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-06-30T21:41:00Z".to_string()),
            data_as_of: Some("Jul 3, 21:41 UTC-2".to_string()),
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo {
                tokens: TokenUsage {
                    input: Some(120_000),
                    cached_input: Some(80_000),
                    output: Some(30_000),
                    total: Some(230_000),
                    ..Default::default()
                },
                activity: ActivityUsage {
                    sessions_count: Some(14),
                    turns_count: Some(128),
                    latest_activity_at: Some("Jul 3, 21:41 UTC-2".to_string()),
                    ..Default::default()
                },
                models: ModelUsage {
                    top_model: Some("gpt-5".to_string()),
                },
                money: MoneyUsage {
                    used_amount: Some(12.4),
                    currency: Some("usd".to_string()),
                    ..Default::default()
                },
            },
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn limit_rows_align_reset_column() {
        let info = sample_limits_info();
        let block = limits_block(&info, &ColorConfig { enabled: false });
        let reset_positions = block
            .body
            .lines()
            .filter_map(|line| line.find(" | reset "))
            .collect::<Vec<_>>();

        assert_eq!(reset_positions.len(), 2);
        assert_eq!(reset_positions[0], reset_positions[1]);
    }

    #[test]
    fn pad_visible_right_ignores_ansi_color_codes() {
        let bar = format!("\x1b[32m{}\x1b[0m", common::visible_limit_bar(54.0));
        let padded = common::pad_visible_right(&bar, common::LIMIT_BAR_WIDTH);

        assert_eq!(common::visible_width(&padded), common::LIMIT_BAR_WIDTH);
    }

    #[test]
    fn limits_block_renders_rows_credits_and_data_as_of() {
        let block = limits_block(&sample_limits_info(), &ColorConfig { enabled: false });

        assert_eq!(block.provider_label, "CODEX");
        assert!(block.body.contains("5h   "));
        assert!(block.body.contains("8.0% left | reset Jun 30, 21:41"));
        assert!(block.body.contains("54.0% left | reset Jul 3, 21:41"));
        assert!(block.body.contains("Credits: 344.2"));
        assert!(block.body.contains("Source codex-cli: Jul 3, 21:41"));
    }

    #[test]
    fn limits_block_renders_available_reset_count_without_details() {
        let mut info = sample_limits_info();
        info.available_limit_resets = Some(2);

        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("Resets:  2"));
    }

    #[test]
    fn limits_block_hides_null_and_zero_reset_counts() {
        for available_limit_resets in [None, Some(0)] {
            let mut info = sample_limits_info();
            info.available_limit_resets = available_limit_resets;

            let block = limits_block(&info, &ColorConfig { enabled: false });

            assert!(!block.body.contains("Resets:"));
        }
    }

    #[test]
    fn limits_block_places_resets_after_credits_and_before_source() {
        let mut info = sample_limits_info();
        info.available_limit_resets = Some(1);

        let block = limits_block(&info, &ColorConfig { enabled: false });
        let credits = block.body.find("Credits: 344.2").expect("credits row");
        let resets = block.body.find("Resets:  1").expect("reset row");
        let source = block.body.find("Source codex-cli:").expect("source row");

        assert!(credits < resets);
        assert!(resets < source);
    }

    #[test]
    fn limits_block_renders_unavailable_state() {
        let info = StructuredSourceInfo {
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some("not logged in".to_string()),
            },
            data_as_of: None,
            ..sample_limits_info()
        };
        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("Unavailable: not logged in"));
        assert!(block.body.contains("Source codex-cli: unknown"));
    }

    #[test]
    fn limits_block_renders_no_limit_data_message() {
        let info = StructuredSourceInfo {
            limits: Vec::new(),
            account: AccountInfo::default(),
            ..sample_limits_info()
        };
        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block
            .body
            .contains("No usable limit records from this source"));
        assert!(block
            .body
            .contains("Other sources may still provide limit data."));
        assert!(block.body.contains("Source codex-cli: Jul 3, 21:41"));
    }

    #[test]
    fn usage_block_renders_available_rows() {
        let block = usage_block(&sample_usage_info());

        assert_eq!(block.provider_label, "CODEX");
        assert!(block.body.contains(
            "Tokens        input 120,000 | cached 80,000 | output 30,000 | total 230,000"
        ));
        assert!(block
            .body
            .contains("Activity      14 sessions | 128 turns | latest Jul 3, 21:41"));
        assert!(block.body.contains("Models        top: gpt-5"));
        assert!(block.body.contains("Money         $12.40 used"));
        assert!(block.body.contains("Source codex-local: Jul 3, 21:41"));
    }

    #[test]
    fn limit_bar_uses_twenty_five_characters_without_color() {
        let bar = common::visible_limit_bar(54.0);
        assert_eq!(bar.chars().count(), 25);
        assert_eq!(
            bar.chars().filter(|character| *character == '■').count(),
            14
        );
        assert_eq!(
            bar.chars().filter(|character| *character == '□').count(),
            11
        );
        assert!(!bar.contains('◧'));
    }

    #[test]
    fn limits_block_shows_one_decimal_place_for_fractional_percent() {
        let mut info = sample_limits_info();
        info.limits = vec![LimitInfo {
            name: "plan_usage".to_string(),
            window_label: Some("plan".to_string()),
            used_percent: Some(37.5),
            remaining_percent: Some(62.5),
            resets_at: Some("Jul 3, 21:41 UTC-2".to_string()),
            ..Default::default()
        }];

        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("62.5% left"));
    }

    #[test]
    fn format_percent_always_shows_one_decimal_place() {
        assert_eq!(common::format_percent(84.0), "84.0");
        assert_eq!(common::format_percent(62.5), "62.5");
        assert_eq!(common::format_percent(100.0), "100.0");
    }

    #[test]
    fn limits_block_formats_iso_data_as_of_for_display() {
        let mut info = sample_limits_info();
        info.collected_at = Some("2026-06-29T23:09:29Z".to_string());
        info.data_as_of = Some("2026-06-29T23:09:29Z".to_string());
        info.limits[0].resets_at = Some("2:20am (Asia/Nicosia)".to_string());

        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(!block.body.contains("2026-06-29T23:09:29Z"));
        assert!(block.body.contains("Source codex-cli:"));
        assert!(block.body.contains(" | reset 02:20"));
    }

    #[test]
    fn window_label_maps_claude_cli_session_and_week() {
        assert_eq!(
            common::window_label_for_display(&LimitInfo {
                name: "Current session".to_string(),
                window_label: Some("Current session".to_string()),
                window_minutes: Some(300),
                ..Default::default()
            }),
            "5h"
        );
        assert_eq!(
            common::window_label_for_display(&LimitInfo {
                name: "Current week".to_string(),
                window_label: Some("Current week".to_string()),
                window_minutes: Some(10080),
                ..Default::default()
            }),
            "7d"
        );
    }
}
