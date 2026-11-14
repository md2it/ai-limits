use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{json, Value};

use crate::infra::os_access::{CURSOR_DASHBOARD_METHODS, CURSOR_DASHBOARD_URL_PREFIX};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    Missing,
    Unreadable,
    SqliteUnavailable,
    Protected,
    Malformed,
    Choice,
    SignedOut,
    Unauthorized,
    Forbidden,
    Transient,
}

impl Failure {
    pub fn message(self) -> &'static str {
        match self {
            Self::Missing | Self::SignedOut => "Sign in to Cursor, then retry.",
            Self::SqliteUnavailable => {
                "Install sqlite3 to read Cursor IDE sign-in information, or sign in to Cursor CLI."
            }
            Self::Protected => "Cursor IDE protects its sign-in data. Run cursor agent login, then use CLI in Settings.",
            Self::Unreadable => "Could not read Cursor sign-in information.",
            Self::Malformed => "Cursor sign-in information is not recognized.",
            Self::Choice => "Choose which Cursor account to use.",
            Self::Forbidden => "This Cursor account cannot access usage data.",
            Self::Unauthorized => {
                "Cursor session could not be renewed. Sign in to Cursor, then retry."
            }
            Self::Transient => "Could not update Cursor. Try again.",
        }
    }

    pub fn invalidates_cache(self) -> bool {
        matches!(
            self,
            Self::Missing | Self::SignedOut | Self::Choice | Self::Unauthorized | Self::Protected
        )
    }
}

pub(super) struct Response {
    pub status: u16,
    pub body: String,
    pub retry_after: Option<u64>,
}

pub(super) trait Transport {
    fn send(&mut self, method: &str, body: &str, token: Option<&str>) -> Result<Response, Failure>;
}

pub(super) struct Curl;

impl Transport for Curl {
    fn send(&mut self, method: &str, body: &str, token: Option<&str>) -> Result<Response, Failure> {
        let url = if method == "oauth/token" {
            "https://api2.cursor.sh/oauth/token".to_string()
        } else if CURSOR_DASHBOARD_METHODS.contains(&method) {
            format!("{CURSOR_DASHBOARD_URL_PREFIX}{method}")
        } else {
            return Err(Failure::Transient);
        };
        let mut config = String::from("header = \"Content-Type: application/json\"\nheader = \"Connect-Protocol-Version: 1\"\n");
        if let Some(token) = token {
            if token.chars().any(|c| c.is_control()) {
                return Err(Failure::Malformed);
            }
            config.push_str(&format!(
                "header = {}\n",
                json!(format!("Authorization: Bearer {token}"))
            ));
        }
        config.push_str(&format!("data = {}\n", json!(body)));
        let mut child = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" })
            .args([
                "-sS",
                "--max-time",
                "20",
                "--connect-timeout",
                "10",
                "-D",
                "-",
                "-X",
                "POST",
                "-K",
                "-",
                "-w",
                "\n%{http_code}",
                &url,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Failure::Transient)?;
        let write_result = child
            .stdin
            .take()
            .ok_or(Failure::Transient)?
            .write_all(config.as_bytes());
        let output = child.wait_with_output().map_err(|_| Failure::Transient)?;
        if write_result.is_err() || !output.status.success() {
            return Err(Failure::Transient);
        }
        parse_response(&String::from_utf8_lossy(&output.stdout))
    }
}

fn parse_response(text: &str) -> Result<Response, Failure> {
    let (mut text, code) = text.rsplit_once('\n').ok_or(Failure::Transient)?;
    let status = code.trim().parse().map_err(|_| Failure::Transient)?;
    let mut retry_after = None;
    while text.starts_with("HTTP/") {
        let (headers, rest) = text.split_once("\r\n\r\n").ok_or(Failure::Transient)?;
        for line in headers.lines() {
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("retry-after") {
                    retry_after = value.trim().parse().ok().or_else(|| {
                        chrono::DateTime::parse_from_rfc2822(value.trim())
                            .ok()
                            .map(|date| {
                                (date.timestamp() - chrono::Utc::now().timestamp()).max(0) as u64
                            })
                    });
                }
            }
        }
        text = rest;
    }
    Ok(Response {
        status,
        body: text.to_string(),
        retry_after,
    })
}

pub(super) fn classify(response: &Response) -> Result<Value, Failure> {
    if response.status == 401 {
        return Err(Failure::Unauthorized);
    }
    if response.status == 403 {
        return Err(Failure::Forbidden);
    }
    let value: Value = serde_json::from_str(&response.body).map_err(|_| Failure::Transient)?;
    match (response.status, value.get("code").and_then(Value::as_str)) {
        (401, _) | (_, Some("unauthenticated")) => Err(Failure::Unauthorized),
        (403, _) | (_, Some("permission_denied")) => Err(Failure::Forbidden),
        (200..=299, None) if value.is_object() => Ok(value),
        _ => Err(Failure::Transient),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn errors_are_parsed_not_substring_searched() {
        let response = |status, body: &str| Response {
            status,
            body: body.into(),
            retry_after: None,
        };
        assert_eq!(
            classify(&response(401, "not json")),
            Err(Failure::Unauthorized)
        );
        assert_eq!(
            classify(&response(200, r#"{"code": "permission_denied"}"#)),
            Err(Failure::Forbidden)
        );
        assert!(classify(&response(200, r#"{"description":"Unauthorized"}"#)).is_ok());
        assert_eq!(classify(&response(500, "{}")), Err(Failure::Transient));
    }
    #[test]
    fn parses_proxy_headers_and_cooldown() {
        let result = parse_response("HTTP/1.1 200 Connection established\r\n\r\nHTTP/2 429\r\nRetry-After: 123\r\n\r\n{}\n429").unwrap();
        assert_eq!(result.status, 429);
        assert_eq!(result.retry_after, Some(123));
        assert_eq!(result.body, "{}");
    }
}
