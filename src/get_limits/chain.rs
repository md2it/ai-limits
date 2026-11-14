use std::collections::HashMap;
use std::io;
use std::sync::{Mutex, OnceLock};

use crate::types::{AccountInfo, Source, SourceReport};

use super::freshness::STALE_LOCAL_DATA_MESSAGE;
use super::get_source_limits;

pub(super) fn get_fallback_chain_limits(sources: &[Source]) -> io::Result<SourceReport> {
    let mut cli_authorization_report = None;
    let mut last_report = None;
    let mut stale_local_report = None;
    let mut last_error = None;

    for source in sources {
        match get_source_limits(*source) {
            Ok(report) if has_usable_limit_data(&report) => {
                let mut report = report;
                backfill_account_from_cache(&mut report);
                return Ok(report);
            }
            Ok(report) => {
                if requires_cli_authorization(&report) {
                    cli_authorization_report = Some(report);
                } else if is_stale_local_report(&report) {
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

    if let Some(mut report) =
        preferred_unusable_report(cli_authorization_report, stale_local_report, last_report)
    {
        backfill_account_from_cache(&mut report);
        return Ok(report);
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "source fallback chain cannot be empty",
        )
    }))
}

fn preferred_unusable_report(
    cli_authorization_report: Option<SourceReport>,
    stale_local_report: Option<SourceReport>,
    last_report: Option<SourceReport>,
) -> Option<SourceReport> {
    cli_authorization_report
        .or(stale_local_report)
        .or(last_report)
}

fn requires_cli_authorization(report: &SourceReport) -> bool {
    report.data.structured.status.cli_authorization.is_some()
}

fn is_stale_local_report(report: &SourceReport) -> bool {
    matches!(report.source, Source::CodexLocal | Source::ClaudeLocal)
        && report.data.structured.status.message.as_deref() == Some(STALE_LOCAL_DATA_MESSAGE)
}

fn has_usable_limit_data(report: &SourceReport) -> bool {
    report.data.structured.status.access_available
        && report.data.structured.status.data_available
        && !report.data.structured.limits.is_empty()
}

#[derive(Clone, Default)]
struct CachedAccountFields {
    renewal_at: Option<String>,
    price_amount: Option<f64>,
    price_currency: Option<String>,
    price_period: Option<String>,
}

fn account_field_cache() -> &'static Mutex<HashMap<String, CachedAccountFields>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedAccountFields>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sources within the same chain disagree on which subscription fields they
/// populate (e.g. `codex_rpc` never reports `renewal_at`/`price_*`, while
/// `codex_local` does). Since the fallback chain can pick a different source
/// on every poll, this caused those fields to appear and disappear from the
/// rendered subscription section between polls. Cache the last known values
/// per provider and backfill gaps so the displayed fields stay stable.
fn backfill_account_from_cache(report: &mut SourceReport) {
    // Cursor sessions can change accounts; never backfill across their credential boundary.
    if report.source == Source::CursorApi2 {
        return;
    }
    let Ok(mut cache) = account_field_cache().lock() else {
        return;
    };

    let provider = report.data.structured.provider.clone();
    let account = &mut report.data.structured.account;

    if account.renewal_at.is_some() || account.price_amount.is_some() {
        cache.insert(provider, cached_fields_from(account));
        return;
    }

    if let Some(cached) = cache.get(&provider) {
        apply_cached_fields(account, cached);
    }
}

fn cached_fields_from(account: &AccountInfo) -> CachedAccountFields {
    CachedAccountFields {
        renewal_at: account.renewal_at.clone(),
        price_amount: account.price_amount,
        price_currency: account.price_currency.clone(),
        price_period: account.price_period.clone(),
    }
}

fn apply_cached_fields(account: &mut AccountInfo, cached: &CachedAccountFields) {
    if account.renewal_at.is_none() {
        account.renewal_at = cached.renewal_at.clone();
    }
    if account.price_amount.is_none() {
        account.price_amount = cached.price_amount;
        account.price_currency = cached.price_currency.clone();
        account.price_period = cached.price_period.clone();
    }
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
                        cli_authorization: None,
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
        assert!(has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            true,
            vec![Default::default()]
        )));
        assert!(!has_usable_limit_data(&report_for(
            Source::CodexLocal,
            false,
            true,
            vec![Default::default()]
        )));
        assert!(!has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            false,
            vec![Default::default()]
        )));
        assert!(!has_usable_limit_data(&report_for(
            Source::CodexLocal,
            true,
            true,
            Vec::new()
        )));
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
    fn cli_authorization_is_preferred_over_generic_fallback_failure() {
        let mut authorization_required = report_for(Source::ClaudeCli, false, false, Vec::new());
        authorization_required
            .data
            .structured
            .status
            .cli_authorization = Some(crate::types::CliAuthorization::Claude);
        let generic_no_data = report_for(Source::ClaudeLocal, true, false, Vec::new());

        let selected =
            preferred_unusable_report(Some(authorization_required), None, Some(generic_no_data))
                .expect("an unavailable report is selected");

        assert_eq!(selected.source, Source::ClaudeCli);
        assert!(requires_cli_authorization(&selected));
    }
}
