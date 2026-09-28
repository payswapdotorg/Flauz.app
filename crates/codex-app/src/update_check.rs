//! UPD-001: the notify-only update check.
//!
//! The law: NOTIFY, never install. No code is ever downloaded; the check
//! reads the public releases atom feed, compares tags against the running
//! build's version, and surfaces the truth in the status surface row —
//! update available / up to date / couldn't check. A failed check is NEVER
//! reported as "up to date" (the honest-state law).
//!
//! Cadence: the automatic check runs at most once per 24 hours (the
//! persisted `updates.last_check_ms` gates it, counting failed attempts —
//! an offline day does not retry-loop the feed). The manual palette check
//! is the user's explicit request and reports its outcome through the
//! status surface. `updates.check = "off"` disables the fetch entirely.
//!
//! The feed URL and the releases-page link are named constants; no
//! credential ever appears in either (public feed only). This module never
//! emits actions itself: it returns the status line and the caller owns
//! the emission (single choke point in the backend).

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use codex_storage::Store;
use crossbeam_channel::Receiver;

/// The public releases atom feed (no credential, ever).
pub(crate) const RELEASES_FEED_URL: &str =
    "https://github.com/payswapdotorg/Flauz.app/releases.atom";
/// The releases page the affordance links to (a named constant).
pub(crate) const RELEASES_PAGE_URL: &str = "https://github.com/payswapdotorg/Flauz.app/releases";
/// The persisted knob: "on" (default) | "off".
pub(crate) const UPDATES_CHECK_PREFERENCE: &str = "updates.check";
/// The persisted last-check timestamp (unix ms, written on every attempt).
pub(crate) const UPDATES_LAST_CHECK_PREFERENCE: &str = "updates.last_check_ms";
/// At most one automatic check per 24 hours.
pub(crate) const UPDATE_CHECK_INTERVAL_MS: i64 = 24 * 60 * 60 * 1000;

/// Every external read is bounded (the kernel law).
const MAX_FEED_BYTES: u64 = 256 * 1024;
const MAX_FEED_ENTRIES: usize = 30;
const MAX_TAG_BYTES: usize = 128;
const MAX_REASON_BYTES: usize = 256;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const STATUS_BYTES: usize = 512;

// ---------------------------------------------------------------------------
// The semver-ish comparison (named pre-release handling)
// ---------------------------------------------------------------------------

/// The result of comparing a feed tag against the running version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReleaseComparison {
    /// The feed tag is strictly newer than the running build.
    Newer,
    /// Identical versions (build metadata ignored).
    Same,
    /// The feed tag is older than the running build.
    Older,
    /// Either side is not a comparable `X.Y.Z[-pre]` tag; a malformed tag
    /// can never claim an update (the honest bound).
    NotComparable,
}

/// A parsed `major.minor.patch[-prerelease]` version (build metadata
/// ignored, per semver).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<Vec<PreSegment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreSegment {
    Numeric(u64),
    Alphanumeric(String),
}

impl PreSegment {
    fn rank(&self) -> u8 {
        match self {
            // Numeric identifiers always have lower precedence than
            // alphanumeric ones (semver §11).
            Self::Numeric(_) => 0,
            Self::Alphanumeric(_) => 1,
        }
    }
}

fn parse_version(tag: &str) -> Option<ParsedVersion> {
    // Optional leading v/V, then core[-pre][+build].
    let trimmed = tag
        .strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag);
    let without_build = trimmed.split('+').next().unwrap_or(trimmed);
    let (core, pre) = match without_build.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (without_build, None),
    };
    let mut segments = core.split('.');
    let parse_segment = |segment: Option<&str>| -> Option<u64> {
        let segment = segment?;
        if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        segment.parse::<u64>().ok()
    };
    let major = parse_segment(segments.next())?;
    let minor = parse_segment(segments.next())?;
    let patch = parse_segment(segments.next())?;
    if segments.next().is_some() {
        // Exactly three core segments (X.Y.Z): "1.2" and "1.2.3.4" are not
        // comparable.
        return None;
    }
    let pre = match pre {
        None => None,
        Some("") => return None,
        Some(pre) => {
            let mut parsed = Vec::new();
            for segment in pre.split('.') {
                if segment.is_empty() {
                    return None;
                }
                if segment.bytes().all(|byte| byte.is_ascii_digit()) {
                    let Ok(value) = segment.parse::<u64>() else {
                        // An unparseably large numeric segment is malformed.
                        return None;
                    };
                    parsed.push(PreSegment::Numeric(value));
                } else {
                    parsed.push(PreSegment::Alphanumeric(segment.to_owned()));
                }
            }
            Some(parsed)
        }
    };
    Some(ParsedVersion {
        major,
        minor,
        patch,
        pre,
    })
}

