//! MIG-002 — the hand-authored migration fixtures for `codex-storage`.
//!
//! One schema constant per historical version carries the exact SQL a
//! database at that version had (restated from the migrator's own history in
//! `src/lib.rs`), together with small, credential-free, representative rows
//! per table (the E2B law: no production data, no credentials, no secrets).
//! The expected current schema — tables, named indexes, columns — is
//! restated here as well, so the migration tests assert exactly what the
//! migrator must land on.
//!
//! Introducing a new migration step REQUIRES extending this module with the
//! new version's fixture and bumping [`CURRENT_USER_VERSION`]: that is the
//! point of MIG-002 — the forward-migration guarantee is enforced by tests,
//! not by convention.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use codex_storage::{
    BrowserDownloadRecordStatus, RecentWorkspace, StoredBrowserDownload, StoredBrowsingHistoryEntry,
};
use rusqlite::{Connection, params};

/// The schema version the current build writes and migrates to. Keep in sync
/// with the migrator's `SCHEMA_VERSION` (`PRAGMA user_version = 5` at the
/// Wave-8b dispatch pin).
pub const CURRENT_USER_VERSION: i64 = 5;

/// A database one version ahead of the build — the downgrade-refusal case
/// (a user downgraded the app below the build that wrote their data).
pub const REFUSED_FUTURE_USER_VERSION: i64 = CURRENT_USER_VERSION + 1;

/// A schema version the migrator has historically written.
#[derive(Debug, Clone, Copy)]
pub enum SchemaVersion {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
}

impl SchemaVersion {
    pub const fn user_version(self) -> i64 {
        self as i64
    }

    const fn at_least(self, other: Self) -> bool {
        self as i64 >= other as i64
    }

    const fn schema_sql(self) -> Option<&'static str> {
        match self {
            Self::V0 => None,
            Self::V1 => Some(V1_SCHEMA),
            Self::V2 => Some(V2_SCHEMA),
            Self::V3 => Some(V3_SCHEMA),
            Self::V4 => Some(V4_SCHEMA),
            Self::V5 => Some(V5_SCHEMA),
        }
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "v{}", self.user_version())
    }
}

const V1_SCHEMA: &str = "CREATE TABLE ui_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE recent_workspaces (
    path BLOB PRIMARY KEY NOT NULL,
    last_opened_at INTEGER NOT NULL
) STRICT;
PRAGMA user_version = 1;";

