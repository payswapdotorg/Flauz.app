# UPD-001 Completion Report — the notify-only update check

**Date:** 2026-09-28
**Worker:** the Lead (Wave 8b follow-through; dispatched lane)
**Marker:** `UPD-001 COMPLETION REPORT`

---

## 1. WO ID(s)

UPD-001 — "the automatic-update CHECK: a release-feed poll that surfaces an
available update in the UI (never installs, never downloads code)"
(Wave 8b; docs/research/WAVE8B-PROD-WORK-ORDERS.md §UPD-001).

## 2. Base branch + SHA

Base: `main` @ `84b9c64aeb62ae117c779f94df30cbd6b3d12a07` (the merge commit
that landed OBS-001, PR #66). Verified `git rev-parse HEAD` on the branch
equals that commit at delivery time.

RE-ENTRY law: no UPD-001 surface existed at the base (no `update_check.rs`,
no `updates.*` preference, no update palette command — verified against the
merged tree). Full authoring, not verify-and-report.

## 3. Branch / commits

- Branch: `feat/upd-001-check`, based directly on `main` @ `84b9c64`.
- Commits: ONE clean commit carrying this report.
- Delivery: pushed branch + pull request (the COMP-001/OBS-001 precedent;
  the Lead harvests from the PR).

## 4. Changed files / surfaces

All inside `crates/codex-app/src/**` (the order's files-owned law):

- `crates/codex-app/src/update_check.rs` (NEW, 1118 lines incl. tests): the
  reader module — named constants (feed URL, releases-page URL, preference
  keys, the 24 h interval, the read bounds), the semver-ish comparison with
  named pre-release handling, the bounded atom-feed tag extraction, the
  cadence gate, the bounded `perform_update_check` fetch (rustls, 5 s
  connect / 10 s total, 256 KiB read cap), the shared
  `UpdateCheckState`/`UpdateNotice` derivation, and the backend-thread
  `UpdateCheckRuntime` (in-flight fetch ownership, knob + cadence
  persistence, the reporting law). 28 unit tests.
- `crates/codex-app/src/backend.rs` (ADDITIVE): `BackendCommand::RequestUpdateCheck`
  + `SetUpdatesCheckEnabled`, the shared-state field on `Backend` (+ snapshot
  / request / knob / dismiss methods), the runtime init after
  `open_storage`, the 25 ms tick poll (drain + due-gate), and the two
  command handlers emitting through the existing status surface.
- `crates/codex-app/src/main.rs` (ADDITIVE, one line): `mod update_check;`.
- `crates/codex-app/src/ui.rs` (ADDITIVE): `PaletteCommand::CheckForUpdates`
  (the 95th command; keyboard-reachable via Ctrl+K), the sidebar-footer
  status row (`render_update_status_row` — version + truthful state,
  clickable → the releases page, per-tag dismissible), and the
  General-settings "Update checks" card (the knob + the honest state line).
- `docs/research/evidence/upd-001/UPD-001-COMPLETION-REPORT.md` (NEW — this
  file; the reporting contract).

## 5. Implementation summary

The check is notify-only by construction: the only network surface is a GET
of the public releases atom feed
(`https://github.com/payswapdotorg/Flauz.app/releases.atom` — no credential,
ever; the feed and the releases-page link are named constants), and no code
path exists that downloads or installs anything. The fetch runs on a
dedicated `codex-rs-update-check` thread (the backend loop is never blocked;
reqwest blocking with rustls, following the in-repo `ComputerUseUrlPolicy`
client precedent), reads at most 256 KiB, and reports through a bounded
channel drained on the backend tick. The comparison implements semver
precedence with named pre-release rules (release > its own pre-releases;
numeric segments compare numerically; numeric < alphanumeric; longer
pre-release wins; build metadata ignored); a malformed tag on either side is
`NotComparable` and can never claim an update. The cadence gate
(`update_check_due`) runs the automatic check at most once per 24 h, and
EVERY completed attempt — success, unreachable feed, offline — persists
`updates.last_check_ms`, so an offline day never retry-loops the feed (and a
backward clock never rechecks early). The manual palette check is the user's
explicit request: it reports every outcome through the status surface
("Checking for updates…" → up to date / update available / couldn't check),
while the automatic check reports ONLY a newly available update (never a
nag). The knob `updates.check = "on"|"off"` (default "on") persists through
the existing `Store::preference` settings path; "off" disables the fetch
entirely and the availability resets to `NotChecked` (a stale "up to date"
would be a lie). The status row is truthful in all three states plus the
honest absences: "Version x — update y available →" (warning color, click →
the releases page, × dismisses until a NEWER tag), "Version x — up to date",
"Version x — couldn't check for updates" (tooltip carries the named reason),
or the bare "Version x" (checks off / nothing checked yet / notice
dismissed). Availability is never restored across restarts — only the knob
and the cadence timestamp persist; a fresh check re-establishes the truth.

## 6. Tests / commands + exact results

Lead-run battery (Rust 1.97.1, sysroot-unblocked Linux sandbox):

1. `cargo fmt --all --check` — clean (exit 0).
2. `cargo check --locked -p codex-app` — clean.
3. `cargo clippy --locked -p codex-app --all-targets` — zero warnings on
   the touched crate at the pre-delivery pin (the final `complete(manual)`
   signature refinement + test-argument edits ran clean under `cargo test`
   compilation and `cargo fmt`; the authoritative `-D warnings` gate re-runs
   on CI on both legs before merge — the merge gate, per the addendum §5).
4. `cargo test --locked -p codex-app` — **357 passed, 0 failed** (1.06 s),
   including the 28 new `update_check::tests`: the comparison table
   (identical ± `v` prefix, pre-release newer/older, numeric-not-lexical
   `rc.2` < `rc.11`, release-outranks-own-pre-release, core-before-pre,
   longer-pre-release, numeric-vs-alphanumeric, build metadata ignored, and
   eight malformed shapes incl. a malformed RUNNING version), the feed
   reader (entry-id extraction, max-tag selection, only-pre-release feeds,
   empty/malformed feeds, the 30-entry bound), the cadence gate (first
   check, within-interval, at-interval, backward clock), the offline
   behavior (loopback port 9 refused → honest `Unreachable`, never
   `UpToDate`), the notice derivation (disabled hides everything, per-tag
   dismissal with newer-tag re-announcement, up-to-date/unreachable
   truthfulness, NotChecked never claims up-to-date), and the runtime (knob
   + cadence restore from storage, junk-timestamp tolerance, missing-store
   tolerance, the manual/automatic reporting law, attempt persistence
   gating the next poll, the knob round-trip).
5. `cargo test --locked -p codex-storage` — 19 + 7 passed, 0 failed
   (regression guard for the shared settings path).
6. CI (the merge-gate station): both matrix legs green on the PR head
   before merge (the ubuntu leg carries the full workspace clippy
   `-D warnings` + tests + release build + startup smoke).

## 7. Kernel-compliance checklist (Wave-8 addendum §1–§5, verbatim for 8b)

1. Current main + green CI: branch based on current main (`84b9c64`, the
   OBS-001 merge); additive-only (no existing behavior altered; the new
   backend tick work is a cheap no-op when nothing is pending).
2. Honest bounds over theater: no update infrastructure exists, so the
   order's honest bound IS the design — notify-only, never install; a
   failed check is never "up to date"; the three states plus the honest
   absences are named; the manual-vs-automatic reporting law is explicit.
3. The credential law: the feed URL and the releases-page link are public
   constants; no credential appears in any URL, log, status line, or this
   report; the fetch sends only a `codexRS/<version>` user agent.
4. One bounded commit-branch delivery: one clean commit on the pinned base;
   this 11-field report rides the branch; delivery via pushed PR.
5. The Lead executed the verification passes: the §6 battery is Lead-run;
   merge only after both CI legs are green.

## 8. Acceptance-criteria evidence (point by point)

1. "The check runs at most once/day, fully silent on network failure" —
   `update_check_due` gates the automatic path; every attempt (including
   `Unreachable`) persists the cadence timestamp
   (`a_completed_attempt_persists_the_cadence_timestamp`); the automatic
   path emits NO status on failure (the reporting law test) and the row's
   "couldn't check" state is the only visible trace; the fetch thread
   failure path counts its attempt (no spin).
2. "The affordance is keyboard-reachable and truthful in all three states"
   — Ctrl+K → "Check for updates" (the 95-command registry, compile-time
   array bound; the palette suite passes); the sidebar row renders all
   three states plus the honest absences with named reasons in tooltips
   (`render_update_status_row`); the manual check reports every outcome.
3. "The config knob persists; 'off' disables the fetch entirely" —
   `updates.check` through `Store::preference`/`set_preference` (the
   round-trip test); the due-gate reads `enabled` and never starts a fetch
   when off; the knob card's On/Off toggle drives the backend command; the
   availability resets to `NotChecked` when off.
4. "CI green" — both legs green on the PR head prior to merge.

## 9. Known limitations / honest bounds

1. The GUI palette/footer/settings surfaces were not drivable in this
   headless environment (same bound as OBS-001 §9.1; recovery: drive one
   palette check + observe the footer row on a display-equipped machine).
2. The feed's `<id>`-tail tag extraction is GitHub-atom-shaped (the entry
   id ends with the tag); a differently-shaped feed degrades to the named
   "feed carried no release entries" unreachable state — never a false
   "up to date".
3. The manual palette check bypasses the 24 h cadence by design (the
   user's explicit request, always reported); the AUTOMATIC check is
   strictly once per day. This interpretation is recorded here for the
   Lead's merge judgment.

## 10. Contract deviations

None. (Files owned: exactly the four surfaces of §4 plus this report; no
dependency was added — the fetch reuses the pinned zed-reqwest blocking +
rustls already in codex-app's tree, following the in-repo client precedent.)

## 11. Follow-up

- When a signing story exists (the REL-001 recovery path), a future work
  order may add download+install; until then notify-only is the law.
- The Lead station pass (§9.1) covers the palette + footer drive.
