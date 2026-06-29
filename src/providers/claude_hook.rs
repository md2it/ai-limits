use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::Value;

const CLAUDE_COMMAND: &str = "claude";
const EXPECT_COMMAND: &str = "expect";
const PROBE_PROCESS_TIMEOUT: Duration = Duration::from_secs(90);
const PROBE_SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

#[derive(Clone, Default)]
struct RateLimits {
    primary_label: &'static str,
    primary: Option<RateLimitWindow>,
    secondary_label: &'static str,
    secondary: Option<RateLimitWindow>,
    credits: Option<f64>,
    plan_type: Option<String>,
}

#[derive(Clone, Default)]
struct RateLimitWindow {
    used_percent: Option<f64>,
    window_minutes: Option<u64>,
    resets_at: Option<u64>,
}

pub fn get_usage_summary() -> io::Result<String> {
    if io::stdin().is_terminal() {
        return probe_statusline_hook();
    }

    let mut payload = String::new();
    io::stdin().read_to_string(&mut payload)?;
    if payload.trim().is_empty() {
        probe_statusline_hook()
    } else {
        Ok(extract_usage_summary_from_hook_payload(&payload))
    }
}

fn probe_statusline_hook() -> io::Result<String> {
    let temp_dir = create_probe_temp_dir()?;
    let payload_path = temp_dir.join("payload.json");
    let capture_path = temp_dir.join("capture.sh");
    let settings_path = temp_dir.join("settings.json");

    write_capture_script(&capture_path, &payload_path)?;
    write_probe_settings(&settings_path, &capture_path)?;

    let mut child = spawn_hook_probe(&settings_path)?;
    let probe_result = wait_for_hook_payload(&mut child, &payload_path);
    let payload = fs::read_to_string(&payload_path).ok();
    let _ = fs::remove_dir_all(&temp_dir);

    match probe_result {
        Err(error) => Ok(unavailable_summary(&format!(
            "Claude hook probe failed: {error}"
        ))),
        Ok(()) => match payload {
            Some(payload) if !payload.trim().is_empty() => {
                Ok(extract_usage_summary_from_hook_payload(&payload))
            }
            _ => Ok(unavailable_summary(
                "Claude hook probe did not capture statusline payload",
            )),
        },
    }
}

fn create_probe_temp_dir() -> io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "ai-usage-claude-hook-{nanos}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_capture_script(capture_path: &Path, payload_path: &Path) -> io::Result<()> {
    let script = format!(
        "#!/bin/sh\ncat > {}\necho ok\n",
        shell_single_quote(payload_path)
    );
    fs::write(capture_path, script)?;
    set_executable(capture_path)?;
    Ok(())
}

fn write_probe_settings(settings_path: &Path, capture_path: &Path) -> io::Result<()> {
    let settings = serde_json::json!({
        "statusLine": {
            "type": "command",
            "command": capture_path.to_string_lossy(),
        }
    });
    fs::write(settings_path, settings.to_string())
}

fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn shell_single_quote(path: &Path) -> String {
    format!(
        "'{}'",
        path.display().to_string().replace('\'', "'\\''")
    )
}

fn spawn_hook_probe(settings_path: &Path) -> io::Result<std::process::Child> {
    let settings_path = settings_path.to_string_lossy();
    let expect_script = format!(
        r#"set timeout 20
log_user 0
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CLAUDE_COMMAND} --no-chrome --settings {settings_path}}}
expect {{
    -re {{Choose.*text.*style|Syntax theme}} {{
        send "\r"
        exp_continue
    }}
    -re {{for shortcuts|Do you trust|Select login method}} {{}}
    timeout {{}}
}}
after 500
send -- "Reply with exactly: pong\r"
set timeout 60
expect {{
    -re {{pong|Pong}} {{}}
    timeout {{}}
}}
set timeout 5
expect {{
    eof {{}}
    timeout {{exit 0}}
}}
"#
    );

    Command::new(EXPECT_COMMAND)
        .args(["-c", &expect_script])
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("failed to run `{EXPECT_COMMAND}` for Claude hook probe: {error}"),
            )
        })
}

