use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};

use crate::infra::os_access::claude_local_roots;
use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, ModelUsage, MoneyUsage, SourceData, SourceStatus,
    StructuredSourceInfo, TokenUsage, UsageInfo,
};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_local";
const SOURCE_LINK: &str = "docs/get-info";
const CLAUDE_LOCAL_MAX5_TOKEN_LIMIT: u64 = 88_000;
const CLAUDE_LOCAL_SESSION_WINDOW_MINUTES: u64 = 5 * 60;

#[derive(Default)]
struct ClaudeLocalUsage {
    files: usize,
    sessions: HashSet<String>,
    turns: usize,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    latest_timestamp: Option<String>,
    models: HashMap<String, u64>,
    turns_by_time: Vec<TurnUsage>,
    latest_server_reset_anchor: Option<ServerResetAnchor>,
}

pub fn collect() -> io::Result<SourceData> {
    let candidate_roots = default_roots()?;
    let scanned_roots = candidate_roots
        .iter()
        .filter(|root| root.is_dir())
        .cloned()
        .collect::<Vec<_>>();

    if scanned_roots.is_empty() {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, None)?),
            structured: structured_no_roots(),
            stderr: String::new(),
        });
    }

    let mut usage = ClaudeLocalUsage::default();

    for root in &scanned_roots {
        scan_root(root, &mut usage)?;
    }

    if usage.turns == 0 {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
            structured: structured_no_usage(scanned_roots.len()),
            stderr: String::new(),
        });
    }

    Ok(SourceData {
        raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
        structured: structured_from_usage(&usage),
        stderr: String::new(),
    })
}

fn default_roots() -> io::Result<Vec<PathBuf>> {
    claude_local_roots()
}

fn scan_root(root: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_root(&path, usage)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            scan_jsonl_file(&path, usage)?;
        }
    }

    Ok(())
}

fn scan_jsonl_file(path: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut seen_messages = HashMap::<String, TurnUsage>::new();
    let mut turns_without_id = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(anchor) = extract_server_reset_anchor(&record) {
            if usage
                .latest_server_reset_anchor
                .as_ref()
                .is_none_or(|current| anchor > *current)
            {
                usage.latest_server_reset_anchor = Some(anchor);
            }
        }

        let Some(turn) = extract_turn_usage(&record) else {
            continue;
        };

        if let Some(message_id) = turn.message_id.clone().filter(|value| !value.is_empty()) {
            seen_messages.insert(message_id, turn);
        } else {
            turns_without_id.push(turn);
        }
    }

    let turn_count = seen_messages.len() + turns_without_id.len();
    if turn_count > 0 {
        usage.files += 1;
    }

    for turn in seen_messages.into_values().chain(turns_without_id) {
        usage.sessions.insert(turn.session_id.clone());
        usage.turns += 1;
        usage.input_tokens += turn.input_tokens;
        usage.output_tokens += turn.output_tokens;
        usage.cache_read_tokens += turn.cache_read_tokens;
        usage.cache_creation_tokens += turn.cache_creation_tokens;

        if let Some(model) = turn.model.as_ref().filter(|value| !value.is_empty()) {
            *usage.models.entry(model.clone()).or_default() += 1;
        }

        if let Some(timestamp) = turn.timestamp.as_ref().filter(|value| !value.is_empty()) {
            if usage
                .latest_timestamp
                .as_ref()
                .is_none_or(|current| timestamp > current)
            {
                usage.latest_timestamp = Some(timestamp.clone());
            }
        }

        usage.turns_by_time.push(turn);
    }

    Ok(())
}

#[derive(Clone)]
struct TurnUsage {
    session_id: String,
    timestamp: Option<String>,
    model: Option<String>,
    message_id: Option<String>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
}

