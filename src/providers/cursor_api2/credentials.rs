use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::transport::Failure;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    Ide,
    Cli,
}

impl CredentialSource {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "ide" => Ok(Self::Ide),
            "cli" => Ok(Self::Cli),
            _ => Err("Unsupported Cursor credential source"),
        }
    }
}

#[derive(Clone)]
pub(super) struct Credential {
    pub source: CredentialSource,
    pub access: String,
    pub refresh: Option<String>,
    pub subject: Option<String>,
    pub expires: Option<i64>,
}

impl Credential {
    pub fn new(
        source: CredentialSource,
        access: String,
        refresh: Option<String>,
    ) -> Result<Self, Failure> {
        const PROTECTED_PREFIX: &str = "cursor-safe-storage:v1:";
        if access.starts_with(PROTECTED_PREFIX)
            || refresh
                .as_deref()
                .is_some_and(|value| value.starts_with(PROTECTED_PREFIX))
        {
            return Err(Failure::Protected);
        }
        if access.trim().is_empty()
            || access.chars().any(|c| c.is_control())
            || refresh
                .as_ref()
                .is_some_and(|s| s.chars().any(|c| c.is_control()))
        {
            return Err(Failure::Malformed);
        }
        let claims = claims(&access);
        Ok(Self {
            source,
            subject: claims.get("sub").and_then(Value::as_str).map(str::to_owned),
            expires: claims.get("exp").and_then(Value::as_i64),
            access,
            refresh: refresh.filter(|v| !v.is_empty()),
        })
    }
    pub fn same_account(&self, other: &Self) -> bool {
        self.subject.is_some() && self.subject == other.subject
    }
}

// JWT claims are untrusted routing/scheduling hints. Only the API authorizes a request.
fn claims(token: &str) -> Value {
    token
        .split('.')
        .nth(1)
        .and_then(decode_base64)
        .and_then(|s| serde_json::from_slice(&s).ok())
        .unwrap_or(Value::Null)
}

fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut bits = 0u32;
    let mut count = 0;
    for c in text.bytes().take_while(|c| *c != b'=') {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return None,
        };
        bits = (bits << 6) | u32::from(value);
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    Some(out)
}

pub(super) trait Store {
    fn read(&mut self, source: CredentialSource) -> Result<Credential, Failure>;
}
pub(super) struct OsStore;
impl Store for OsStore {
    fn read(&mut self, source: CredentialSource) -> Result<Credential, Failure> {
        if source == CredentialSource::Cli && cfg!(target_os = "macos") {
            let access = read_keychain("cursor-access-token")?;
            let refresh = read_keychain("cursor-refresh-token").ok();
            return Credential::new(source, access, refresh);
        }
        let home = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
            .map(PathBuf::from)
            .ok_or(Failure::Unreadable)?;
        let (ide, cli) = paths(
            env::consts::OS,
            &home,
            env::var_os("APPDATA").map(PathBuf::from),
            env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        );
        if source == CredentialSource::Ide {
            read_ide(&ide)
        } else {
            read_cli(&cli)
        }
    }
}

fn read_keychain(service: &str) -> Result<String, Failure> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-a",
            "cursor-user",
            "-s",
            service,
            "-w",
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|_| Failure::Unreadable)?;
    if !output.status.success() {
        return Err(if output.status.code() == Some(44) {
            Failure::Missing
        } else {
            Failure::Unreadable
        });
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|_| Failure::Malformed)
}

fn paths(
    os: &str,
    home: &Path,
    appdata: Option<PathBuf>,
    xdg: Option<PathBuf>,
) -> (PathBuf, PathBuf) {
    let config = xdg.unwrap_or_else(|| home.join(".config"));
    let (ide, cli) = match os {
        "macos" => (
            home.join("Library/Application Support/Cursor"),
            home.join(".cursor"),
        ),
        "windows" => {
            let base = appdata.unwrap_or_else(|| home.join("AppData/Roaming"));
            (base.join("Cursor"), base.join("Cursor"))
        }
        _ => (config.join("Cursor"), config.join("cursor")),
    };
    (
        ide.join("User/globalStorage/state.vscdb"),
        cli.join("auth.json"),
    )
}

fn read_cli(path: &Path) -> Result<Credential, Failure> {
    let content = fs::read(path).map_err(file_error)?;
    let value: Value = serde_json::from_slice(&content).map_err(|_| Failure::Malformed)?;
    let access = value
        .get("accessToken")
        .and_then(Value::as_str)
        .ok_or(Failure::Missing)?;
    Credential::new(
        CredentialSource::Cli,
        access.to_owned(),
        value
            .get("refreshToken")
            .and_then(Value::as_str)
            .map(str::to_owned),
    )
}

fn file_error(error: std::io::Error) -> Failure {
    if error.kind() == std::io::ErrorKind::NotFound {
        Failure::Missing
    } else {
        Failure::Unreadable
    }
}

