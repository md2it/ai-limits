use std::io;

use crate::providers::{
    claude_cli, claude_local, claude_statusline, codex_cli, codex_local, cursor_api2,
};
use crate::types::{Source, SourceData, SourceReport};

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

const DEFAULT_CODEX_CHAIN: &[Source] = &[Source::CodexLocal];
const DEFAULT_CLAUDE_CHAIN: &[Source] = &[Source::ClaudeStatusline, Source::ClaudeLocal];
const DEFAULT_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

const BEST_CODEX_CHAIN: &[Source] = &[Source::CodexLocal, Source::CodexCli];
const BEST_CLAUDE_CHAIN: &[Source] = &[
    Source::ClaudeStatusline,
    Source::ClaudeLocal,
    Source::ClaudeCli,
];
const BEST_CURSOR_CHAIN: &[Source] = &[Source::CursorApi2];

pub fn default_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: DEFAULT_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: DEFAULT_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: DEFAULT_CURSOR_CHAIN,
        },
    ]
}

pub fn best_source_plan() -> Vec<SourcePlan> {
    vec![
        SourcePlan::Chain {
            label: "codex",
            sources: BEST_CODEX_CHAIN,
        },
        SourcePlan::Chain {
            label: "claude",
            sources: BEST_CLAUDE_CHAIN,
        },
        SourcePlan::Chain {
            label: "cursor",
            sources: BEST_CURSOR_CHAIN,
        },
    ]
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
    let mut last_error = None;

    for source in sources {
        match get_source_limits(*source) {
            Ok(report) if has_usable_limit_data(&report) => return Ok(report),
            Ok(report) => {
                last_report = Some(report);
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
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

fn has_usable_limit_data(report: &SourceReport) -> bool {
    report.data.structured.status.access_available
        && report.data.structured.status.data_available
        && !report.data.structured.limits.is_empty()
}

pub fn get_source_limits(source: Source) -> io::Result<SourceReport> {
    let data = match source {
        Source::CodexLocal => codex_local::get_usage()?,
        Source::CodexCli => codex_cli::collect_usage()?,
        Source::ClaudeStatusline => claude_statusline::collect()?,
        Source::ClaudeCli => claude_cli::collect_usage()?,
        Source::ClaudeLocal => claude_local::collect()?,
        Source::CursorApi2 => cursor_api2::collect_usage()?,
    };

    Ok(SourceReport { source, data })
}

pub fn get_source_data(source: Source) -> io::Result<SourceData> {
    get_source_limits(source).map(|report| report.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SourceData, SourceStatus, StructuredSourceInfo};

    fn report(access_available: bool, data_available: bool, limits: usize) -> SourceReport {
        SourceReport {
            source: Source::CodexLocal,
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
                    limits: vec![Default::default(); limits],
                    usage: Default::default(),
                    diagnostics: Vec::new(),
                },
                stderr: String::new(),
            },
        }
    }

    #[test]
    fn usable_limit_data_requires_access_data_and_limit_records() {
        assert!(has_usable_limit_data(&report(true, true, 1)));
        assert!(!has_usable_limit_data(&report(false, true, 1)));
        assert!(!has_usable_limit_data(&report(true, false, 1)));
        assert!(!has_usable_limit_data(&report(true, true, 0)));
    }

    #[test]
    fn default_plan_uses_fast_free_provider_chains() {
        assert_eq!(
            default_source_plan(),
            vec![
                SourcePlan::Chain {
                    label: "codex",
                    sources: DEFAULT_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: DEFAULT_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: DEFAULT_CURSOR_CHAIN
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
                    sources: BEST_CODEX_CHAIN
                },
                SourcePlan::Chain {
                    label: "claude",
                    sources: BEST_CLAUDE_CHAIN
                },
                SourcePlan::Chain {
                    label: "cursor",
                    sources: BEST_CURSOR_CHAIN
                },
            ]
        );
    }
}