struct ActiveSessionLimit {
    resets_at: DateTime<Utc>,
    used_tokens: u64,
    token_limit: u64,
    reset_source: ResetSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServerResetAnchor {
    resets_at: DateTime<Utc>,
    source_path: String,
}

impl PartialOrd for ServerResetAnchor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ServerResetAnchor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resets_at.cmp(&other.resets_at)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResetSource {
    ServerAnchor,
    TranscriptEstimate,
}

fn extract_turn_usage(record: &Value) -> Option<TurnUsage> {
    if record.get("type")?.as_str()? != "assistant" {
        return None;
    }

    let session_id = record.get("sessionId")?.as_str()?.to_string();
    let message = record.get("message")?;
    let usage = message.get("usage")?;
    let input_tokens = number_field(usage, "input_tokens");
    let output_tokens = number_field(usage, "output_tokens");
    let cache_read_tokens = number_field(usage, "cache_read_input_tokens");
    let cache_creation_tokens = number_field(usage, "cache_creation_input_tokens");

    if input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens == 0 {
        return None;
    }

    Some(TurnUsage {
        session_id,
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model: message
            .get("model")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        message_id: message
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

fn number_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn extract_server_reset_anchor(record: &Value) -> Option<ServerResetAnchor> {
    let mut candidates = Vec::new();
    collect_server_reset_anchor_candidates(record, "", false, &mut candidates);
    candidates.into_iter().max()
}

fn collect_server_reset_anchor_candidates(
    value: &Value,
    path: &str,
    in_reset_context: bool,
    candidates: &mut Vec<ServerResetAnchor>,
) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = join_json_path(path, key);
                let key_is_reset_context = in_reset_context || is_server_reset_context_key(key);

                if is_reset_timestamp(key) && key_is_reset_context {
                    if let Some(resets_at) = parse_reset_timestamp_value(child) {
                        candidates.push(ServerResetAnchor {
                            resets_at,
                            source_path: child_path.clone(),
                        });
                    }
                }

                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    key_is_reset_context,
                    candidates,
                );
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                collect_server_reset_anchor_candidates(
                    child,
                    &child_path,
                    in_reset_context,
                    candidates,
                );
            }
        }
        _ => {}
    }
}

fn join_json_path(path: &str, key: &str) -> String {
    let escaped = key.replace('~', "~0").replace('/', "~1");
    if path.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{path}/{escaped}")
    }
}

fn is_server_reset_context_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    normalized.contains("ratelimit")
        || normalized.contains("usagelimit")
        || normalized.contains("usage")
        || normalized.contains("quota")
        || normalized.contains("429")
}

fn is_reset_timestamp(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    matches!(
        normalized.as_str(),
        "resetsat" | "resetat" | "resettime" | "resettimestamp" | "limitresetat"
    )
}

fn parse_reset_timestamp_value(value: &Value) -> Option<DateTime<Utc>> {
    if let Some(timestamp) = value.as_u64() {
        return DateTime::from_timestamp(timestamp as i64, 0);
    }

    if let Some(timestamp) = value.as_i64() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    let text = value.as_str()?.trim();
    if text.is_empty() {
        return None;
    }

    if let Ok(timestamp) = text.parse::<i64>() {
        return DateTime::from_timestamp(timestamp, 0);
    }

    parse_timestamp(text)
}