const V2_SCHEMA: &str = "CREATE TABLE ui_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE recent_workspaces (
    path BLOB PRIMARY KEY NOT NULL,
    last_opened_at INTEGER NOT NULL
) STRICT;
CREATE TABLE browser_downloads (
    id TEXT PRIMARY KEY NOT NULL,
    context_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    path BLOB NOT NULL,
    received_bytes INTEGER NOT NULL CHECK(received_bytes >= 0),
    started_at_ms INTEGER NOT NULL CHECK(started_at_ms >= 0),
    status INTEGER NOT NULL CHECK(status IN (0, 1, 2)),
    total_bytes INTEGER NOT NULL CHECK(total_bytes >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    user_initiated INTEGER NOT NULL CHECK(user_initiated IN (0, 1))
) STRICT;
CREATE INDEX browser_downloads_updated
ON browser_downloads(updated_at_ms DESC, id ASC);
PRAGMA user_version = 2;";

const V3_SCHEMA: &str = "CREATE TABLE ui_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE recent_workspaces (
    path BLOB PRIMARY KEY NOT NULL,
    last_opened_at INTEGER NOT NULL,
    name TEXT,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0, 1))
) STRICT;
CREATE TABLE browser_downloads (
    id TEXT PRIMARY KEY NOT NULL,
    context_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    path BLOB NOT NULL,
    received_bytes INTEGER NOT NULL CHECK(received_bytes >= 0),
    started_at_ms INTEGER NOT NULL CHECK(started_at_ms >= 0),
    status INTEGER NOT NULL CHECK(status IN (0, 1, 2)),
    total_bytes INTEGER NOT NULL CHECK(total_bytes >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    user_initiated INTEGER NOT NULL CHECK(user_initiated IN (0, 1))
) STRICT;
CREATE INDEX browser_downloads_updated
ON browser_downloads(updated_at_ms DESC, id ASC);
PRAGMA user_version = 3;";

const V4_SCHEMA: &str = "CREATE TABLE ui_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE recent_workspaces (
    path BLOB PRIMARY KEY NOT NULL,
    last_opened_at INTEGER NOT NULL,
    name TEXT,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0, 1))
) STRICT;
CREATE TABLE browser_downloads (
    id TEXT PRIMARY KEY NOT NULL,
    context_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    path BLOB NOT NULL,
    received_bytes INTEGER NOT NULL CHECK(received_bytes >= 0),
    started_at_ms INTEGER NOT NULL CHECK(started_at_ms >= 0),
    status INTEGER NOT NULL CHECK(status IN (0, 1, 2)),
    total_bytes INTEGER NOT NULL CHECK(total_bytes >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    user_initiated INTEGER NOT NULL CHECK(user_initiated IN (0, 1))
) STRICT;
CREATE INDEX browser_downloads_updated
ON browser_downloads(updated_at_ms DESC, id ASC);
CREATE TABLE workspace_folders (
    workspace_path BLOB NOT NULL,
    folder_path BLOB NOT NULL,
    position INTEGER NOT NULL CHECK(position >= 0),
    PRIMARY KEY (workspace_path, folder_path)
) STRICT;
CREATE INDEX workspace_folders_position
ON workspace_folders(workspace_path, position ASC);
PRAGMA user_version = 4;";

const V5_SCHEMA: &str = "CREATE TABLE ui_preferences (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE recent_workspaces (
    path BLOB PRIMARY KEY NOT NULL,
    last_opened_at INTEGER NOT NULL,
    name TEXT,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0, 1))
) STRICT;
CREATE TABLE browser_downloads (
    id TEXT PRIMARY KEY NOT NULL,
    context_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    path BLOB NOT NULL,
    received_bytes INTEGER NOT NULL CHECK(received_bytes >= 0),
    started_at_ms INTEGER NOT NULL CHECK(started_at_ms >= 0),
    status INTEGER NOT NULL CHECK(status IN (0, 1, 2)),
    total_bytes INTEGER NOT NULL CHECK(total_bytes >= 0),
    updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
    user_initiated INTEGER NOT NULL CHECK(user_initiated IN (0, 1))
) STRICT;
CREATE INDEX browser_downloads_updated
ON browser_downloads(updated_at_ms DESC, id ASC);
CREATE TABLE workspace_folders (
    workspace_path BLOB NOT NULL,
    folder_path BLOB NOT NULL,
    position INTEGER NOT NULL CHECK(position >= 0),
    PRIMARY KEY (workspace_path, folder_path)
) STRICT;
CREATE INDEX workspace_folders_position
ON workspace_folders(workspace_path, position ASC);
CREATE TABLE browsing_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    visited_at_ms INTEGER NOT NULL CHECK(visited_at_ms >= 0)
) STRICT;
CREATE INDEX browsing_history_visited
ON browsing_history(visited_at_ms DESC, id DESC);
PRAGMA user_version = 5;";

/// The related folders of the first seed workspace, in position order.
const SEED_WORKSPACE_FOLDERS: &[&str] = &["shared", "docs"];

/// UI preference rows present since v1. Oldest-updated first.
const SEED_PREFERENCES: &[(&str, &str, i64)] =
    &[("route", "repository", 110), ("ui.density", "compact", 120)];

pub struct SeedWorkspace {
    /// Directory name under the seed base; the workspace path itself.
    pub directory: &'static str,
    pub last_opened_at: i64,
    pub name: Option<&'static str>,
    pub pinned: bool,
}

/// Recent-workspace rows present since v1. Oldest-opened first; the store
/// lists them most-recently-opened first.
const SEED_WORKSPACES: &[SeedWorkspace] = &[
    SeedWorkspace {
        directory: "project-a",
        last_opened_at: 1_000,
        name: None,
        pinned: false,
    },
    SeedWorkspace {
        directory: "project-b",
        last_opened_at: 2_000,
        name: Some("Project B"),
        pinned: true,
    },
];

pub struct SeedDownload {
    pub id: &'static str,
    pub context_id: &'static str,
    pub filename: &'static str,
    pub received_bytes: u64,
    pub started_at_ms: u64,
    pub status: BrowserDownloadRecordStatus,
    pub total_bytes: u64,
    pub updated_at_ms: u64,
    pub user_initiated: bool,
}

/// Browser-download rows present since v2. Oldest-updated first; the store
/// lists them most-recently-updated first.
const SEED_DOWNLOADS: &[SeedDownload] = &[
    SeedDownload {
        id: "dl-001",
        context_id: "ctx-alpha",
        filename: "draft.txt",
        received_bytes: 1_024,
        started_at_ms: 4_000,
        status: BrowserDownloadRecordStatus::Canceled,
        total_bytes: 8_192,
        updated_at_ms: 4_500,
        user_initiated: false,
    },
    SeedDownload {
        id: "dl-002",
        context_id: "ctx-beta",
        filename: "report.txt",
        received_bytes: 2_048,
        started_at_ms: 5_000,
        status: BrowserDownloadRecordStatus::Complete,
        total_bytes: 4_096,
        updated_at_ms: 5_100,
        user_initiated: true,
    },
];