impl ParsedVersion {
    /// Semver precedence: core fields numerically; a release outranks its
    /// own pre-releases; pre-release segments compare per semver §11.
    fn precedence(&self, other: &ParsedVersion) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let core =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        if core != Ordering::Equal {
            return core;
        }
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            // A version without a pre-release outranks the pre-release.
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(left), Some(right)) => {
                for (left, right) in left.iter().zip(right.iter()) {
                    let segment = match (left, right) {
                        (PreSegment::Numeric(left), PreSegment::Numeric(right)) => left.cmp(right),
                        (PreSegment::Alphanumeric(left), PreSegment::Alphanumeric(right)) => {
                            left.as_str().cmp(right.as_str())
                        }
                        _ => left.rank().cmp(&right.rank()),
                    };
                    if segment != Ordering::Equal {
                        return segment;
                    }
                }
                // All shared segments equal: the longer pre-release wins.
                left.len().cmp(&right.len())
            }
        }
    }
}

/// Compares the running version against a feed tag. Malformed tags on
/// either side are `NotComparable` and can never trigger an update notice.
pub(crate) fn compare_release_tags(running: &str, candidate: &str) -> ReleaseComparison {
    let Some(running) = parse_version(running) else {
        return ReleaseComparison::NotComparable;
    };
    let Some(candidate) = parse_version(candidate) else {
        return ReleaseComparison::NotComparable;
    };
    use std::cmp::Ordering;
    match candidate.precedence(&running) {
        Ordering::Greater => ReleaseComparison::Newer,
        Ordering::Equal => ReleaseComparison::Same,
        Ordering::Less => ReleaseComparison::Older,
    }
}

// ---------------------------------------------------------------------------
// The feed reader (bounded, entry-id-based extraction)
// ---------------------------------------------------------------------------

/// Extracts the release tags from the atom feed body: the `<id>` of each of
/// the first [`MAX_FEED_ENTRIES`] `<entry>` blocks (GitHub's entry id ends
/// with the tag, e.g. `tag:github.com,2008:Repository/1/v0.1.0-rc.14`).
/// Bounded to [`MAX_TAG_BYTES`] per tag; malformed entries are skipped,
/// never fatal.
pub(crate) fn parse_feed_tags(body: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut rest = body;
    while tags.len() < MAX_FEED_ENTRIES {
        let Some(start) = rest.find("<entry>") else {
            break;
        };
        let Some(end_offset) = rest[start..].find("</entry>") else {
            break;
        };
        let entry = &rest[start + "<entry>".len()..start + end_offset];
        rest = &rest[start + end_offset + "</entry>".len()..];
        let Some(id_start) = entry.find("<id>") else {
            continue;
        };
        let Some(id_end) = entry[id_start..].find("</id>") else {
            continue;
        };
        let id = &entry[id_start + "<id>".len()..id_start + id_end];
        let Some(tag) = id.rsplit('/').next() else {
            continue;
        };
        if tag.is_empty() || tag.len() > MAX_TAG_BYTES {
            continue;
        }
        tags.push(tag.to_owned());
    }
    tags
}

/// The newest comparable tag in the feed, or `None` when the feed carries
/// no comparable release (the named honest bound — never "up to date").
pub(crate) fn latest_release_tag(body: &str) -> Option<String> {
    parse_feed_tags(body)
        .into_iter()
        .filter_map(|tag| {
            let parsed = parse_version(&tag)?;
            Some((tag, parsed))
        })
        .max_by(|(_, left), (_, right)| left.precedence(right))
        .map(|(tag, _)| tag)
}

// ---------------------------------------------------------------------------
// The cadence gate
// ---------------------------------------------------------------------------

/// At most one automatic check per interval. A missing timestamp checks
/// immediately; a clock that jumped backwards simply waits (the honest
/// bound — never a retry storm).
pub(crate) fn update_check_due(last_check_ms: Option<i64>, now_ms: i64) -> bool {
    match last_check_ms {
        None => true,
        Some(last) => now_ms.saturating_sub(last) >= UPDATE_CHECK_INTERVAL_MS,
    }
}

// ---------------------------------------------------------------------------
// The fetch (the only network surface; failure-tolerant by design)
// ---------------------------------------------------------------------------