fn encode_raw(
    candidate_roots: &[PathBuf],
    scanned_roots: &[PathBuf],
    usage: Option<&ClaudeLocalUsage>,
) -> io::Result<String> {
    let mut payload = json!({
        "candidate_roots": path_strings(candidate_roots),
        "scanned_roots": path_strings(scanned_roots),
    });

    if let Some(usage) = usage {
        let total_tokens = usage.input_tokens
            + usage.output_tokens
            + usage.cache_read_tokens
            + usage.cache_creation_tokens;
        let mut models = usage
            .models
            .iter()
            .map(|(model, count)| (model.clone(), json!(count)))
            .collect::<Vec<_>>();
        models.sort_by(|(left, _), (right, _)| left.cmp(right));

        payload["usage"] = json!({
            "files": usage.files,
            "sessions": usage.sessions.iter().collect::<Vec<_>>(),
            "turns": usage.turns,
            "input_tokens": usage.input_tokens,
            "output_tokens": usage.output_tokens,
            "cache_read_tokens": usage.cache_read_tokens,
            "cache_creation_tokens": usage.cache_creation_tokens,
            "total_tokens": total_tokens,
            "models": Value::Object(models.into_iter().collect()),
            "latest_timestamp": usage.latest_timestamp,
            "latest_server_reset_anchor": usage.latest_server_reset_anchor.as_ref().map(|anchor| {
                json!({
                    "resets_at": anchor.resets_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    "source_path": anchor.source_path,
                })
            }),
        });
    }

    serde_json::to_string(&payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn structured_base(
    status: SourceStatus,
    raw_data_available: bool,
    data_as_of: Option<String>,
) -> StructuredSourceInfo {
    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status,
        raw_data_available,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: Vec::new(),
        available_limit_resets: None,
        usage: UsageInfo::default(),
        diagnostics: Vec::new(),
    }
}

fn structured_no_roots() -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some("local transcript roots were not found".to_string()),
        },
        true,
        None,
    )
}

fn structured_no_usage(root_count: usize) -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some(format!(
                "no token usage found in {root_count} local transcript root(s)"
            )),
        },
        true,
        None,
    )
}

fn structured_from_usage(usage: &ClaudeLocalUsage) -> StructuredSourceInfo {
    let total_tokens = usage.input_tokens
        + usage.output_tokens
        + usage.cache_read_tokens
        + usage.cache_creation_tokens;
    let active_session_limit = active_session_limit(usage, Utc::now());
    let mut diagnostics = vec![
        "5h token usage is reconstructed from transcript input+output tokens".to_string(),
        "5h local estimate uses Claude Max5 token limit: 88,000".to_string(),
    ];
    let data_as_of = usage.latest_timestamp.clone();
    if data_as_of.is_none() {
        diagnostics.push("latest transcript record timestamp is unavailable".to_string());
    }
    match active_session_limit
        .as_ref()
        .map(|limit| limit.reset_source)
    {
        None => diagnostics.push("no active 5h local transcript window found".to_string()),
        Some(ResetSource::ServerAnchor) => {
            if let Some(anchor) = usage.latest_server_reset_anchor.as_ref() {
                diagnostics.push(format!(
                    "5h reset uses latest server reset anchor found in local data at {}",
                    anchor.source_path
                ));
            }
        }
        Some(ResetSource::TranscriptEstimate) => diagnostics.push(
            "5h reset is estimated from local transcript timing; official reset unavailable"
                .to_string(),
        ),
    }

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available: true,
            access_available: true,
            message: None,
        },
        raw_data_available: true,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: active_session_limit
            .as_ref()
            .map(limit_info_from_active_session)
            .into_iter()
            .collect(),
        available_limit_resets: None,
        usage: UsageInfo {
            tokens: TokenUsage {
                input: Some(usage.input_tokens),
                cached_input: None,
                output: Some(usage.output_tokens),
                reasoning_output: None,
                cache_read: Some(usage.cache_read_tokens),
                cache_write: Some(usage.cache_creation_tokens),
                total: Some(total_tokens),
            },
            money: MoneyUsage::default(),
            activity: ActivityUsage {
                events_count: None,
                files_count: Some(usage.files as u64),
                sessions_count: Some(usage.sessions.len() as u64),
                turns_count: Some(usage.turns as u64),
                latest_activity_at: usage.latest_timestamp.clone(),
            },
            models: ModelUsage {
                top_model: top_model(&usage.models).map(str::to_string),
            },
        },
        diagnostics,
    }
}

