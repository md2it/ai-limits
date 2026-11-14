use std::io;

use super::credentials::Store;
use super::session::{with_session, Session};
use super::transport::{Failure, Transport};

use super::parse::{
    billing_window, is_enterprise_account, monthly_billing_window, plan_usage_is_present, team_id,
    UsageEventPages, PAGE_SIZE,
};

/// Hard ceiling on `GetFilteredUsageEvents` pages. Reaching it makes the page
/// set incomplete, which turns the session count into `null` rather than into
/// an understated number.
const MAX_EVENT_PAGES: u64 = 60;

pub(super) struct AccessDenied {
    pub message: String,
}

/// The body of one dashboard response, or a fixed diagnostic literal.
///
/// The error side is always a `&'static str`: no response body, HTTP status, or
/// `curl` message is ever carried out of this module.
pub type MethodOutcome = Result<String, &'static str>;

/// Every response the source collected in one run.
#[derive(Clone, Debug, Default)]
pub struct CursorResponses {
    pub plan_info: Option<MethodOutcome>,
    pub current_period: Option<MethodOutcome>,
    pub hard_limit: Option<MethodOutcome>,
    pub aggregated: Option<MethodOutcome>,
    pub filtered: UsageEventPages,
    /// Diagnostics produced while fetching. Fixed literals only.
    pub fetch_diagnostics: Vec<&'static str>,
}

pub(super) fn fetch_dashboard() -> io::Result<Result<CursorResponses, AccessDenied>> {
    Ok(
        with_session(fetch_with_session).map_err(|error| AccessDenied {
            message: error.message().to_string(),
        }),
    )
}

fn fetch_with_session<T: Transport, S: Store>(
    session: &mut Session<'_, T, S>,
) -> Result<CursorResponses, Failure> {
    let mut responses = CursorResponses::default();

    let plan_info = post(session, "GetPlanInfo", "{}")?;
    if plan_info.is_err() {
        return Err(Failure::Transient);
    }
    responses.plan_info = Some(plan_info);

    let current_period = post(session, "GetCurrentPeriodUsage", "{}")?;
    responses.current_period = Some(current_period);
    responses.hard_limit = Some(post(session, "GetHardLimit", "{}")?);

    let current_period_body = responses
        .current_period
        .as_ref()
        .and_then(|outcome| outcome.as_ref().ok())
        .cloned();

    let (window, team) = match current_period_body.as_deref() {
        Some(body) if plan_usage_is_present(body) => (billing_window(body), 0),
        Some(_) | None => team_window(session, &mut responses)?,
    };

    let Some((start, end)) = window else {
        responses
            .fetch_diagnostics
            .push("usage totals and activity: the billing cycle window is unknown");
        return Ok(responses);
    };

    let window_body = format!("{{\"teamId\":{team},\"startDate\":{start},\"endDate\":{end}}}");
    responses.aggregated = Some(post(session, "GetAggregatedUsageEvents", &window_body)?);

    fetch_event_pages(session, team, start, end, &mut responses.filtered)?;

    Ok(responses)
}

/// Team and enterprise accounts report no `planUsage`, so the window comes from
/// `GetMonthlyBillingCycle` and the aggregation is scoped to the real team.
///
/// This branch is derived from the official client only and has never run
/// against a live team account, so every step that does not answer degrades to
/// no window at all rather than to a guessed one.
fn team_window(
    session: &mut Session<'_, impl Transport, impl Store>,
    responses: &mut CursorResponses,
) -> Result<(Option<(i64, i64)>, i64), Failure> {
    responses
        .fetch_diagnostics
        .push("team or enterprise account: the team branch is unverified against a live account");

    let me = post(session, "GetMe", "{}")?;
    let Ok(me) = me else {
        responses
            .fetch_diagnostics
            .push("team account: the account response could not be read");
        return Ok((None, 0));
    };

    if is_enterprise_account(&me) != Some(true) && team_id(&me).is_none() {
        responses
            .fetch_diagnostics
            .push("team account: no team identity was reported");
        return Ok((None, 0));
    }

    let team = team_id(&me).unwrap_or(0);
    let cycle = post(session, "GetMonthlyBillingCycle", "{}")?;
    let window = cycle
        .as_ref()
        .ok()
        .and_then(|body| monthly_billing_window(body));

    if window.is_none() {
        responses
            .fetch_diagnostics
            .push("team account: the monthly billing cycle window could not be read");
    }

    Ok((window, team))
}

fn fetch_event_pages(
    session: &mut Session<'_, impl Transport, impl Store>,
    team: i64,
    start: i64,
    end: i64,
    pages: &mut UsageEventPages,
) -> Result<(), Failure> {
    for page in 1..=MAX_EVENT_PAGES {
        let body = format!(
            "{{\"teamId\":{team},\"startDate\":{start},\"endDate\":{end},\"page\":{page},\"pageSize\":{PAGE_SIZE}}}"
        );
        match post(session, "GetFilteredUsageEvents", &body)? {
            Ok(response) => {
                if !pages.add_page(&response) {
                    return Ok(());
                }
            }
            Err(_) => {
                pages.mark_failed();
                return Ok(());
            }
        }

        if pages.is_complete() {
            return Ok(());
        }
    }

    pages.mark_capped();
    Ok(())
}

fn post(
    session: &mut Session<'_, impl Transport, impl Store>,
    method: &str,
    body: &str,
) -> Result<MethodOutcome, Failure> {
    match session.post(method, body) {
        Ok(body) => Ok(Ok(body)),
        Err(
            error @ (Failure::Unauthorized
            | Failure::SignedOut
            | Failure::Missing
            | Failure::Choice
            | Failure::Forbidden
            | Failure::Protected),
        ) => Err(error),
        Err(_) => Ok(Err("request failed")),
    }
}