fn wait_for_hook_payload(
    child: &mut std::process::Child,
    payload_path: &Path,
) -> io::Result<()> {
    let started_at = Instant::now();

    loop {
        if payload_path.exists() {
            if payload_contains_rate_limits(payload_path)? {
                let _ = child.kill();
                let _ = child.wait();
                thread::sleep(PROBE_SHUTDOWN_WAIT);
                return Ok(());
            }
        }

        if child.try_wait()?.is_some() {
            thread::sleep(PROBE_SHUTDOWN_WAIT);
            return Ok(());
        }

        if started_at.elapsed() >= PROBE_PROCESS_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Claude hook probe timed out",
            ));
        }

        thread::sleep(Duration::from_millis(200));
    }
}

fn payload_contains_rate_limits(payload_path: &Path) -> io::Result<bool> {
    let payload = fs::read_to_string(payload_path)?;
    let record = match serde_json::from_str::<Value>(&payload) {
        Ok(record) => record,
        Err(_) => return Ok(false),
    };
    let Some(rate_limits_value) = locate_rate_limits(&record) else {
        return Ok(false);
    };
    let rate_limits = parse_rate_limits(rate_limits_value);
    Ok(rate_limits.primary.is_some() || rate_limits.secondary.is_some())
}

pub fn extract_usage_summary_from_hook_payload(payload: &str) -> String {
    let payload = payload.trim();
    if payload.is_empty() {
        return unavailable_summary("hook stdin payload is empty");
    }

    let record = match serde_json::from_str::<Value>(payload) {
        Ok(record) => record,
        Err(_) => return unavailable_summary("hook stdin payload is not valid JSON"),
    };

    let Some(rate_limits_value) = locate_rate_limits(&record) else {
        return unavailable_summary("`rate_limits` field is missing in hook payload");
    };

    let rate_limits = parse_rate_limits(rate_limits_value);
    if rate_limits.primary.is_none()
        && rate_limits.secondary.is_none()
        && rate_limits.credits.is_none()
        && rate_limits.plan_type.is_none()
    {
        return unavailable_summary("`rate_limits` has no supported limit fields");
    }

    let mut summary = String::from("Claude usage:\n");
    summary.push_str("Source: Claude hook rate_limits\n");
    summary.push_str(&format_rate_limit_window(
        rate_limits.primary_label,
        &rate_limits.primary,
    ));
    summary.push_str(&format_rate_limit_window(
        rate_limits.secondary_label,
        &rate_limits.secondary,
    ));

    if let Some(credits) = rate_limits.credits {
        summary.push_str(&format!("Credits: {}\n", format_decimal(credits)));
    }

    if let Some(plan_type) = rate_limits.plan_type {
        summary.push_str(&format!("Plan: {plan_type}\n"));
    }

    summary
}

fn locate_rate_limits(record: &Value) -> Option<&Value> {
    record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))
}

fn parse_rate_limits(value: &Value) -> RateLimits {
    let (primary_label, primary) = if value.get("five_hour").is_some() {
        (
            "5-hour window",
            parse_named_window(value.get("five_hour"), 300),
        )
    } else {
        (
            "Primary window",
            parse_rate_limit_window(value.get("primary")),
        )
    };

    let (secondary_label, secondary) = if value.get("seven_day").is_some() {
        (
            "7-day window",
            parse_named_window(value.get("seven_day"), 10080),
        )
    } else {
        (
            "Secondary window",
            parse_rate_limit_window(value.get("secondary")),
        )
    };

    RateLimits {
        primary_label,
        primary,
        secondary_label,
        secondary,
        credits: value.get("credits").and_then(number_f64_any),
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn parse_named_window(value: Option<&Value>, default_minutes: u64) -> Option<RateLimitWindow> {
    let mut window = parse_rate_limit_window(value)?;
    if window.window_minutes.is_none() {
        window.window_minutes = Some(default_minutes);
    }
    Some(window)
}

fn parse_rate_limit_window(value: Option<&Value>) -> Option<RateLimitWindow> {
    let value = value?;
    let used_percent = value
        .get("used_percent")
        .or_else(|| value.get("used_percentage"))
        .and_then(number_f64_any);
    let window_minutes = value.get("window_minutes").and_then(number_u64_any);
    let resets_at = value.get("resets_at").and_then(number_u64_any);

    if used_percent.is_none() && window_minutes.is_none() && resets_at.is_none() {
        return None;
    }

    Some(RateLimitWindow {
        used_percent,
        window_minutes,
        resets_at,
    })
}

fn number_u64_any(value: &Value) -> Option<u64> {
    if let Some(number) = value.as_u64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<u64>().ok())
}

fn number_f64_any(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<f64>().ok())
}