fn read_ide(path: &Path) -> Result<Credential, Failure> {
    if !path.try_exists().map_err(file_error)? {
        return Err(Failure::Missing);
    }
    let output = std::process::Command::new("sqlite3")
        .arg("-readonly").arg("-json").arg("-cmd").arg(".timeout 250").arg(path)
        .arg("SELECT key, value FROM ItemTable WHERE key IN ('cursorAuth/accessToken', 'cursorAuth/refreshToken')")
        .stdin(std::process::Stdio::null()).output().map_err(|error| if error.kind()==std::io::ErrorKind::NotFound { Failure::SqliteUnavailable } else { Failure::Unreadable })?;
    if !output.status.success() {
        return Err(Failure::Unreadable);
    }
    if output.stdout.iter().all(u8::is_ascii_whitespace) {
        return Err(Failure::Missing);
    }
    let rows: Vec<Value> =
        serde_json::from_slice(&output.stdout).map_err(|_| Failure::Malformed)?;
    let mut access = None;
    let mut refresh = None;
    for row in rows {
        let key = row
            .get("key")
            .and_then(Value::as_str)
            .ok_or(Failure::Malformed)?;
        let value = row
            .get("value")
            .and_then(Value::as_str)
            .ok_or(Failure::Malformed)?
            .to_owned();
        if key == "cursorAuth/accessToken" {
            access = Some(value);
        } else {
            refresh = Some(value);
        }
    }
    Credential::new(
        CredentialSource::Ide,
        access.ok_or(Failure::Missing)?,
        refresh,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_follow_platform_user_context() {
        let home = Path::new("/users/test");
        assert_eq!(
            paths("linux", home, None, Some("/xdg".into())).1,
            PathBuf::from("/xdg/cursor/auth.json")
        );
        assert_eq!(
            paths("windows", home, None, None).1,
            home.join("AppData/Roaming/Cursor/auth.json")
        );
        assert_eq!(
            paths("windows", home, Some("/roaming".into()), None).0,
            PathBuf::from("/roaming/Cursor/User/globalStorage/state.vscdb")
        );
        assert_eq!(
            paths("macos", home, None, None).0,
            home.join("Library/Application Support/Cursor/User/globalStorage/state.vscdb")
        );
    }
    #[test]
    fn claims_are_optional_and_control_characters_rejected() {
        let token = Credential::new(
            CredentialSource::Ide,
            "a.eyJzdWIiOiJ0ZXN0IiwiZXhwIjoxMjN9.c".into(),
            None,
        )
        .unwrap();
        assert_eq!(token.subject.as_deref(), Some("test"));
        assert_eq!(token.expires, Some(123));
        assert!(Credential::new(CredentialSource::Ide, "a\nb".into(), None).is_err());
        assert!(
            Credential::new(CredentialSource::Ide, "unknown".into(), None)
                .unwrap()
                .subject
                .is_none()
        );
    }
    #[test]
    fn sqlite_reads_only_requested_credentials_without_creating_missing_store() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".temp")
            .join(format!("cursor-reader-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("state.vscdb");
        assert!(matches!(read_ide(&path), Err(Failure::Missing)));
        assert!(!path.exists());
        let setup=std::process::Command::new("sqlite3").arg(&path).arg("CREATE TABLE ItemTable (key TEXT, value TEXT); INSERT INTO ItemTable VALUES ('cursorAuth/accessToken','test-access'),('cursorAuth/refreshToken','test-refresh'),('unrelated','private');").output();
        if let Ok(setup) = setup {
            assert!(setup.status.success());
            let credential = read_ide(&path).unwrap();
            assert_eq!(credential.access, "test-access");
            assert_eq!(credential.refresh.as_deref(), Some("test-refresh"));
            let before = fs::read(&path).unwrap();
            read_ide(&path).unwrap();
            assert_eq!(before, fs::read(&path).unwrap());
            let cleared = std::process::Command::new("sqlite3")
                .arg(&path)
                .arg("DELETE FROM ItemTable")
                .output()
                .unwrap();
            assert!(cleared.status.success());
            let empty = std::process::Command::new("sqlite3")
                .arg("-readonly")
                .arg("-json")
                .arg(&path)
                .arg("SELECT key,value FROM ItemTable")
                .output()
                .unwrap();
            assert!(empty.stdout.is_empty());
            assert!(matches!(read_ide(&path), Err(Failure::Missing)));
        }
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn rejects_protected_access_and_refresh_containers() {
        let container = "cursor-safe-storage:v1:fixture";
        assert!(matches!(
            Credential::new(CredentialSource::Ide, container.into(), None),
            Err(Failure::Protected)
        ));
        assert!(matches!(
            Credential::new(
                CredentialSource::Ide,
                "access".into(),
                Some(container.into())
            ),
            Err(Failure::Protected)
        ));
    }
}