pub struct SeedBrowsingVisit {
    pub url: &'static str,
    pub title: Option<&'static str>,
    pub visited_at_ms: u64,
}

/// Browsing-history rows present since v5. Oldest-visited first; the store
/// lists them most-recently-visited first.
const SEED_BROWSING_VISITS: &[SeedBrowsingVisit] = &[
    SeedBrowsingVisit {
        url: "https://example.com/",
        title: None,
        visited_at_ms: 7_000,
    },
    SeedBrowsingVisit {
        url: "https://docs.rs/rust/",
        title: Some("Rust docs"),
        visited_at_ms: 8_000,
    },
];

pub struct ExpectedColumn {
    pub name: &'static str,
    pub column_type: &'static str,
    pub not_null: bool,
    pub primary_key: bool,
}

pub struct ExpectedTable {
    pub name: &'static str,
    pub columns: &'static [ExpectedColumn],
}

/// The schema the current build must land on after a forward migration —
/// restated so the tests assert it explicitly instead of trusting the
/// migrator's internal constants.
pub const EXPECTED_CURRENT_TABLES: &[ExpectedTable] = &[
    ExpectedTable {
        name: "ui_preferences",
        columns: &[
            ExpectedColumn {
                name: "key",
                column_type: "TEXT",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "value",
                column_type: "TEXT",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "updated_at",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
        ],
    },
    ExpectedTable {
        name: "recent_workspaces",
        columns: &[
            ExpectedColumn {
                name: "path",
                column_type: "BLOB",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "last_opened_at",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "name",
                column_type: "TEXT",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "pinned",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
        ],
    },
    ExpectedTable {
        name: "browser_downloads",
        columns: &[
            ExpectedColumn {
                name: "id",
                column_type: "TEXT",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "context_id",
                column_type: "TEXT",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "filename",
                column_type: "TEXT",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "path",
                column_type: "BLOB",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "received_bytes",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "started_at_ms",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "status",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "total_bytes",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "updated_at_ms",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "user_initiated",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
        ],
    },
    ExpectedTable {
        name: "workspace_folders",
        columns: &[
            ExpectedColumn {
                name: "workspace_path",
                column_type: "BLOB",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "folder_path",
                column_type: "BLOB",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "position",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
        ],
    },
    ExpectedTable {
        name: "browsing_history",
        columns: &[
            ExpectedColumn {
                name: "id",
                column_type: "INTEGER",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "url",
                column_type: "TEXT",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "title",
                column_type: "TEXT",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "visited_at_ms",
                column_type: "INTEGER",
                not_null: true,
                primary_key: false,
            },
        ],
    },
];

/// The named (non-implicit) indexes of the current schema.
pub const EXPECTED_CURRENT_INDEXES: &[&str] = &[
    "browser_downloads_updated",
    "browsing_history_visited",
    "workspace_folders_position",
];

/// The absolute directory every seed path is rooted at. Absolute on both CI
/// platforms, so the seeded rows look like real rows to the store.
pub fn seed_base() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\flauz")
    } else {
        PathBuf::from("/flauz")
    }
}

fn workspace_path(workspace: &SeedWorkspace) -> PathBuf {
    seed_base().join(workspace.directory)
}

fn download_path(download: &SeedDownload) -> PathBuf {
    seed_base().join("Downloads").join(download.filename)
}

fn folder_path(directory: &str) -> PathBuf {
    seed_base().join(directory)
}

/// Mirrors the store's private per-platform path encoding (Unix: raw bytes;
/// Windows: UTF-16LE), so fixture rows carry exactly the bytes a real
/// database at that version would carry on the running platform.
#[cfg(unix)]
fn encode_seed_path(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str().as_bytes().to_vec()
}

#[cfg(windows)]
fn encode_seed_path(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// Mirrors the store's u64-to-SQLite-integer conversion for seed constants.
fn sqlite_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Mirrors the store's private download-status encoding.
fn status_as_i64(status: BrowserDownloadRecordStatus) -> i64 {
    match status {
        BrowserDownloadRecordStatus::Failed => 0,
        BrowserDownloadRecordStatus::Canceled => 1,
        BrowserDownloadRecordStatus::Complete => 2,
    }
}

/// Builds a database at `version` with its hand-authored schema and the
/// representative seed rows that version's tables carry. Hard-fails with a
/// named message if the fixture does not land at the expected
/// `PRAGMA user_version` (a misauthored fixture, never a silent pass).
pub fn build(path: &Path, version: SchemaVersion) -> Result<(), Box<dyn Error>> {
    let connection = Connection::open(path)?;
    if let Some(schema) = version.schema_sql() {
        connection.execute_batch(schema)?;
        seed(&connection, version)?;
    }
    let landed: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if landed != version.user_version() {
        return Err(format!(
            "MIG-002 fixture guard: the {version} fixture landed at user_version {landed} \
             instead of {} — the fixture SQL is misauthored",
            version.user_version()
        )
        .into());
    }
    Ok(())
}

/// Builds a database written by a hypothetical NEWER build: the current
/// schema with its seed rows, but `PRAGMA user_version` set one step ahead.
/// (What a database beyond the current version really contains is unknowable
/// from this build; the version header is what the refusal reads.)
pub fn build_future(path: &Path) -> Result<(), Box<dyn Error>> {
    let connection = Connection::open(path)?;
    connection.execute_batch(V5_SCHEMA)?;
    seed(&connection, SchemaVersion::V5)?;
    connection.execute_batch(&format!(
        "PRAGMA user_version = {REFUSED_FUTURE_USER_VERSION};"
    ))?;
    let landed: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if landed != REFUSED_FUTURE_USER_VERSION {
        return Err(format!(
            "MIG-002 fixture guard: the future fixture landed at user_version {landed} \
             instead of {REFUSED_FUTURE_USER_VERSION}"
        )
        .into());
    }
    Ok(())
}

fn seed(connection: &Connection, version: SchemaVersion) -> Result<(), rusqlite::Error> {
    for (key, value, updated_at) in SEED_PREFERENCES {
        connection.execute(
            "INSERT INTO ui_preferences(key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, updated_at],
        )?;
    }
    for workspace in SEED_WORKSPACES {
        if version.at_least(SchemaVersion::V3) {
            connection.execute(
                "INSERT INTO recent_workspaces(path, last_opened_at, name, pinned)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    encode_seed_path(&workspace_path(workspace)),
                    workspace.last_opened_at,
                    workspace.name,
                    i64::from(workspace.pinned)
                ],
            )?;
        } else {
            connection.execute(
                "INSERT INTO recent_workspaces(path, last_opened_at) VALUES (?1, ?2)",
                params![
                    encode_seed_path(&workspace_path(workspace)),
                    workspace.last_opened_at
                ],
            )?;
        }
    }
    if version.at_least(SchemaVersion::V2) {
        for download in SEED_DOWNLOADS {
            connection.execute(
                "INSERT INTO browser_downloads(
                    id, context_id, filename, path, received_bytes, started_at_ms,
                    status, total_bytes, updated_at_ms, user_initiated
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    download.id,
                    download.context_id,
                    download.filename,
                    encode_seed_path(&download_path(download)),
                    sqlite_i64(download.received_bytes),
                    sqlite_i64(download.started_at_ms),
                    status_as_i64(download.status),
                    sqlite_i64(download.total_bytes),
                    sqlite_i64(download.updated_at_ms),
                    i64::from(download.user_initiated)
                ],
            )?;
        }
    }
    if version.at_least(SchemaVersion::V4) {
        for (position, directory) in SEED_WORKSPACE_FOLDERS.iter().enumerate() {
            connection.execute(
                "INSERT INTO workspace_folders(workspace_path, folder_path, position)
                 VALUES (?1, ?2, ?3)",
                params![
                    encode_seed_path(&workspace_path(&SEED_WORKSPACES[0])),
                    encode_seed_path(&folder_path(directory)),
                    i64::try_from(position).unwrap_or_default()
                ],
            )?;
        }
    }
    if version.at_least(SchemaVersion::V5) {
        for visit in SEED_BROWSING_VISITS {
            connection.execute(
                "INSERT INTO browsing_history(url, title, visited_at_ms) VALUES (?1, ?2, ?3)",
                params![visit.url, visit.title, sqlite_i64(visit.visited_at_ms)],
            )?;
        }
    }
    Ok(())
}

/// Asserts the connection carries exactly the documented current schema:
/// `PRAGMA user_version`, the full table set, the named indexes, and every
/// table's columns (name, type, NOT NULL, primary key).
pub fn assert_current_schema(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let user_version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    assert_eq!(
        user_version, CURRENT_USER_VERSION,
        "PRAGMA user_version after forward migration"
    );

    let mut tables = Vec::new();
    let mut indexes = Vec::new();
    {
        let mut statement = connection.prepare(
            "SELECT type, name FROM sqlite_master
             WHERE name NOT LIKE 'sqlite\\_%' ESCAPE '\\'
             ORDER BY type DESC, name ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (kind, name) = row?;
            match kind.as_str() {
                "table" => tables.push(name),
                "index" => indexes.push(name),
                _ => {}
            }
        }
    }
    // The guarantee is the exact table/index SET (creation order is not).
    tables.sort();
    indexes.sort();
    let mut expected_tables: Vec<&str> = EXPECTED_CURRENT_TABLES
        .iter()
        .map(|table| table.name)
        .collect();
    expected_tables.sort_unstable();
    assert_eq!(tables, expected_tables, "tables after forward migration");
    let mut expected_indexes: Vec<&str> = EXPECTED_CURRENT_INDEXES.to_vec();
    expected_indexes.sort_unstable();
    assert_eq!(
        indexes, expected_indexes,
        "named indexes after forward migration"
    );

    for table in EXPECTED_CURRENT_TABLES {
        let mut columns = Vec::new();
        {
            let mut statement =
                connection.prepare(&format!("PRAGMA table_info({})", table.name))?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(5)?,
                ))
            })?;
            for row in rows {
                let (name, column_type, not_null, primary_key) = row?;
                columns.push((name, column_type, not_null != 0, primary_key != 0));
            }
        }
        let expected: Vec<(String, String, bool, bool)> = table
            .columns
            .iter()
            .map(|column| {
                (
                    column.name.to_owned(),
                    column.column_type.to_owned(),
                    column.not_null,
                    column.primary_key,
                )
            })
            .collect();
        assert_eq!(
            columns, expected,
            "columns of {} after forward migration",
            table.name
        );
    }
    Ok(())
}