fn active_session_limit(
    usage: &ClaudeLocalUsage,
    now: DateTime<Utc>,
) -> Option<ActiveSessionLimit> {
    let mut turns = usage
        .turns_by_time
        .iter()
        .filter_map(|turn| {
            let timestamp = turn.timestamp.as_deref().and_then(parse_timestamp)?;
            Some((timestamp, turn))
        })
        .collect::<Vec<_>>();
    turns.sort_by_key(|(timestamp, _)| *timestamp);

    let mut current: Option<ActiveSessionLimit> = None;
    let mut previous_timestamp: Option<DateTime<Utc>> = None;
    let session_duration = Duration::minutes(CLAUDE_LOCAL_SESSION_WINDOW_MINUTES as i64);

    if let Some(anchor) = usage
        .latest_server_reset_anchor
        .as_ref()
        .filter(|anchor| anchor.resets_at > now)
    {
        let resets_at = anchor.resets_at;
        let window_start = resets_at - session_duration;
        let used_tokens = turns
            .iter()
            .filter(|(timestamp, _)| *timestamp >= window_start && *timestamp < resets_at)
            .map(|(_, turn)| turn.input_tokens + turn.output_tokens)
            .sum();

        return Some(ActiveSessionLimit {
            resets_at,
            used_tokens,
            token_limit: CLAUDE_LOCAL_MAX5_TOKEN_LIMIT,
            reset_source: ResetSource::ServerAnchor,
        });
    }

    for (timestamp, turn) in turns {
        let should_start_new = current
            .as_ref()
            .is_none_or(|block| timestamp >= block.resets_at)
            || previous_timestamp.is_some_and(|previous| timestamp - previous >= session_duration);

        if should_start_new {
            current = Some(ActiveSessionLimit {
                resets_at: timestamp + session_duration,
                used_tokens: 0,
                token_limit: CLAUDE_LOCAL_MAX5_TOKEN_LIMIT,
                reset_source: ResetSource::TranscriptEstimate,
            });
        }

        if let Some(block) = current.as_mut() {
            block.used_tokens += turn.input_tokens + turn.output_tokens;
        }
        previous_timestamp = Some(timestamp);
    }

    current.filter(|block| block.resets_at > now)
}

fn limit_info_from_active_session(session: &ActiveSessionLimit) -> LimitInfo {
    let used_percent = if session.token_limit > 0 {
        (session.used_tokens as f64 / session.token_limit as f64) * 100.0
    } else {
        0.0
    };
    let remaining_amount = session.token_limit.saturating_sub(session.used_tokens);

    LimitInfo {
        name: match session.reset_source {
            ResetSource::ServerAnchor => "5h local estimate (server reset anchor)".to_string(),
            ResetSource::TranscriptEstimate => "5h local estimate (estimated reset)".to_string(),
        },
        window_label: Some("5h".to_string()),
        window_minutes: Some(CLAUDE_LOCAL_SESSION_WINDOW_MINUTES),
        resets_at: Some(
            session
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ),
        used_percent: Some((used_percent * 10.0).round() / 10.0),
        remaining_percent: Some(((100.0 - used_percent).clamp(0.0, 100.0) * 10.0).round() / 10.0),
        used_amount: Some(session.used_tokens as f64),
        remaining_amount: Some(remaining_amount as f64),
        total_amount: Some(session.token_limit as f64),
        amount_unit: Some("tokens".to_string()),
    }
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .ok()
}