fn format_rate_limit_window(label: &str, value: &Option<RateLimitWindow>) -> String {
    let Some(value) = value else {
        return format!("{label}: unavailable\n");
    };

    let mut details = Vec::new();
    if let Some(used_percent) = value.used_percent {
        details.push(format!("used {}%", format_decimal(used_percent)));
    }
    if let Some(window_minutes) = value.window_minutes {
        details.push(format!("window {}", format_window(window_minutes)));
    }
    if let Some(resets_at) = value.resets_at {
        details.push(format!("resets at {resets_at} (unix)"));
    }

    if details.is_empty() {
        format!("{label}: unavailable\n")
    } else {
        format!("{label}: {}\n", details.join(", "))
    }
}

fn format_window(minutes: u64) -> String {
    match minutes {
        300 => "5h (300m)".to_string(),
        10080 => "weekly (10080m)".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

fn unavailable_summary(reason: &str) -> String {
    format!(
        "Claude usage:\nClaude hook live limits unavailable: {reason}\nFallback: Claude CLI /usage or claude_local history\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hook_rate_limits_with_supported_fields() {
        let payload = r#"{
  "rate_limits": {
    "primary": {"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},
    "secondary": {"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},
    "credits":123.6,
    "plan_type":"max"
  }
}"#;

        let summary = extract_usage_summary_from_hook_payload(payload);
        assert!(summary.contains("Source: Claude hook rate_limits"));
        assert!(summary
            .contains("Primary window: used 45%, window 5h (300m), resets at 1750003600 (unix)"));
        assert!(summary.contains(
            "Secondary window: used 71.9%, window weekly (10080m), resets at 1750600000 (unix)"
        ));
        assert!(summary.contains("Credits: 123.6"));
        assert!(summary.contains("Plan: max"));
    }

    #[test]
    fn parses_official_statusline_rate_limits() {
        let payload = r#"{
  "rate_limits": {
    "five_hour": {"used_percentage": 1, "resets_at": 1782721800},
    "seven_day": {"used_percentage": 69, "resets_at": 1782813600}
  }
}"#;

        let summary = extract_usage_summary_from_hook_payload(payload);
        assert!(summary.contains(
            "5-hour window: used 1%, window 5h (300m), resets at 1782721800 (unix)"
        ));
        assert!(summary.contains(
            "7-day window: used 69%, window weekly (10080m), resets at 1782813600 (unix)"
        ));
    }

    #[test]
    fn handles_missing_rate_limits_with_clear_fallback() {
        let payload = r#"{"hook_event":"statusline","payload":{"session_id":"abc"}}"#;
        let summary = extract_usage_summary_from_hook_payload(payload);

        assert!(summary.contains(
            "Claude hook live limits unavailable: `rate_limits` field is missing in hook payload"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn handles_invalid_json_with_clear_fallback() {
        let summary = extract_usage_summary_from_hook_payload("{invalid");

        assert!(summary
            .contains("Claude hook live limits unavailable: hook stdin payload is not valid JSON"));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn handles_rate_limits_without_supported_fields() {
        let payload = r#"{"rate_limits":{"primary":{"foo":"bar"}}}"#;
        let summary = extract_usage_summary_from_hook_payload(payload);

        assert!(summary.contains(
            "Claude hook live limits unavailable: `rate_limits` has no supported limit fields"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }
}