/// The result of one completed check attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UpdateFetchOutcome {
    Available { tag: String },
    UpToDate,
    Unreachable { reason: String },
}

fn bounded(reason: &str, limit: usize) -> String {
    let mut cut = limit.min(reason.len());
    while cut > 0 && !reason.is_char_boundary(cut) {
        cut -= 1;
    }
    if cut == reason.len() {
        reason.to_owned()
    } else {
        format!("{}…", &reason[..cut])
    }
}

/// Fetches the feed (bounded read) and classifies the outcome against the
/// running build's version constant. Network or parse failures are honest
/// `Unreachable` outcomes — never `UpToDate`.
pub(crate) fn perform_update_check(feed_url: &str) -> UpdateFetchOutcome {
    let client = match reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(FETCH_TIMEOUT)
        .user_agent(concat!("codexRS/", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return UpdateFetchOutcome::Unreachable {
                reason: bounded(
                    &format!("HTTP client unavailable: {error}"),
                    MAX_REASON_BYTES,
                ),
            };
        }
    };
    let response = match client.get(feed_url).send() {
        Ok(response) => response,
        Err(error) => {
            return UpdateFetchOutcome::Unreachable {
                reason: bounded(&format!("feed unreachable: {error}"), MAX_REASON_BYTES),
            };
        }
    };
    if !response.status().is_success() {
        return UpdateFetchOutcome::Unreachable {
            reason: bounded(
                &format!("feed returned HTTP {}", response.status()),
                MAX_REASON_BYTES,
            ),
        };
    }
    let mut body = String::new();
    if let Err(error) = response.take(MAX_FEED_BYTES).read_to_string(&mut body) {
        return UpdateFetchOutcome::Unreachable {
            reason: bounded(&format!("feed read failed: {error}"), MAX_REASON_BYTES),
        };
    }
    match latest_release_tag(&body) {
        Some(tag) => match compare_release_tags(env!("CARGO_PKG_VERSION"), &tag) {
            ReleaseComparison::Newer => UpdateFetchOutcome::Available { tag },
            ReleaseComparison::Same | ReleaseComparison::Older => UpdateFetchOutcome::UpToDate,
            ReleaseComparison::NotComparable => UpdateFetchOutcome::Unreachable {
                reason: "feed carried no comparable release tags".to_owned(),
            },
        },
        None => UpdateFetchOutcome::Unreachable {
            reason: "feed carried no release entries".to_owned(),
        },
    }
}

// ---------------------------------------------------------------------------
// The shared state + the backend runtime
// ---------------------------------------------------------------------------

/// What the status surface shows. Derived, never stored — the stored truth
/// is [`UpdateCheckState::availability`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UpdateNotice {
    /// No row content beyond the version: checks disabled, nothing checked
    /// yet, or the current notice was dismissed.
    Hidden,
    UpdateAvailable {
        tag: String,
    },
    UpToDate,
    CouldNotCheck {
        reason: String,
    },
}

/// The availability truth (never restored across restarts — a fresh check
/// re-establishes it; a stale "up to date" would be a lie).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UpdateAvailability {
    NotChecked,
    Available { tag: String },
    UpToDate { checked_at_ms: i64 },
    Unreachable { reason: String, checked_at_ms: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateCheckState {
    pub enabled: bool,
    pub availability: UpdateAvailability,
    pub last_check_ms: Option<i64>,
    pub dismissed_tag: Option<String>,
}

impl Default for UpdateCheckState {
    fn default() -> Self {
        Self {
            enabled: true,
            availability: UpdateAvailability::NotChecked,
            last_check_ms: None,
            dismissed_tag: None,
        }
    }
}

impl UpdateCheckState {
    /// The status-surface row content. The dismissed tag only hides the
    /// notice for THAT tag; a newer release re-announces itself.
    pub(crate) fn notice(&self) -> UpdateNotice {
        if !self.enabled {
            return UpdateNotice::Hidden;
        }
        match &self.availability {
            UpdateAvailability::NotChecked => UpdateNotice::Hidden,
            UpdateAvailability::Available { tag } => {
                if self.dismissed_tag.as_deref() == Some(tag.as_str()) {
                    UpdateNotice::Hidden
                } else {
                    UpdateNotice::UpdateAvailable { tag: tag.clone() }
                }
            }
            UpdateAvailability::UpToDate { .. } => UpdateNotice::UpToDate,
            UpdateAvailability::Unreachable { reason, .. } => UpdateNotice::CouldNotCheck {
                reason: reason.clone(),
            },
        }
    }
}

fn unix_timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or_default()
}

