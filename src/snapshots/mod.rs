use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::get_limits::has_usable_limit_data;
use crate::types::StructuredSourceInfo;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotProvider {
    Codex,
    Claude,
    Cursor,
}

impl SnapshotProvider {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    fn file_name(self) -> &'static str {
        match self {
            Self::Codex => "codex.json",
            Self::Claude => "claude.json",
            Self::Cursor => "cursor.json",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SnapshotStore {
    snapshots_dir: PathBuf,
}

impl SnapshotStore {
    pub fn new(app_group_dir: impl Into<PathBuf>) -> Self {
        Self {
            snapshots_dir: app_group_dir.into().join("snapshots"),
        }
    }

    pub fn read(&self, provider: SnapshotProvider) -> io::Result<Option<StructuredSourceInfo>> {
        let path = self.snapshot_path(provider);
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(serde_json::from_slice(&bytes)
            .ok()
            .filter(|info: &StructuredSourceInfo| {
                SnapshotProvider::parse(&info.provider) == Some(provider)
            }))
    }

    pub fn replace(&self, info: &StructuredSourceInfo) -> io::Result<bool> {
        if !has_usable_limit_data(info) {
            return Ok(false);
        }
        let provider = SnapshotProvider::parse(&info.provider).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported snapshot provider `{}`", info.provider),
            )
        })?;
        let serialized = serde_json::to_vec_pretty(info).map_err(io::Error::other)?;
        fs::create_dir_all(&self.snapshots_dir)?;
        let destination = self.snapshot_path(provider);
        let temporary = self.unique_temporary_path(provider);
        let result = write_and_replace(&temporary, &destination, &serialized);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map(|()| true)
    }

    fn snapshot_path(&self, provider: SnapshotProvider) -> PathBuf {
        self.snapshots_dir.join(provider.file_name())
    }

    fn unique_temporary_path(&self, provider: SnapshotProvider) -> PathBuf {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        self.snapshots_dir.join(format!(
            ".{}.{}.{}.{}.tmp",
            provider.file_name(),
            std::process::id(),
            timestamp,
            sequence
        ))
    }
}

fn write_and_replace(temporary: &Path, destination: &Path, serialized: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)?;
    file.write_all(serialized)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AccountInfo, LimitInfo, SourceStatus, UsageInfo};
    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("ai-limits-{name}-{}-{unique}", std::process::id()))
    }

    fn snapshot(provider: &str, source: &str, usable: bool) -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: provider.to_string(),
            source: source.to_string(),
            source_link: "docs/get-limits".to_string(),
            status: SourceStatus {
                data_available: usable,
                access_available: usable,
                message: None,
            },
            raw_data_available: false,
            collected_at: Some("2026-07-26T12:00:00Z".to_string()),
            data_as_of: None,
            account: AccountInfo::default(),
            limits: if usable {
                vec![LimitInfo {
                    name: "five_hour".to_string(),
                    ..Default::default()
                }]
            } else {
                Vec::new()
            },
            available_limit_resets: None,
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn writes_structured_info_directly_and_reads_it_back() {
        let root = test_dir("round-trip");
        let store = SnapshotStore::new(&root);
        let info = snapshot("codex", "codex_cli", true);

        assert!(store.replace(&info).expect("snapshot writes"));
        assert_eq!(
            store.read(SnapshotProvider::Codex).expect("snapshot reads"),
            Some(info)
        );
        let value: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("snapshots/codex.json")).expect("snapshot file reads"),
        )
        .expect("snapshot JSON decodes");
        assert_eq!(value["provider"], "codex");
        assert!(value.get("structured").is_none());

        fs::remove_dir_all(root).expect("test directory removes");
    }

    #[test]
    fn unusable_result_leaves_previous_snapshot_untouched() {
        let root = test_dir("unusable");
        let store = SnapshotStore::new(&root);
        let previous = snapshot("claude", "claude_cli", true);
        store.replace(&previous).expect("initial snapshot writes");

        assert!(!store
            .replace(&snapshot("claude", "claude_local", false))
            .expect("unusable snapshot is rejected"));
        assert_eq!(
            store
                .read(SnapshotProvider::Claude)
                .expect("snapshot reads"),
            Some(previous)
        );

        fs::remove_dir_all(root).expect("test directory removes");
    }

    #[test]
    fn missing_and_malformed_snapshots_are_unavailable() {
        let root = test_dir("unavailable");
        let store = SnapshotStore::new(&root);
        assert_eq!(
            store
                .read(SnapshotProvider::Cursor)
                .expect("missing snapshot is tolerated"),
            None
        );
        fs::create_dir_all(root.join("snapshots")).expect("snapshots directory creates");
        fs::write(root.join("snapshots/cursor.json"), b"{not json")
            .expect("malformed snapshot writes");

        assert_eq!(
            store
                .read(SnapshotProvider::Cursor)
                .expect("malformed snapshot is tolerated"),
            None
        );

        fs::remove_dir_all(root).expect("test directory removes");
    }

    #[test]
    fn snapshot_for_another_provider_is_unavailable() {
        let root = test_dir("provider-mismatch");
        let store = SnapshotStore::new(&root);
        fs::create_dir_all(root.join("snapshots")).expect("snapshots directory creates");
        let claude = serde_json::to_vec(&snapshot("claude", "claude_cli", true))
            .expect("snapshot serializes");
        fs::write(root.join("snapshots/codex.json"), claude).expect("mismatched snapshot writes");

        assert_eq!(
            store
                .read(SnapshotProvider::Codex)
                .expect("mismatched snapshot is tolerated"),
            None
        );

        fs::remove_dir_all(root).expect("test directory removes");
    }

    #[test]
    fn each_provider_uses_an_independent_file() {
        let root = test_dir("providers");
        let store = SnapshotStore::new(&root);
        let codex = snapshot("codex", "codex_local", true);
        let cursor = snapshot("cursor", "cursor_api2", true);

        store.replace(&codex).expect("Codex snapshot writes");
        store.replace(&cursor).expect("Cursor snapshot writes");

        assert_eq!(
            store
                .read(SnapshotProvider::Codex)
                .expect("Codex snapshot reads"),
            Some(codex)
        );
        assert_eq!(
            store
                .read(SnapshotProvider::Cursor)
                .expect("Cursor snapshot reads"),
            Some(cursor)
        );

        fs::remove_dir_all(root).expect("test directory removes");
    }
}
