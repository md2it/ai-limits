use std::io;

use crate::providers::{claude_cli, claude_local, codex_cli, codex_local, cursor_api2};
use crate::types::{Source, SourceData, SourceReport, StructuredSourceInfo};
use chrono::{DateTime, Duration, Utc};

const LOCAL_RESET_EXPIRY_GRACE_MINUTES: i64 = 2;
const STALE_LOCAL_DATA_MESSAGE: &str = "Local provider data is outdated";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourcePlan {
    Single(Source),
    Chain {
        label: &'static str,
        sources: &'static [Source],
    },
}

impl SourcePlan {
    pub fn label(self) -> &'static str {
        match self {
            Self::Single(source) => source.label(),
            Self::Chain { label, .. } => label,
        }
    }
}

const FAST_CODEX_CHAIN: &[Source] = &[Source::CodexLocal];
const FAST_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeLocal];
const FAST_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const CLI_FALLBACK_CODEX_CHAIN: &[Source] = &[Source::CodexLocal, Source::CodexCli];
const CLI_FALLBACK_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeLocal, Source::ClaudeCli];
const CLI_FALLBACK_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const CLI_FIRST_CODEX_CHAIN: &[Source] = &[Source::CodexCli, Source::CodexLocal];
const CLI_FIRST_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeCli, Source::ClaudeLocal];
const CLI_FIRST_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

pub fn default_source_plan() -> Vec<SourcePlan> {
    fast_free_source_plan()
}

pub fn best_source_plan() -> Vec<SourcePlan> {
    cli_fallback_source_plan()
}

pub fn fast_free_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: FAST_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: FAST_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: FAST_CURSOR_CHAIN,
        },
    ]
}

pub fn cli_fallback_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: CLI_FALLBACK_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: CLI_FALLBACK_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: CLI_FALLBACK_CURSOR_CHAIN,
        },
    ]
}