fn top_model(models: &HashMap<String, u64>) -> Option<&str> {
    models
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(model, _)| model.as_str())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use super::*;

    fn sample_usage() -> ClaudeLocalUsage {
        let mut usage = ClaudeLocalUsage::default();
        usage.files = 2;
        usage.sessions.extend(["s1".to_string(), "s2".to_string()]);
        usage.turns = 5;
        usage.input_tokens = 100;
        usage.output_tokens = 40;
        usage.cache_read_tokens = 10;
        usage.cache_creation_tokens = 5;
        usage.latest_timestamp = Some("2026-06-28T10:01:00Z".to_string());
        usage.models.insert("claude-sonnet-4-6".to_string(), 3);
        usage.models.insert("claude-haiku-4-5".to_string(), 2);
        usage
    }

    #[test]
    fn scans_usage_and_deduplicates_streaming_message_records() {
        let path = env::temp_dir().join(format!(
            "ai-limits-claude-local-{}.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:00:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":1,"cache_creation_input_tokens":2}}}
{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:01:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":30,"output_tokens":7,"cache_read_input_tokens":3,"cache_creation_input_tokens":4}}}
{"type":"assistant","sessionId":"s2","timestamp":"2026-06-28T10:02:00Z","message":{"model":"claude-haiku-4-5","usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}
"#,
        )
        .expect("write fixture");

        let mut usage = ClaudeLocalUsage::default();
        scan_jsonl_file(&path, &mut usage).expect("scan fixture");
        let structured = structured_from_usage(&usage);
        let _ = fs::remove_file(&path);

        assert_eq!(usage.files, 1);
        assert_eq!(usage.sessions.len(), 1);
        assert_eq!(usage.turns, 1);
        assert_eq!(usage.input_tokens, 30);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_read_tokens, 3);
        assert_eq!(usage.cache_creation_tokens, 4);

        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(structured.usage.tokens.input, Some(30));
        assert_eq!(structured.usage.tokens.output, Some(7));
        assert_eq!(structured.usage.tokens.cache_read, Some(3));
        assert_eq!(structured.usage.tokens.cache_write, Some(4));
        assert_eq!(structured.usage.tokens.total, Some(44));
        assert_eq!(structured.usage.activity.turns_count, Some(1));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
    }

    #[test]
    fn builds_structured_data_from_representative_usage_sample() {
        let usage = sample_usage();
        let structured = structured_from_usage(&usage);

        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_local");
        assert_eq!(structured.source_link, "docs/get-info");
        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.raw_data_available);
        assert_eq!(structured.usage.tokens.input, Some(100));
        assert_eq!(structured.usage.tokens.output, Some(40));
        assert_eq!(structured.usage.tokens.cache_read, Some(10));
        assert_eq!(structured.usage.tokens.cache_write, Some(5));
        assert_eq!(structured.usage.tokens.total, Some(155));
        assert_eq!(structured.usage.activity.files_count, Some(2));
        assert_eq!(structured.usage.activity.sessions_count, Some(2));
        assert_eq!(structured.usage.activity.turns_count, Some(5));
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("transcript input+output tokens")));
    }

    #[test]
    fn transcript_estimate_does_not_round_reset_down_to_hour() {
        let mut usage = ClaudeLocalUsage::default();
        usage.turns_by_time.push(TurnUsage {
            session_id: "s1".to_string(),
            timestamp: Some("2026-06-28T10:37:12Z".to_string()),
            model: None,
            message_id: Some("m1".to_string()),
            input_tokens: 100,
            output_tokens: 40,
            cache_read_tokens: 1_000,
            cache_creation_tokens: 2_000,
        });

        let now = parse_timestamp("2026-06-28T11:00:00Z").expect("parse now");
        let limit = active_session_limit(&usage, now).expect("active limit");

        assert_eq!(
            limit
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-28T15:37:12Z"
        );
        assert_eq!(limit.used_tokens, 140);
        assert_eq!(limit.reset_source, ResetSource::TranscriptEstimate);

        let info = limit_info_from_active_session(&limit);
        assert_eq!(info.name.as_str(), "5h local estimate (estimated reset)");
    }

    #[test]
    fn server_reset_anchor_overrides_transcript_estimated_window() {
        let mut usage = ClaudeLocalUsage::default();
        usage.latest_server_reset_anchor = Some(ServerResetAnchor {
            resets_at: parse_timestamp("2026-06-28T15:00:00Z").expect("parse anchor"),
            source_path: "/payload/rate_limits/five_hour/resets_at".to_string(),
        });
        usage.turns_by_time.push(TurnUsage {
            session_id: "old".to_string(),
            timestamp: Some("2026-06-28T09:59:59Z".to_string()),
            model: None,
            message_id: Some("old".to_string()),
            input_tokens: 1_000,
            output_tokens: 1_000,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        });
        usage.turns_by_time.push(TurnUsage {
            session_id: "current".to_string(),
            timestamp: Some("2026-06-28T10:00:00Z".to_string()),
            model: None,
            message_id: Some("current".to_string()),
            input_tokens: 20,
            output_tokens: 5,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
        });

        let now = parse_timestamp("2026-06-28T11:00:00Z").expect("parse now");
        let limit = active_session_limit(&usage, now).expect("active limit");

        assert_eq!(
            limit
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-28T15:00:00Z"
        );
        assert_eq!(limit.used_tokens, 25);
        assert_eq!(limit.reset_source, ResetSource::ServerAnchor);

        let info = limit_info_from_active_session(&limit);
        assert_eq!(
            info.name.as_str(),
            "5h local estimate (server reset anchor)"
        );
    }

    #[test]
    fn extracts_server_reset_anchor_from_rate_limits_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"assistant","payload":{"rate_limits":{"five_hour":{"resets_at":"1782721800"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(
            anchor.source_path,
            "/payload/rate_limits/five_hour/resets_at"
        );
    }

    #[test]
    fn extracts_server_reset_anchor_from_nested_429_usage_limit_payload() {
        let record: Value = serde_json::from_str(
            r#"{"type":"error","payload":{"status":429,"error":{"usage_limit":{"reset_time":"2026-06-29T08:30:00Z"}}}}"#,
        )
        .expect("parse record");

        let anchor = extract_server_reset_anchor(&record).expect("server reset anchor");

        assert_eq!(
            anchor
                .resets_at
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "2026-06-29T08:30:00Z"
        );
        assert_eq!(anchor.source_path, "/payload/error/usage_limit/reset_time");
    }

    #[test]
    fn structured_unavailable_when_transcript_roots_are_missing() {
        let structured = structured_no_roots();

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("local transcript roots were not found")
        );
        assert!(structured.raw_data_available);
        assert!(structured.limits.is_empty());
    }

    #[test]
    fn structured_unavailable_when_no_token_usage_is_found() {
        let structured = structured_no_usage(2);

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("no token usage found in 2 local transcript root(s)")
        );
        assert!(structured.raw_data_available);
    }

    #[test]
    fn raw_payload_contains_scanned_roots_and_extracted_usage() {
        let candidate_roots = vec![PathBuf::from("/tmp/.config/claude/projects")];
        let scanned_roots = candidate_roots.clone();
        let usage = sample_usage();

        let raw = encode_raw(&candidate_roots, &scanned_roots, Some(&usage)).expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["candidate_roots"][0].as_str(),
            Some("/tmp/.config/claude/projects")
        );
        assert_eq!(payload["usage"]["turns"].as_u64(), Some(5));
        assert_eq!(payload["usage"]["total_tokens"].as_u64(), Some(155));
        assert_eq!(
            payload["usage"]["latest_timestamp"].as_str(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(payload["usage"]["latest_server_reset_anchor"].is_null());
    }

    #[test]
    fn raw_payload_exposes_latest_server_reset_anchor_for_diagnostics() {
        let candidate_roots = vec![PathBuf::from("/tmp/.config/claude/projects")];
        let scanned_roots = candidate_roots.clone();
        let mut usage = sample_usage();
        usage.latest_server_reset_anchor = Some(ServerResetAnchor {
            resets_at: parse_timestamp("2026-06-29T08:30:00Z").expect("parse anchor"),
            source_path: "/payload/error/usage_limit/reset_time".to_string(),
        });

        let raw = encode_raw(&candidate_roots, &scanned_roots, Some(&usage)).expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["usage"]["latest_server_reset_anchor"]["resets_at"].as_str(),
            Some("2026-06-29T08:30:00Z")
        );
        assert_eq!(
            payload["usage"]["latest_server_reset_anchor"]["source_path"].as_str(),
            Some("/payload/error/usage_limit/reset_time")
        );
    }
}