/// The seeded preferences, as the store must still read them after a
/// forward migration.
pub fn expected_preferences() -> Vec<(&'static str, &'static str)> {
    SEED_PREFERENCES
        .iter()
        .map(|(key, value, _)| (*key, *value))
        .collect()
}

/// The seeded recent workspaces in the store's list order
/// (`last_opened_at DESC, path ASC`). Databases older than v3 predate the
/// `name`/`pinned` columns, so those rows must come back with their
/// NULL/unpinned migration defaults; databases older than v4 predate
/// related folders entirely.
pub fn expected_recent_workspaces(
    workspace_metadata: bool,
    workspace_folders: bool,
) -> Vec<RecentWorkspace> {
    let mut items: Vec<RecentWorkspace> = SEED_WORKSPACES
        .iter()
        .map(|workspace| RecentWorkspace {
            path: workspace_path(workspace),
            last_opened_at: workspace.last_opened_at,
            name: if workspace_metadata {
                workspace.name.map(str::to_owned)
            } else {
                None
            },
            pinned: workspace_metadata && workspace.pinned,
            folders: Vec::new(),
        })
        .collect();
    items.reverse();
    if workspace_folders {
        let with_folders = workspace_path(&SEED_WORKSPACES[0]);
        for item in &mut items {
            if item.path == with_folders {
                item.folders = SEED_WORKSPACE_FOLDERS
                    .iter()
                    .map(|directory| folder_path(directory))
                    .collect();
            }
        }
    }
    items
}