pub fn cli_first_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: CLI_FIRST_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: CLI_FIRST_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: CLI_FIRST_CURSOR_CHAIN,
        },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourcePriority {
    Fast,
    #[default]
    Full,
    Best,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSourcePlanOptions {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
    pub source_priority: SourcePriority,
}

impl Default for UiSourcePlanOptions {
    fn default() -> Self {
        Self {
            enabled_codex: true,
            enabled_claude: true,
            enabled_cursor: true,
            source_priority: SourcePriority::Full,
        }
    }
}

pub fn ui_source_plan(options: UiSourcePlanOptions) -> Vec<SourcePlan> {
    let plans = match options.source_priority {
        SourcePriority::Fast => fast_free_source_plan(),
        SourcePriority::Full => cli_fallback_source_plan(),
        SourcePriority::Best => cli_first_source_plan(),
    };

    plans
        .into_iter()
        .filter(|plan| match plan.label() {
            "codex" => options.enabled_codex,
            "claude" => options.enabled_claude,
            "cursor" => options.enabled_cursor,
            _ => false,
        })
        .collect()
}

pub fn source_list_plan(sources: Vec<Source>) -> Vec<SourcePlan> {
    sources.into_iter().map(SourcePlan::Single).collect()
}

pub fn get_limits(sources: &[Source]) -> io::Result<Vec<SourceReport>> {
    sources
        .iter()
        .map(|source| get_source_limits(*source))
        .collect()
}

pub fn get_source_plan_limits(plan: SourcePlan) -> io::Result<SourceReport> {
    match plan {
        SourcePlan::Single(source) => get_source_limits(source),
        SourcePlan::Chain { sources, .. } => get_fallback_chain_limits(sources),
    }
}

fn get_fallback_chain_limits(sources: &[Source]) -> io::Result<SourceReport> {
    let mut last_report = None;
    let mut stale_local_report = None;
    let mut last_error = None;

    for source in sources {
        match get_source_limits(*source) {
            Ok(report) if report_has_usable_limit_data(&report) => return Ok(report),
            Ok(report) => {
                if is_stale_local_report(&report) {
                    stale_local_report = Some(report);
                } else {
                    last_report = Some(report);
                }
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    if let Some(report) = stale_local_report {
        return Ok(report);
    }

    if let Some(report) = last_report {
        return Ok(report);
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "source fallback chain cannot be empty",
        )
    }))
}

fn is_stale_local_report(report: &SourceReport) -> bool {
    matches!(report.source, Source::CodexLocal | Source::ClaudeLocal)
        && report.data.structured.status.message.as_deref() == Some(STALE_LOCAL_DATA_MESSAGE)
}

pub fn has_usable_limit_data(info: &StructuredSourceInfo) -> bool {
    info.status.access_available && info.status.data_available && !info.limits.is_empty()
}

fn report_has_usable_limit_data(report: &SourceReport) -> bool {
    has_usable_limit_data(&report.data.structured)
}

pub fn get_source_limits(source: Source) -> io::Result<SourceReport> {
    let data = match source {
        Source::CodexLocal => codex_local::get_usage()?,
        Source::CodexCli => codex_cli::collect_usage()?,
        Source::ClaudeCli => claude_cli::collect_usage()?,
        Source::ClaudeLocal => claude_local::collect()?,
        Source::CursorApi2 => cursor_api2::collect_usage()?,
    };

    Ok(mark_expired_local_limit_data(
        SourceReport { source, data },
        Utc::now(),
    ))
}

pub fn get_source_data(source: Source) -> io::Result<SourceData> {
    get_source_limits(source).map(|report| report.data)
}

fn mark_expired_local_limit_data(mut report: SourceReport, now: DateTime<Utc>) -> SourceReport {
    if !matches!(report.source, Source::CodexLocal | Source::ClaudeLocal) {
        return report;
    }

    let expiry_cutoff = now - Duration::minutes(LOCAL_RESET_EXPIRY_GRACE_MINUTES);
    let has_expired_reset = report.data.structured.limits.iter().any(|limit| {
        limit
            .resets_at
            .as_deref()
            .and_then(parse_absolute_reset)
            .is_some_and(|reset| reset < expiry_cutoff)
    });
    if !has_expired_reset {
        return report;
    }

    report.data.structured.status.data_available = false;
    report.data.structured.status.message = Some(STALE_LOCAL_DATA_MESSAGE.to_string());
    report.data.structured.limits.clear();
    report.data.structured.available_limit_resets = None;
    report.data.structured.diagnostics.push(
        "local limit snapshot rejected because an automatic reset time is in the past".to_string(),
    );
    report
}

fn parse_absolute_reset(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LimitInfo, SourceData, SourceStatus, StructuredSourceInfo};

    fn report_for(
        source: Source,
        access_available: bool,
        data_available: bool,
        limits: Vec<LimitInfo>,
    ) -> SourceReport {
        SourceReport {
            source,
            data: SourceData {
                raw: None,
                structured: StructuredSourceInfo {
                    provider: "codex".to_string(),
                    source: "codex_local".to_string(),
                    source_link: String::new(),
                    status: SourceStatus {
                        access_available,
                        data_available,
                        message: None,
                    },
                    raw_data_available: false,
                    collected_at: None,
                    data_as_of: None,
                    account: Default::default(),
                    limits,
                    available_limit_resets: None,
                    usage: Default::default(),
                    diagnostics: Vec::new(),
                },
                stderr: String::new(),
            },
        }
    }

    #[test]
    fn usable_limit_data_requires_access_data_and_limit_records() {
        assert!(report_has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            true,
            vec![Default::default()]
        )));
        assert!(!report_has_usable_limit_data(&report_for(
            Source::CodexLocal,
            false,
            true,
            vec![Default::default()]
        )));
        assert!(!report_has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            false,
            vec![Default::default()]
        )));
        assert!(!report_has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            true,
            Vec::new()
        )));
    }

    #[test]
    fn expired_codex_local_reset_rejects_the_whole_limit_snapshot() {
        let report = report_for(
            Source::CodexLocal,
            true,
            true,
            vec![
                LimitInfo {
                    resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                    ..Default::default()
                },
                LimitInfo {
                    resets_at: Some("2026-08-01T09:00:00Z".to_string()),
                    ..Default::default()
                },
            ],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:03:00Z".parse().expect("valid timestamp"),
        );

        assert!(!result.data.structured.status.data_available);
        assert_eq!(
            result.data.structured.status.message.as_deref(),
            Some(STALE_LOCAL_DATA_MESSAGE)
        );
        assert!(result.data.structured.limits.is_empty());
        assert!(!report_has_usable_limit_data(&result));
    }

    #[test]
    fn expired_claude_local_reset_is_rejected_even_if_provider_reconstruction_returns_it() {
        let report = report_for(
            Source::ClaudeLocal,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:03:00Z".parse().expect("valid timestamp"),
        );

        assert!(!result.data.structured.status.data_available);
        assert_eq!(
            result.data.structured.status.message.as_deref(),
            Some(STALE_LOCAL_DATA_MESSAGE)
        );
        assert!(result.data.structured.limits.is_empty());
    }

    #[test]
    fn local_reset_within_clock_grace_remains_usable() {
        let report = report_for(
            Source::ClaudeLocal,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T09:01:59Z".parse().expect("valid timestamp"),
        );

        assert!(report_has_usable_limit_data(&result));
    }

    #[test]
    fn non_local_sources_are_not_rejected_by_local_freshness_rule() {
        let report = report_for(
            Source::CodexCli,
            true,
            true,
            vec![LimitInfo {
                resets_at: Some("2026-07-26T09:00:00Z".to_string()),
                ..Default::default()
            }],
        );

        let result = mark_expired_local_limit_data(
            report,
            "2026-07-26T10:00:00Z".parse().expect("valid timestamp"),
        );

        assert!(report_has_usable_limit_data(&result));
    }

    #[test]
    fn stale_local_report_has_explicit_fallback_failure_priority() {
        let mut stale = report_for(Source::ClaudeLocal, true, false, Vec::new());
        stale.data.structured.status.message = Some(STALE_LOCAL_DATA_MESSAGE.to_string());
        let unavailable_cli = report_for(Source::ClaudeCli, true, false, Vec::new());

        assert!(is_stale_local_report(&stale));
        assert!(!is_stale_local_report(&unavailable_cli));
    }

    #[test]
    fn default_plan_uses_fast_free_provider_chains() {
        assert_eq!(
            default_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: FAST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: FAST_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: FAST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn best_plan_adds_cli_fallbacks_for_codex_and_claude() {
        assert_eq!(
            best_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FALLBACK_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FALLBACK_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: CLI_FALLBACK_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn cli_first_plan_prefers_cli_for_codex_and_claude() {
        assert_eq!(
            cli_first_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FIRST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FIRST_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: CLI_FIRST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_defaults_to_full_priority() {
        assert_eq!(
            UiSourcePlanOptions::default().source_priority,
            SourcePriority::Full
        );
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions::default()),
            cli_fallback_source_plan()
        );
    }

    #[test]
    fn ui_source_plan_filters_disabled_providers() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: false,
                enabled_cursor: true,
                source_priority: SourcePriority::Fast,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: FAST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: FAST_CURSOR_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_uses_cli_fallback_chains_for_full_priority() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: true,
                enabled_cursor: false,
                source_priority: SourcePriority::Full,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FALLBACK_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FALLBACK_CLAUDE_CHAIN
                },
            ]
        );
    }

    #[test]
    fn ui_source_plan_uses_cli_first_chains_for_best_priority() {
        assert_eq!(
            ui_source_plan(UiSourcePlanOptions {
                enabled_codex: true,
                enabled_claude: true,
                enabled_cursor: false,
                source_priority: SourcePriority::Best,
            }),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: CLI_FIRST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: CLI_FIRST_CLAUDE_CHAIN
                },
            ]
        );
    }
}
