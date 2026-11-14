use ai_limits::get_limits::UiSourcePlanOptions;
use ai_limits::presentation::{
    format_user_timestamp, is_limit_shown_to_user, normalize_percent, plan_display_lines,
    remaining_percent_for_display, source_label_for_display, window_label_for_desktop, TimeContext,
};
use ai_limits::types::{AccountInfo, StructuredSourceInfo};

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitsQuery {
    pub enabled_codex: bool,
    pub enabled_claude: bool,
    pub enabled_cursor: bool,
    pub notifications_enabled: bool,
}

impl Default for ProviderLimitsQuery {
    fn default() -> Self {
        let defaults = UiSourcePlanOptions::default();
        Self {
            enabled_codex: defaults.enabled_codex,
            enabled_claude: defaults.enabled_claude,
            enabled_cursor: defaults.enabled_cursor,
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
    /// Raw ISO-8601 instant of the actual collection this snapshot came from
    /// (`StructuredSourceInfo::collected_at`), unformatted — unlike
    /// `data_timestamp`, which is pre-formatted for display. Frontend refresh
    /// scheduling parses this to compute the next update time from the
    /// shared collection instant rather than a per-window "just fetched"
    /// clock reading; see docs/desktop/ui/refresh.md.
    collected_at: Option<String>,
    limits: Vec<ProviderLimitRow>,
    credits_remaining: Option<f64>,
    available_limit_resets: Option<u64>,
    plan: ProviderPlan,
    error_message: Option<String>,
    no_fresh_data: bool,
    authorization_required: Option<String>,
    pub(super) cursor_epoch: Option<u64>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimitRow {
    label: String,
    remaining_percentage: f64,
    reset_time: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPlan {
    lines: Vec<String>,
    links: Vec<ProviderLink>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLink {
    label: String,
    url: String,
}

pub(super) fn provider_limits_from_structured(
    id: &str,
    info: &StructuredSourceInfo,
) -> ProviderLimits {
    let time_context = TimeContext::from_structured(info);
    let limits: Vec<ProviderLimitRow> = info
        .limits
        .iter()
        .filter(|limit| is_limit_shown_to_user(limit))
        .filter_map(|limit| {
            let remaining = normalize_percent(remaining_percent_for_display(limit)?);
            Some(ProviderLimitRow {
                label: window_label_for_desktop(limit),
                remaining_percentage: remaining,
                reset_time: limit
                    .resets_at
                    .as_deref()
                    .map(|value| format_user_timestamp(value, &time_context)),
            })
        })
        .collect();

    let no_fresh_data =
        info.status.access_available && limits.is_empty() && info.available_limit_resets.is_none();

    let authorization_required = info
        .status
        .cli_authorization
        .map(|auth| auth.provider_id().to_string());

    let error_message = if authorization_required.is_some()
        || (info.status.access_available && info.status.data_available)
    {
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
        collected_at: info.collected_at.clone(),
        data_timestamp: Some(
            info.data_as_of
                .as_deref()
                .map(|value| format_user_timestamp(value, &time_context))
                .unwrap_or_else(|| "unknown".to_string()),
        ),
        limits,
        credits_remaining: info.account.credits_remaining,
        available_limit_resets: info.available_limit_resets,
        plan: ProviderPlan {
            lines: plan_display_lines(&info.account, &time_context),
            links: plan_links(&info.account),
        },
        error_message,
        no_fresh_data,
        authorization_required,
        cursor_epoch: (id == "cursor").then(ai_limits::providers::cursor_api2::cache_epoch),
    }
}

pub(super) fn provider_error(id: &str, message: String) -> ProviderLimits {
    ProviderLimits {
        id: id.to_string(),
        label: provider_label(id),
        source_id: None,
        collected_at: None,
        data_timestamp: None,
        limits: Vec::new(),
        credits_remaining: None,
        available_limit_resets: None,
        plan: ProviderPlan {
            lines: Vec::new(),
            links: Vec::new(),
        },
        error_message: Some(message),
        no_fresh_data: false,
        authorization_required: None,
        cursor_epoch: (id == "cursor").then(ai_limits::providers::cursor_api2::cache_epoch),
    }
}

pub(super) fn cursor_failure(
    message: String,
    previous: Option<&StructuredSourceInfo>,
) -> ProviderLimits {
    if ai_limits::providers::cursor_api2::retain_snapshot(&message) {
        if let Some(previous) = previous {
            let mut payload = provider_limits_from_structured("cursor", previous);
            payload.error_message = Some(if message == "Could not update Cursor. Try again." {
                "Could not update Cursor. Showing previous data.".into()
            } else {
                message
            });
            return payload;
        }
    }
    provider_error("cursor", message)
}

fn plan_links(account: &AccountInfo) -> Vec<ProviderLink> {
    [
        (account.plan_management_url.as_deref(), "Manage"),
        (account.billing_management_url.as_deref(), "Billing"),
    ]
    .into_iter()
    .filter_map(|(url, label)| {
        url.map(|url| ProviderLink {
            label: label.to_string(),
            url: url.to_string(),
        })
    })
    .collect()
}

fn provider_label(id: &str) -> String {
    let mut characters = id.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_limits::types::{CliAuthorization, LimitInfo, SourceStatus, UsageInfo};

    fn structured_with_resets(available_limit_resets: Option<u64>) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_cli".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
                cli_authorization: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-07-01T12:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    fn structured_cli_authorization(auth: CliAuthorization) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: auth.provider_id().to_string(),
            source: format!("{}_cli", auth.provider_id()),
            source_link: String::new(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some("technical auth message".to_string()),
                cli_authorization: Some(auth),
            },
            raw_data_available: true,
            collected_at: Some("2026-07-01T12:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn projects_null_available_limit_resets_and_marks_no_fresh_data() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(None));

        assert!(response.available_limit_resets.is_none());
        assert!(response.no_fresh_data);
        assert!(response.authorization_required.is_none());
    }

    #[test]
    fn projects_zero_available_limit_resets_as_known_limit_data() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(Some(0)));

        assert_eq!(response.available_limit_resets, Some(0));
        assert!(!response.no_fresh_data);
    }

    #[test]
    fn projects_known_positive_available_limit_resets() {
        let response = provider_limits_from_structured("codex", &structured_with_resets(Some(1)));

        assert_eq!(response.available_limit_resets, Some(1));
        assert!(!response.no_fresh_data);
    }

    #[test]
    fn projects_codex_cli_authorization_without_error_message() {
        let response = provider_limits_from_structured(
            "codex",
            &structured_cli_authorization(CliAuthorization::Codex),
        );

        assert_eq!(response.authorization_required.as_deref(), Some("codex"));
        assert!(response.error_message.is_none());
        assert!(!response.no_fresh_data);
        assert!(response.limits.is_empty());
    }

    #[test]
    fn projects_claude_cli_authorization_without_error_message() {
        let response = provider_limits_from_structured(
            "claude",
            &structured_cli_authorization(CliAuthorization::Claude),
        );

        assert_eq!(response.authorization_required.as_deref(), Some("claude"));
        assert!(response.error_message.is_none());
        assert!(!response.no_fresh_data);
    }

    fn structured_source(
        provider: &str,
        source: &str,
        account: AccountInfo,
    ) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: provider.to_string(),
            source: source.to_string(),
            account,
            ..structured_with_resets(None)
        }
    }

    fn serialized(response: &ProviderLimits) -> serde_json::Value {
        serde_json::to_value(response).expect("response should serialize")
    }

    fn codex_local() -> StructuredSourceInfo {
        structured_source(
            "codex",
            "codex_local",
            AccountInfo {
                plan: Some("plus".to_string()),
                credits_remaining: Some(38.6355075),
                ..Default::default()
            },
        )
    }

    #[test]
    fn projects_codex_local_plan() {
        let response = provider_limits_from_structured("codex", &codex_local());

        assert_eq!(response.plan.lines, vec!["Plus".to_string()]);
        assert!(response.plan.links.is_empty());
    }

    #[test]
    fn keeps_credits_remaining_out_of_the_plan_section() {
        let response = provider_limits_from_structured("codex", &codex_local());
        let json = serialized(&response);

        assert_eq!(json["creditsRemaining"], serde_json::json!(38.6355075));
        assert!(!response.plan.lines.iter().any(|line| line.contains("38.6")));
    }

    #[test]
    fn projects_codex_cli_without_plan_lines() {
        let info = structured_source(
            "codex",
            "codex_cli",
            AccountInfo {
                credits_remaining: Some(39.0),
                ..Default::default()
            },
        );
        let response = provider_limits_from_structured("codex", &info);

        assert!(response.plan.lines.is_empty());
        assert!(response.plan.links.is_empty());
        assert_eq!(response.credits_remaining, Some(39.0));
    }

    #[test]
    fn projects_cursor_pools_with_full_labels_and_hides_plan_and_included() {
        let mut info = structured_source("cursor", "cursor_api2", AccountInfo::default());
        info.status.data_available = true;
        info.limits = vec![
            LimitInfo {
                name: "plan_usage".to_string(),
                remaining_percent: Some(50.0),
                ..Default::default()
            },
            LimitInfo {
                name: "auto".to_string(),
                remaining_percent: Some(80.0),
                ..Default::default()
            },
            LimitInfo {
                name: "api_models".to_string(),
                remaining_percent: Some(20.0),
                ..Default::default()
            },
            LimitInfo {
                name: "included_spend".to_string(),
                remaining_percent: Some(40.0),
                ..Default::default()
            },
        ];
        let response = provider_limits_from_structured("cursor", &info);

        assert_eq!(response.limits.len(), 2);
        assert_eq!(response.limits[0].label, "Cursor Models");
        assert_eq!(response.limits[1].label, "Other Models");
        assert!(!response.no_fresh_data);
    }

    #[test]
    fn projects_plan_links_from_management_urls() {
        let info = structured_source(
            "codex",
            "codex_local",
            AccountInfo {
                plan: Some("pro".to_string()),
                plan_management_url: Some("https://example.test/plan".to_string()),
                billing_management_url: Some("https://example.test/billing".to_string()),
                ..Default::default()
            },
        );
        let json = serialized(&provider_limits_from_structured("codex", &info));

        assert_eq!(
            json["plan"]["links"],
            serde_json::json!([
                { "label": "Manage", "url": "https://example.test/plan" },
                { "label": "Billing", "url": "https://example.test/billing" },
            ])
        );
    }

    #[test]
    fn omits_missing_management_urls_from_plan_links() {
        let info = structured_source(
            "codex",
            "codex_local",
            AccountInfo {
                billing_management_url: Some("https://example.test/billing".to_string()),
                ..Default::default()
            },
        );
        let response = provider_limits_from_structured("codex", &info);

        assert_eq!(response.plan.links.len(), 1);
        assert_eq!(response.plan.links[0].label, "Billing");
    }

    #[test]
    fn serializes_empty_plan_object_instead_of_null() {
        let json = serialized(&provider_limits_from_structured(
            "codex",
            &structured_with_resets(None),
        ));

        assert_eq!(
            json["plan"],
            serde_json::json!({ "lines": [], "links": [] })
        );
    }

    #[test]
    fn provider_error_emits_empty_plan_object() {
        let json = serialized(&provider_error("codex", "boom".to_string()));

        assert_eq!(
            json["plan"],
            serde_json::json!({ "lines": [], "links": [] })
        );
    }

    #[test]
    fn serializes_plan_price_and_renewal_as_two_display_lines() {
        let info = structured_source(
            "codex",
            "codex_local",
            AccountInfo {
                plan: Some("plus".to_string()),
                subscription_started_at: Some("2024-01-12T12:00:00Z".to_string()),
                renewal_at: Some("2026-08-28T12:00:00Z".to_string()),
                price_amount: Some(20.0),
                price_currency: Some("USD".to_string()),
                price_period: Some("mo".to_string()),
                price_note: Some("may vary by country/currency".to_string()),
                ..Default::default()
            },
        );
        let response = provider_limits_from_structured("codex", &info);

        assert_eq!(response.plan.lines.len(), 2);
        assert_eq!(response.plan.lines[0], "Plus \u{2248} $20.00 /mo");
        assert!(response.plan.lines[1].starts_with("renews "));
        assert!(response.plan.lines[1].contains("2026"));
        assert!(!response.plan.lines[1].contains("2024"));
        assert!(!response.plan.lines[1].contains(':'));
    }
    #[test]
    fn cursor_transient_failure_keeps_snapshot_and_timestamp() {
        let mut previous = structured_source("cursor", "cursor_api2", AccountInfo::default());
        previous.data_as_of = Some("2026-10-28T10:00:00Z".into());
        previous.collected_at = previous.data_as_of.clone();
        previous.limits.push(ai_limits::types::LimitInfo {
            name: "auto".into(),
            remaining_percent: Some(70.0),
            ..Default::default()
        });
        let payload = cursor_failure(
            "Could not update Cursor. Try again.".into(),
            Some(&previous),
        );
        assert_eq!(payload.limits.len(), 1);
        assert_eq!(payload.collected_at, previous.collected_at);
        assert_eq!(
            payload.error_message.as_deref(),
            Some("Could not update Cursor. Showing previous data.")
        );
        let terminal = cursor_failure("Sign in to Cursor, then retry.".into(), Some(&previous));
        assert!(terminal.limits.is_empty());
        assert!(terminal.collected_at.is_none());
    }
}
