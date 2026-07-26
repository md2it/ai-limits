#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    CodexLocal,
    CodexCli,
    ClaudeCli,
    ClaudeLocal,
    CursorApi2,
}

impl Source {
    pub const ALL: [Self; 5] = [
        Self::CodexLocal,
        Self::CodexCli,
        Self::ClaudeCli,
        Self::ClaudeLocal,
        Self::CursorApi2,
    ];

    pub const DEFAULTS: [Self; 3] = [Self::CodexLocal, Self::ClaudeLocal, Self::CursorApi2];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "codex_local" => Ok(Self::CodexLocal),
            "codex_cli" => Ok(Self::CodexCli),
            "claude_cli" => Ok(Self::ClaudeCli),
            "claude_local" => Ok(Self::ClaudeLocal),
            "cursor_api2" => Ok(Self::CursorApi2),
            _ => Err(format!(
                "unknown source `{value}`; expected one of: codex_local, codex_cli, claude_cli, claude_local, cursor_api2"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CodexLocal => "codex-local",
            Self::CodexCli => "codex-cli",
            Self::ClaudeCli => "claude-cli",
            Self::ClaudeLocal => "claude-local",
            Self::CursorApi2 => "cursor-api2",
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            Self::CodexLocal => "CODEX-LOCAL",
            Self::CodexCli => "CODEX-CLI",
            Self::ClaudeCli => "CLAUDE-CLI",
            Self::ClaudeLocal => "CLAUDE-LOCAL",
            Self::CursorApi2 => "CURSOR-API2",
        }
    }
}

pub struct ProviderRun {
    pub compacted_stdout: String,
    pub stderr: String,
}

pub struct SourceReport {
    pub source: Source,
    pub data: SourceData,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SourceData {
    pub raw: Option<String>,
    pub structured: StructuredSourceInfo,
    pub stderr: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StructuredSourceInfo {
    pub provider: String,
    pub source: String,
    pub source_link: String,
    pub status: SourceStatus,
    pub raw_data_available: bool,
    pub collected_at: Option<String>,
    pub data_as_of: Option<String>,
    pub account: AccountInfo,
    pub limits: Vec<LimitInfo>,
    pub available_limit_resets: Option<u64>,
    pub usage: UsageInfo,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SourceStatus {
    pub data_available: bool,
    pub access_available: bool,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct AccountInfo {
    pub plan: Option<String>,
    pub credits_total: Option<f64>,
    pub credits_used: Option<f64>,
    pub credits_remaining: Option<f64>,
}

pub type StructuredAccount = AccountInfo;

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct LimitInfo {
    pub name: String,
    pub window_label: Option<String>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<String>,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub used_amount: Option<f64>,
    pub remaining_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub amount_unit: Option<String>,
}

pub type StructuredLimit = LimitInfo;

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct UsageInfo {
    pub tokens: TokenUsage,
    pub money: MoneyUsage,
    pub activity: ActivityUsage,
    pub models: ModelUsage,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct TokenUsage {
    pub input: Option<u64>,
    pub cached_input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning_output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
    pub total: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct MoneyUsage {
    pub used_amount: Option<f64>,
    pub remaining_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct ActivityUsage {
    pub events_count: Option<u64>,
    pub files_count: Option<u64>,
    pub sessions_count: Option<u64>,
    pub turns_count: Option<u64>,
    pub latest_activity_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct ModelUsage {
    pub top_model: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_available_limit_resets_in_structured_source_info() {
        let info = StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_cli".to_string(),
            source_link: String::new(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: true,
            collected_at: None,
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            available_limit_resets: Some(2),
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(info).expect("structured data serializes");
        assert_eq!(value["available_limit_resets"], 2);
    }

    #[test]
    fn deserializes_optional_fields_when_absent_and_ignores_unknown_fields() {
        let value = serde_json::json!({
            "provider": "codex",
            "source": "codex_cli",
            "source_link": "docs/get-limits",
            "status": {
                "data_available": true,
                "access_available": true
            },
            "raw_data_available": false,
            "account": {},
            "limits": [{
                "name": "five_hour"
            }],
            "usage": {
                "tokens": {},
                "money": {},
                "activity": {},
                "models": {}
            },
            "diagnostics": [],
            "future_field": "ignored"
        });

        let info: StructuredSourceInfo =
            serde_json::from_value(value).expect("structured data deserializes");

        assert_eq!(info.status.message, None);
        assert_eq!(info.collected_at, None);
        assert_eq!(info.limits[0].remaining_percent, None);
        assert_eq!(info.available_limit_resets, None);
    }
}