/// The backend-thread runtime: owns the in-flight fetch, persists the knob
/// and the cadence timestamp, and hands status lines to the caller (the
/// backend owns the single emission choke point).
pub(crate) struct UpdateCheckRuntime {
    state: Arc<Mutex<UpdateCheckState>>,
    pending: Option<Receiver<UpdateFetchOutcome>>,
    pending_manual: bool,
}

impl UpdateCheckRuntime {
    /// Initializes from the persisted knob + cadence timestamp. The
    /// availability always starts `NotChecked` (a fresh check tells the
    /// truth; a restored one could not).
    pub(crate) fn new(state: Arc<Mutex<UpdateCheckState>>, store: Option<&Store>) -> Self {
        if let Some(store) = store {
            let enabled = store
                .preference(UPDATES_CHECK_PREFERENCE)
                .ok()
                .flatten()
                .is_none_or(|value| value != "off");
            let last_check_ms = store
                .preference(UPDATES_LAST_CHECK_PREFERENCE)
                .ok()
                .flatten()
                .and_then(|value| value.parse::<i64>().ok());
            if let Ok(mut state) = state.lock() {
                state.enabled = enabled;
                state.last_check_ms = last_check_ms;
            }
        }
        Self {
            state,
            pending: None,
            pending_manual: false,
        }
    }

