//! MIG-002 — the migration guarantee, made testable.
//!
//! For every schema version below the one this build writes, a hand-authored
//! fixture (see `fixtures`) is opened through the real `Store::open` path —
//! the migrator runs — and the resulting schema plus every seeded row are
//! asserted. The empty database (v0), the already-current database (v5), and
//! a database written by a NEWER build (refused by name, never silently
//! downgraded) are covered too.
//!
//! A failure in this suite is a migration defect: per the Wave-8 kernel
//! addendum's synchronization law it becomes a focused fix work order —
//! never a silent fix inside this suite.

mod fixtures;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use codex_storage::{Store, StoreError};
use rusqlite::Connection;

use fixtures::SchemaVersion;

/// Which seeded rows the fixture database at a given version already
/// contains; everything listed must survive the forward migration untouched,
/// and everything else must be created empty.
struct PreservedRows {
    workspace_metadata: bool,
    browser_downloads: bool,
    workspace_folders: bool,
    browsing_history: bool,
}

/// One migration-guarantee case: build the version fixture, open it with the
/// real store (the migrator runs inside `Store::open`), then assert the
/// resulting schema and the preserved rows through the public API — the same
/// reads the app performs.
fn run_forward_migration_case(
    version: SchemaVersion,
    preserved: PreservedRows,
) -> Result<(), Box<dyn Error>> {
    let sandbox = Sandbox::new(&version.to_string())?;
    let database_path = sandbox.database_path();
    fixtures::build(&database_path, version)?;

    // The guarantee itself: an old state.sqlite3 opens on a new build.
    let store = Store::open(&database_path)?;

    // The resulting schema is exactly the documented current schema.
    {
        let inspection = Connection::open(&database_path)?;
        fixtures::assert_current_schema(&inspection)?;
    }

    // The seeded rows survived, read through the same API the app uses.
    for (key, value) in fixtures::expected_preferences() {
        assert_eq!(store.preference(key)?.as_deref(), Some(value));
    }
    assert_eq!(
        store.recent_workspaces(10, 0)?.items,
        fixtures::expected_recent_workspaces(
            preserved.workspace_metadata,
            preserved.workspace_folders
        )
    );
    let downloads = store.browser_downloads(10, 0)?.items;
    if preserved.browser_downloads {
        assert_eq!(downloads, fixtures::expected_browser_downloads());
    } else {
        assert!(
            downloads.is_empty(),
            "browser_downloads must be created empty by the migration"
        );
    }
    let history = store.browsing_history(10, 0)?.items;
    if preserved.browsing_history {
        assert_eq!(history, fixtures::expected_browsing_history());
    } else {
        assert!(
            history.is_empty(),
            "browsing_history must be created empty by the migration"
        );
    }
    Ok(())
}

/// A per-test scratch directory under the system temp dir, removed on drop.
struct Sandbox {
    directory: PathBuf,
}

impl Sandbox {
    fn new(label: &str) -> Result<Self, Box<dyn Error>> {
        let directory = std::env::temp_dir().join(format!(
            "codex-storage-mig-002-{}-{label}",
            std::process::id()
        ));
        fs::create_dir_all(&directory)?;
        Ok(Self { directory })
    }

    fn database_path(&self) -> PathBuf {
        self.directory.join("state.sqlite3")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn fresh_database_lands_on_the_current_schema() -> Result<(), Box<dyn Error>> {
    let sandbox = Sandbox::new("v0-fresh")?;
    let database_path = sandbox.database_path();
    fixtures::build(&database_path, SchemaVersion::V0)?;

    let store = Store::open(&database_path)?;

    {
        let inspection = Connection::open(&database_path)?;
        fixtures::assert_current_schema(&inspection)?;
    }
    assert_eq!(store.preference("route")?.as_deref(), None);
    assert!(store.recent_workspaces(10, 0)?.items.is_empty());
    assert!(store.browser_downloads(10, 0)?.items.is_empty());
    assert!(store.browsing_history(10, 0)?.items.is_empty());
    Ok(())
}

#[test]
fn version_one_storage_migrates_forward_preserving_rows() -> Result<(), Box<dyn Error>> {
    run_forward_migration_case(
        SchemaVersion::V1,
        PreservedRows {
            workspace_metadata: false,
            browser_downloads: false,
            workspace_folders: false,
            browsing_history: false,
        },
    )
}

#[test]
fn version_two_storage_migrates_forward_preserving_rows() -> Result<(), Box<dyn Error>> {
    run_forward_migration_case(
        SchemaVersion::V2,
        PreservedRows {
            workspace_metadata: false,
            browser_downloads: true,
            workspace_folders: false,
            browsing_history: false,
        },
    )
}

#[test]
fn version_three_storage_migrates_forward_preserving_rows() -> Result<(), Box<dyn Error>> {
    run_forward_migration_case(
        SchemaVersion::V3,
        PreservedRows {
            workspace_metadata: true,
            browser_downloads: true,
            workspace_folders: false,
            browsing_history: false,
        },
    )
}

#[test]
fn version_four_storage_migrates_forward_preserving_rows() -> Result<(), Box<dyn Error>> {
    run_forward_migration_case(
        SchemaVersion::V4,
        PreservedRows {
            workspace_metadata: true,
            browser_downloads: true,
            workspace_folders: true,
            browsing_history: false,
        },
    )
}

#[test]
fn current_version_storage_opens_unchanged() -> Result<(), Box<dyn Error>> {
    run_forward_migration_case(
        SchemaVersion::V5,
        PreservedRows {
            workspace_metadata: true,
            browser_downloads: true,
            workspace_folders: true,
            browsing_history: true,
        },
    )
}

#[test]
fn future_version_storage_is_refused_never_downgraded() -> Result<(), Box<dyn Error>> {
    let sandbox = Sandbox::new("future-refusal")?;
    let database_path = sandbox.database_path();
    fixtures::build_future(&database_path)?;

    let error = match Store::open(&database_path) {
        Err(error) => error,
        Ok(_) => panic!("a database newer than this build was accepted without the named refusal"),
    };
    match &error {
        StoreError::UnsupportedSchema(version) => {
            assert_eq!(*version, fixtures::REFUSED_FUTURE_USER_VERSION);
            // The named honest refusal — the exact user-visible text.
            assert_eq!(
                error.to_string(),
                format!(
                    "storage schema version {} is newer than this build",
                    fixtures::REFUSED_FUTURE_USER_VERSION
                )
            );
        }
        other => panic!("unexpected error opening a future-version database: {other}"),
    }

    // The refusal leaves the database exactly as it was — never a silent
    // downgrade, never a rewrite.
    let inspection = Connection::open(&database_path)?;
    let user_version: i64 = inspection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    assert_eq!(
        user_version,
        fixtures::REFUSED_FUTURE_USER_VERSION,
        "the refusal must not rewrite the schema version"
    );
    let preferences: i64 =
        inspection.query_row("SELECT COUNT(*) FROM ui_preferences", [], |row| row.get(0))?;
    assert_eq!(preferences, 2, "the refusal must not touch stored rows");
    Ok(())
}