/// The seeded browser downloads in the store's list order
/// (`updated_at_ms DESC, id ASC`).
pub fn expected_browser_downloads() -> Vec<StoredBrowserDownload> {
    let mut items: Vec<StoredBrowserDownload> = SEED_DOWNLOADS
        .iter()
        .map(|download| StoredBrowserDownload {
            context_id: download.context_id.to_owned(),
            filename: download.filename.to_owned(),
            id: download.id.to_owned(),
            path: download_path(download),
            received_bytes: download.received_bytes,
            started_at_ms: download.started_at_ms,
            status: download.status,
            total_bytes: download.total_bytes,
            updated_at_ms: download.updated_at_ms,
            user_initiated: download.user_initiated,
        })
        .collect();
    items.reverse();
    items
}

/// The seeded browsing visits in the store's list order
/// (`visited_at_ms DESC, id DESC`).
pub fn expected_browsing_history() -> Vec<StoredBrowsingHistoryEntry> {
    let mut items: Vec<StoredBrowsingHistoryEntry> = SEED_BROWSING_VISITS
        .iter()
        .map(|visit| StoredBrowsingHistoryEntry {
            url: visit.url.to_owned(),
            title: visit.title.map(str::to_owned),
            visited_at_ms: visit.visited_at_ms,
        })
        .collect();
    items.reverse();
    items
}