    /// Drains a completed fetch and starts the automatic check when due.
    /// Cheap on every backend tick (25 ms) when nothing is pending.
    /// Returns the status line the caller should emit, if any.
    pub(crate) fn poll(&mut self, store: Option<&mut Store>) -> Option<String> {
        if let Some(pending) = self.pending.take() {
            let manual = self.pending_manual;
            self.pending_manual = false;
            match pending.try_recv() {
                Ok(outcome) => return self.complete(store, outcome, manual),
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    self.pending = Some(pending);
                    self.pending_manual = manual;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // The fetch thread died without reporting: the honest
                    // unreachable bound, counted as an attempt (no spin).
                    return self.complete(
                        store,
                        UpdateFetchOutcome::Unreachable {
                            reason: "the check did not report a result".to_owned(),
                        },
                        manual,
                    );
                }
            }
        }
        let (enabled, last_check_ms) = {
            let Ok(state) = self.state.lock() else {
                return None;
            };
            (state.enabled, state.last_check_ms)
        };
        if enabled && update_check_due(last_check_ms, unix_timestamp_ms()) {
            self.start_fetch(false, store);
        }
        None
    }

    /// The palette path: the user's explicit request. Always reports its
    /// outcome through the status surface.
    pub(crate) fn request_manual_check(&mut self, store: Option<&mut Store>) -> Option<String> {
        if self.pending.is_some() {
            return Some("Checking for updates…".to_owned());
        }
        if !self.enabled() {
            return Some(
                "Update checks are off — enable them in Settings to check for updates".to_owned(),
            );
        }
        self.start_fetch(true, store);
        Some("Checking for updates…".to_owned())
    }

    fn enabled(&self) -> bool {
        self.state.lock().map(|state| state.enabled).unwrap_or(true)
    }

    /// Spawns the fetch thread. On a spawn failure the attempt is still
    /// counted (the cadence timestamp persists) so a broken thread spawner
    /// cannot spin — the honest unreachable bound.
    fn start_fetch(&mut self, manual: bool, store: Option<&mut Store>) {
        let (sender, receiver) = crossbeam_channel::bounded::<UpdateFetchOutcome>(1);
        let spawned = thread::Builder::new()
            .name("codex-rs-update-check".to_owned())
            .spawn(move || {
                let outcome = perform_update_check(RELEASES_FEED_URL);
                let _ = sender.send(outcome);
            });
        match spawned {
            Ok(_) => {
                self.pending = Some(receiver);
                self.pending_manual = manual;
            }
            Err(_) => {
                // Count the failed attempt now (no spin); report nothing on
                // the automatic path — silence is the failure law.
                let now_ms = unix_timestamp_ms();
                if let Some(store) = store {
                    let _ = store.set_preference(
                        UPDATES_LAST_CHECK_PREFERENCE,
                        &now_ms.to_string(),
                        now_ms / 1000,
                    );
                }
                if let Ok(mut state) = self.state.lock() {
                    state.last_check_ms = Some(now_ms);
                    if manual {
                        state.availability = UpdateAvailability::Unreachable {
                            reason: "the update check could not start".to_owned(),
                            checked_at_ms: now_ms,
                        };
                    }
                }
                self.pending_manual = false;
            }
        }
    }

    /// Applies a completed outcome: persists the cadence timestamp (every
    /// attempt counts — once per day, offline included), updates the shared
    /// state, and returns the status line to emit. `manual` selects the
    /// reporting law: the manual check always reports; the automatic check
    /// reports only a newly available update (never a nag).
    fn complete(
        &mut self,
        store: Option<&mut Store>,
        outcome: UpdateFetchOutcome,
        manual: bool,
    ) -> Option<String> {
        let now_ms = unix_timestamp_ms();
        if let Some(store) = store {
            let _ = store.set_preference(
                UPDATES_LAST_CHECK_PREFERENCE,
                &now_ms.to_string(),
                now_ms / 1000,
            );
        }
        let (notice, status) = {
            let Ok(mut state) = self.state.lock() else {
                return None;
            };
            state.last_check_ms = Some(now_ms);
            match outcome {
                UpdateFetchOutcome::Available { tag } => {
                    state.availability = UpdateAvailability::Available { tag: tag.clone() };
                    let status =
                        format!("Update {tag} is available — see the update row in the sidebar");
                    (UpdateNotice::UpdateAvailable { tag }, Some(status))
                }
                UpdateFetchOutcome::UpToDate => {
                    state.availability = UpdateAvailability::UpToDate {
                        checked_at_ms: now_ms,
                    };
                    let status = format!("codexRS {} is up to date", env!("CARGO_PKG_VERSION"));
                    (UpdateNotice::UpToDate, Some(status))
                }
                UpdateFetchOutcome::Unreachable { reason } => {
                    state.availability = UpdateAvailability::Unreachable {
                        reason: reason.clone(),
                        checked_at_ms: now_ms,
                    };
                    let status = format!("Couldn't check for updates: {reason}");
                    (UpdateNotice::CouldNotCheck { reason }, Some(status))
                }
            }
        };
        // Reporting law: the manual check always reports; the automatic
        // check reports only a newly available update (never a nag).
        match (&notice, manual) {
            (UpdateNotice::UpdateAvailable { .. }, _) => status,
            (_, true) => status,
            _ => None,
        }
        .map(|status| bounded(&status, STATUS_BYTES))
    }

    /// The config knob: "on"/"off", persisted through the settings path.
    /// "off" disables the fetch entirely (and hides any pending notice).
    pub(crate) fn set_enabled(
        &mut self,
        store: Option<&mut Store>,
        enabled: bool,
    ) -> Option<String> {
        if let Some(store) = store {
            let value = if enabled { "on" } else { "off" };
            let now_ms = unix_timestamp_ms();
            if store
                .set_preference(UPDATES_CHECK_PREFERENCE, value, now_ms / 1000)
                .is_err()
            {
                return Some("Unable to save the update-check setting".to_owned());
            }
        }
        if let Ok(mut state) = self.state.lock() {
            state.enabled = enabled;
            if !enabled {
                state.availability = UpdateAvailability::NotChecked;
            }
        }
        Some(if enabled {
            "Update checks are on".to_owned()
        } else {
            "Update checks are off".to_owned()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The comparison table: older / same / newer / pre-release / malformed.

    #[test]
    fn identical_versions_are_same_with_or_without_prefixes() {
        assert_eq!(
            compare_release_tags("0.1.0-rc.14", "v0.1.0-rc.14"),
            ReleaseComparison::Same
        );
        assert_eq!(
            compare_release_tags("1.2.3", "1.2.3"),
            ReleaseComparison::Same
        );
    }

    #[test]
    fn newer_and_older_pre_releases_compare_by_segment() {
        assert_eq!(
            compare_release_tags("0.1.0-rc.14", "v0.1.0-rc.15"),
            ReleaseComparison::Newer
        );
        assert_eq!(
            compare_release_tags("0.1.0-rc.14", "v0.1.0-rc.13"),
            ReleaseComparison::Older
        );
    }

    #[test]
    fn numeric_segments_compare_numerically_not_lexically() {
        assert_eq!(
            compare_release_tags("1.0.0-rc.2", "1.0.0-rc.11"),
            ReleaseComparison::Newer
        );
    }

    #[test]
    fn a_release_outranks_its_own_pre_release() {
        assert_eq!(
            compare_release_tags("0.1.0-rc.14", "0.1.0"),
            ReleaseComparison::Newer
        );
        assert_eq!(
            compare_release_tags("0.1.0", "0.1.0-rc.14"),
            ReleaseComparison::Older
        );
    }

    #[test]
    fn core_fields_compare_before_pre_release() {
        assert_eq!(
            compare_release_tags("0.1.0", "0.2.0-rc.1"),
            ReleaseComparison::Newer
        );
        assert_eq!(
            compare_release_tags("1.0.0", "0.9.9"),
            ReleaseComparison::Older
        );
        assert_eq!(
            compare_release_tags("0.1.0-rc.99", "0.1.1-rc.1"),
            ReleaseComparison::Newer
        );
    }

    #[test]
    fn longer_pre_release_wins_when_prefixes_match() {
        assert_eq!(
            compare_release_tags("1.0.0-alpha", "1.0.0-alpha.1"),
            ReleaseComparison::Newer
        );
        assert_eq!(
            compare_release_tags("1.0.0-alpha.1", "1.0.0-alpha"),
            ReleaseComparison::Older
        );
    }

    #[test]
    fn numeric_pre_release_segments_rank_below_alphanumeric() {
        assert_eq!(
            compare_release_tags("1.0.0-1", "1.0.0-alpha"),
            ReleaseComparison::Newer
        );
        assert_eq!(
            compare_release_tags("1.0.0-alpha", "1.0.0-beta"),
            ReleaseComparison::Newer
        );
    }

    #[test]
    fn build_metadata_is_ignored() {
        assert_eq!(
            compare_release_tags("1.0.0+build.1", "1.0.0+build.2"),
            ReleaseComparison::Same
        );
        assert_eq!(
            compare_release_tags("1.0.0-rc.1+b", "1.0.0-rc.1+a"),
            ReleaseComparison::Same
        );
    }

    #[test]
    fn malformed_tags_are_never_comparable() {
        for tag in [
            "not-a-version",
            "",
            "v",
            "1.2",
            "1.2.3.4",
            "1.0.0-",
            "1.0.0-rc.",
            "x.y.z",
        ] {
            assert_eq!(
                compare_release_tags("0.1.0-rc.14", tag),
                ReleaseComparison::NotComparable,
                "tag {tag:?} must not be comparable"
            );
        }
        // A malformed RUNNING version is equally incomparable.
        assert_eq!(
            compare_release_tags("garbage", "1.2.3"),
            ReleaseComparison::NotComparable
        );
    }

    // The feed reader.

    const FEED_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Release notes from Flauz.app</title>
  <entry>
    <id>tag:github.com,2008:Repository/1372445846/v0.1.0-rc.14</id>
    <title>Flauz.app v0.1.0-rc.14</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/1372445846/v0.1.0-rc.13</id>
    <title>Flauz.app v0.1.0-rc.13</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/1372445846/v0.1.0</id>
    <title>Flauz.app v0.1.0</title>
  </entry>
</feed>"#;

    #[test]
    fn feed_tags_extract_from_entry_ids() {
        let tags = parse_feed_tags(FEED_FIXTURE);
        assert_eq!(
            tags,
            vec![
                "v0.1.0-rc.14".to_owned(),
                "v0.1.0-rc.13".to_owned(),
                "v0.1.0".to_owned(),
            ]
        );
    }

    #[test]
    fn latest_release_tag_picks_the_max_comparable_tag() {
        // rc.14 > rc.13, but the plain release 0.1.0 outranks both.
        assert_eq!(latest_release_tag(FEED_FIXTURE).as_deref(), Some("v0.1.0"));
    }

    #[test]
    fn a_feed_of_only_pre_releases_picks_the_newest_pre_release() {
        let feed = r#"<feed><entry><id>t/1/v2.0.0-rc.1</id></entry>
        <entry><id>t/1/v2.0.0-rc.3</id></entry>
        <entry><id>t/1/v2.0.0-rc.2</id></entry></feed>"#;
        assert_eq!(latest_release_tag(feed).as_deref(), Some("v2.0.0-rc.3"));
    }

    #[test]
    fn empty_or_malformed_feeds_yield_no_release() {
        assert_eq!(latest_release_tag(""), None);
        assert_eq!(latest_release_tag("<feed></feed>"), None);
        // Entries whose ids carry no version-shaped tail are skipped.
        assert_eq!(
            latest_release_tag("<entry><id>not-a-version</id></entry>"),
            None
        );
    }

    #[test]
    fn feed_parsing_is_entry_bounded() {
        let mut feed = String::from("<feed>");
        for index in 0..(MAX_FEED_ENTRIES + 10) {
            feed.push_str(&format!(
                "<entry><id>tag:github.com,2008:R/1/v0.1.{index}</id></entry>"
            ));
        }
        feed.push_str("</feed>");
        assert_eq!(parse_feed_tags(&feed).len(), MAX_FEED_ENTRIES);
    }

    // The cadence gate.

    #[test]
    fn the_first_check_is_due_immediately() {
        assert!(update_check_due(None, 1_000));
    }

    #[test]
    fn a_check_within_the_interval_is_not_due() {
        let now = 1_000_000_000_i64;
        assert!(!update_check_due(
            Some(now - UPDATE_CHECK_INTERVAL_MS + 1),
            now
        ));
    }

    #[test]
    fn a_check_after_the_full_interval_is_due() {
        let now = 1_000_000_000_i64;
        assert!(update_check_due(Some(now - UPDATE_CHECK_INTERVAL_MS), now));
    }

    #[test]
    fn a_backward_clock_never_rechecks_early() {
        let now = 1_000_000_000_i64;
        assert!(!update_check_due(Some(now + 10_000), now));
    }

    // The offline behavior: an unreachable feed is an honest Unreachable,
    // never UpToDate.

    #[test]
    fn an_unreachable_feed_reports_could_not_check() {
        // Loopback port 9 (discard): the connection is refused immediately
        // on every platform this suite runs on — the deterministic offline
        // case.
        let outcome = perform_update_check("http://127.0.0.1:9/releases.atom");
        assert!(
            matches!(outcome, UpdateFetchOutcome::Unreachable { .. }),
            "the offline probe must be Unreachable, got {outcome:?}"
        );
    }

    // The state/notice derivation.

    #[test]
    fn disabled_checks_hide_every_notice() {
        let state = UpdateCheckState {
            enabled: false,
            availability: UpdateAvailability::Available {
                tag: "v9.9.9".to_owned(),
            },
            ..UpdateCheckState::default()
        };
        assert_eq!(state.notice(), UpdateNotice::Hidden);
    }

    #[test]
    fn an_available_update_surfaces_until_dismissed_for_that_tag() {
        let mut state = UpdateCheckState {
            availability: UpdateAvailability::Available {
                tag: "v0.2.0".to_owned(),
            },
            ..UpdateCheckState::default()
        };
        assert_eq!(
            state.notice(),
            UpdateNotice::UpdateAvailable {
                tag: "v0.2.0".to_owned()
            }
        );
        state.dismissed_tag = Some("v0.2.0".to_owned());
        assert_eq!(state.notice(), UpdateNotice::Hidden);
        // A NEWER tag re-announces itself (dismissal is per-tag).
        state.availability = UpdateAvailability::Available {
            tag: "v0.3.0".to_owned(),
        };
        assert_eq!(
            state.notice(),
            UpdateNotice::UpdateAvailable {
                tag: "v0.3.0".to_owned()
            }
        );
    }

    #[test]
    fn up_to_date_and_unreachable_are_both_truthful() {
        let mut state = UpdateCheckState {
            availability: UpdateAvailability::UpToDate { checked_at_ms: 42 },
            ..UpdateCheckState::default()
        };
        assert_eq!(state.notice(), UpdateNotice::UpToDate);
        state.availability = UpdateAvailability::Unreachable {
            reason: "feed unreachable".to_owned(),
            checked_at_ms: 42,
        };
        assert_eq!(
            state.notice(),
            UpdateNotice::CouldNotCheck {
                reason: "feed unreachable".to_owned()
            }
        );
        // NotChecked stays hidden (never claims up-to-date without a check).
        state.availability = UpdateAvailability::NotChecked;
        assert_eq!(state.notice(), UpdateNotice::Hidden);
    }

    #[test]
    fn the_runtime_restores_the_knob_and_cadence_from_storage() {
        let Ok(mut store) = Store::open_in_memory() else {
            panic!("in-memory store should open");
        };
        assert!(
            store
                .set_preference(UPDATES_CHECK_PREFERENCE, "off", 1_700_000_000)
                .is_ok()
        );
        assert!(
            store
                .set_preference(
                    UPDATES_LAST_CHECK_PREFERENCE,
                    "1234567890123",
                    1_700_000_000
                )
                .is_ok()
        );
        let state = Arc::new(Mutex::new(UpdateCheckState::default()));
        let _runtime = UpdateCheckRuntime::new(Arc::clone(&state), Some(&store));
        let Ok(snapshot) = state.lock() else {
            panic!("state lock");
        };
        assert!(!snapshot.enabled);
        assert_eq!(snapshot.last_check_ms, Some(1_234_567_890_123));
    }

    #[test]
    fn the_runtime_defaults_the_knob_to_on_and_ignores_junk_timestamps() {
        let Ok(mut store) = Store::open_in_memory() else {
            panic!("in-memory store should open");
        };
        assert!(
            store
                .set_preference(
                    UPDATES_LAST_CHECK_PREFERENCE,
                    "not-a-timestamp",
                    1_700_000_000
                )
                .is_ok()
        );
        let state = Arc::new(Mutex::new(UpdateCheckState::default()));
        let _runtime = UpdateCheckRuntime::new(Arc::clone(&state), Some(&store));
        let Ok(snapshot) = state.lock() else {
            panic!("state lock");
        };
        assert!(snapshot.enabled, "the knob defaults to on");
        assert_eq!(snapshot.last_check_ms, None, "junk timestamps are ignored");
    }

    #[test]
    fn the_runtime_tolerates_a_missing_store() {
        let state = Arc::new(Mutex::new(UpdateCheckState::default()));
        let _runtime = UpdateCheckRuntime::new(Arc::clone(&state), None);
        let Ok(snapshot) = state.lock() else {
            panic!("state lock");
        };
        assert!(snapshot.enabled);
        assert_eq!(snapshot.last_check_ms, None);
    }

    #[test]
    fn the_manual_check_reports_every_outcome_and_the_automatic_only_available() {
        let state = Arc::new(Mutex::new(UpdateCheckState::default()));
        let mut runtime = UpdateCheckRuntime::new(Arc::clone(&state), None);
        // Manual: every outcome reports.
        assert_eq!(
            runtime.complete(None, UpdateFetchOutcome::UpToDate, true),
            Some(format!(
                "codexRS {} is up to date",
                env!("CARGO_PKG_VERSION")
            ))
        );
        assert_eq!(
            runtime.complete(
                None,
                UpdateFetchOutcome::Unreachable {
                    reason: "offline".to_owned()
                },
                true
            ),
            Some("Couldn't check for updates: offline".to_owned())
        );
        assert_eq!(
            runtime.complete(
                None,
                UpdateFetchOutcome::Available {
                    tag: "v9.9.9".to_owned()
                },
                true
            ),
            Some("Update v9.9.9 is available — see the update row in the sidebar".to_owned())
        );
        // Automatic: only an available update reports.
        assert_eq!(
            runtime.complete(None, UpdateFetchOutcome::UpToDate, false),
            None
        );
        assert_eq!(
            runtime.complete(
                None,
                UpdateFetchOutcome::Unreachable {
                    reason: "offline".to_owned()
                },
                false
            ),
            None
        );
        assert_eq!(
            runtime.complete(
                None,
                UpdateFetchOutcome::Available {
                    tag: "v9.9.9".to_owned()
                },
                false
            ),
            Some("Update v9.9.9 is available — see the update row in the sidebar".to_owned())
        );
    }

    #[test]
    fn a_completed_attempt_persists_the_cadence_timestamp() {
        let Ok(mut store) = Store::open_in_memory() else {
            panic!("in-memory store should open");
        };
        let state = Arc::new(Mutex::new(UpdateCheckState::default()));
        let mut runtime = UpdateCheckRuntime::new(Arc::clone(&state), None);
        let _ = runtime.complete(Some(&mut store), UpdateFetchOutcome::UpToDate, false);
        let persisted = match store.preference(UPDATES_LAST_CHECK_PREFERENCE) {
            Ok(Some(value)) => value,
            Ok(None) => panic!("timestamp should be persisted"),
            Err(_) => panic!("preference read failed"),
        };
        assert!(persisted.parse::<i64>().is_ok(), "persisted {persisted:?}");
        // The next poll is not due (the attempt just counted).
        let Ok(snapshot) = state.lock() else {
            panic!("state lock");
        };
        assert!(!update_check_due(
            snapshot.last_check_ms,
            unix_timestamp_ms()
        ));
    }

    #[test]
    fn the_persisted_round_trip_of_the_knob_is_the_named_values() {
        let Ok(mut store) = Store::open_in_memory() else {
            panic!("in-memory store should open");
        };
        assert!(
            store
                .set_preference(UPDATES_CHECK_PREFERENCE, "off", 1)
                .is_ok()
        );
        assert_eq!(
            store.preference(UPDATES_CHECK_PREFERENCE).ok().flatten(),
            Some("off".to_owned())
        );
    }
}
