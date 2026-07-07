use std::env;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub const CURSOR_ACCESS_TOKEN_SERVICE: &str = "cursor-access-token";
pub const CURSOR_USAGE_URL: &str =
    "https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage";

pub const CLAUDE_CLI_COMMAND: &str = "claude";
pub const CODEX_CLI_COMMAND: &str = "codex";
pub const EXPECT_COMMAND: &str = "expect";

pub const ALLOWED_EXTERNAL_URLS: &[&str] = &[
    "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md",
    "https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md",
];

pub fn ai_limits_config_dir() -> io::Result<PathBuf> {
    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(".config").join("ai-limits"))
}

pub fn codex_local_root() -> io::Result<PathBuf> {
    if let Some(value) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(value));
    }

    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate ${CODEX_HOME:-~/.codex}",
        )
    })?;

    Ok(PathBuf::from(home).join(".codex"))
}

pub fn claude_local_roots() -> io::Result<Vec<PathBuf>> {
    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate Claude local transcript roots",
        )
    })?;
    let home = PathBuf::from(home);

    Ok(vec![
        home.join(".config").join("claude").join("projects"),
        home.join(".claude").join("projects"),
        home.join("Library")
            .join("Developer")
            .join("Xcode")
            .join("CodingAssistant")
            .join("ClaudeAgentConfig")
            .join("projects"),
    ])
}

pub fn read_cursor_access_token() -> io::Result<std::process::Output> {
    Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            CURSOR_ACCESS_TOKEN_SERVICE,
            "-w",
        ])
        .stdin(Stdio::null())
        .output()
}

pub fn cursor_usage_request_command() -> Command {
    let mut command = Command::new("curl");
    command.args(["-sS", "-X", "POST", CURSOR_USAGE_URL, "-K", "-", "-d", "{}"]);
    command
}

pub fn allowed_cli_command_is_available(command: &str) -> bool {
    if !matches!(command, CLAUDE_CLI_COMMAND | CODEX_CLI_COMMAND) {
        return false;
    }

    Command::new(command)
        .arg("--version")
        .env("PATH", cli_process_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn is_allowed_external_url(url: &str) -> bool {
    ALLOWED_EXTERNAL_URLS.contains(&url)
}

#[cfg(target_os = "macos")]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("open").arg(url).spawn()?.wait()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn open_external_url_with_system(url: &str) -> io::Result<()> {
    Command::new("xdg-open").arg(url).spawn()?.wait()?;
    Ok(())
}

pub fn cli_process_path() -> OsString {
    let current_path = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<_> = env::split_paths(&current_path).collect();

    let mut extra_paths: Vec<PathBuf> = vec![
        "/usr/local/bin".into(),
        "/usr/bin".into(),
        "/bin".into(),
        "/usr/sbin".into(),
        "/sbin".into(),
        "/opt/homebrew/bin".into(),
    ];

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        extra_paths.push(home.join(".local").join("bin"));
        extra_paths.push(home.join(".cargo").join("bin"));
    }

    for path in extra_paths {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    env::join_paths(paths).unwrap_or(current_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keychain_access_is_limited_to_cursor_access_token() {
        assert_eq!(CURSOR_ACCESS_TOKEN_SERVICE, "cursor-access-token");
    }

    #[test]
    fn network_access_is_limited_to_cursor_usage_api() {
        assert_eq!(
            CURSOR_USAGE_URL,
            "https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage"
        );
    }

    #[test]
    fn cli_availability_check_rejects_commands_outside_whitelist() {
        assert!(!allowed_cli_command_is_available("sh"));
        assert!(!allowed_cli_command_is_available("security"));
        assert!(!allowed_cli_command_is_available("curl"));
    }

    #[test]
    fn external_urls_are_limited_to_setup_links() {
        assert!(is_allowed_external_url(
            "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md"
        ));
        assert!(is_allowed_external_url(
            "https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md"
        ));
        assert!(!is_allowed_external_url("https://example.com"));
        assert!(!is_allowed_external_url(
            "https://github.com/md2it/ai-limits"
        ));
    }

    #[test]
    fn local_roots_match_documented_provider_paths() {
        let home = PathBuf::from(env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
        let roots = claude_local_roots().expect("HOME should be available in tests");
        assert!(roots.contains(&home.join(".config").join("claude").join("projects")));
        assert!(roots.contains(&home.join(".claude").join("projects")));
        assert!(roots.contains(
            &home
                .join("Library")
                .join("Developer")
                .join("Xcode")
                .join("CodingAssistant")
                .join("ClaudeAgentConfig")
                .join("projects")
        ));
    }
}
