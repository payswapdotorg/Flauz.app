# Wave 8b Work Orders — Production Hardening, part two

> **Status: PINNED 2026-09-26 — opens after the Wave-8 trio (REL-001 merged
> PR #62, SEC-001 merged PR #63, OBS-001 in flight).**
>
> Shared-contract authority: F2-CONTRACT-KERNEL (frozen) + the Wave-2..8
> addenda + **the Wave-8 kernel addendum (WAVE8-PROD-WORK-ORDERS.md — §1–§5
> apply verbatim to this wave)**. Every work order follows
> WORK-ORDER-TEMPLATE.md. Non-empty `Contract deviations` blocks closure.

## UPD-001 — The auto-update check (notify-don't-install)

```
ID: UPD-001
Title: the automatic-update CHECK: a release-feed poll that surfaces an
       available update in the UI (never installs, never downloads code)
Phase: Production (Wave 8b)
Owner: dispatched worker
Dependencies: the Wave-8 trio; REL-001 merged (the release artifacts carry
             SHA256SUMS — the feed's integrity story)
Contract(s): as the Wave-8 addendum + this note: the update story is
             NOTIFY-ONLY (the honest bound: no signing key exists, so
             auto-INSTALL would be unverifiable and is out of scope by law)
Problem: a production user on an old build has no in-app signal that a newer
         release exists
User-visible outcome: a subtle, dismissible "update available" affordance
         (release tag + link to the releases page) when a newer tagged
         release exists; NEVER a nag, NEVER an automatic download
Scope:
  - a release-feed reader (NEW, in the app's support path): fetch the
         repo's releases atom feed (or the GitHub API releases endpoint)
         on a conservative cadence (e.g. every 24h, on app start at most
         once per day, offline-tolerant, fully silent on failure)
  - the version comparison: semver-ish tag comparison against the running
         build's version constant; NAMED handling for pre-release tags
  - the UI affordance: a palette entry + the status surface row
         ("Version x.y.z — update n available → releases page"), keyboard
         reachable, honest when the feed is unreachable ("couldn't check
         for updates" — never "up to date" as a lie)
  - the config knob: updates.check = "on"|"off" (default "on"), persisted
         through the existing settings path
  - tests: the comparison table (older/same/newer/pre-release/malformed
         tags), the offline behavior, the cadence gate (once per day)
Non-goals: downloading/installing updates (notify-only by law); Sparkle
         or any update framework; the web client; macOS
Files/subsystems owned: crates/codex-app/src/** (additive: the reader
         module + the wiring + the tests); NOTHING else
Inputs: main at the dispatch pin; the REL-001 strategy doc (the channel
         story); the palette precedents
Outputs/artifacts: one clean commit branch feat/upd-001-check, git
         bundle, completion report (11-field; marker `UPD-001 COMPLETION
         REPORT`)
Tests: cargo test (the comparison + cadence + offline suites); the Lead
         spot-runs the affordance at the station with a fake feed
GUI/lab evidence: the Lead drives the palette path at the station
UX journey IDs: J-13 adjacent (the honest-state law: reachable/unreachable
         both truthful)
Acceptance criteria:
  1. The check runs at most once/day, fully silent on network failure
  2. The affordance is keyboard-reachable and truthful in all three
     states (update available / up to date / couldn't check)
  3. The config knob persists; "off" disables the fetch entirely
  4. CI green
Rollback/recovery: revert the merge; additive feature
Integration notes: no credential ever in the feed URL (public releases
         feed only); the release-link URL is a named constant
Status: DISPATCH-READY (Lead pins at dispatch)
```

## MIG-002 — The migration guarantee: tests + the story

```
ID: MIG-002
Title: the migration guarantee made testable: forward-migration tests for
       every schema version + the user-facing migration story doc
Phase: Production (Wave 8b)
Owner: dispatched worker
Dependencies: the Wave-8 trio
Contract(s): as the Wave-8 addendum
Problem: codex-storage carries PRAGMA user_version migrations (v1→v3) but
         the guarantee "an old state.sqlite3 opens on a new build" is
         enforced only by convention; the user-facing story is undocumented
User-visible outcome: none directly (the guarantee gates production); the
         doc tells users what happens to their data across upgrades
Scope:
  - migration tests (NEW, in codex-storage's test suite): for EVERY schema
         version v < current: build a fixture DB at v, run the migrator,
         assert the resulting schema + the preserved rows; the v0 (empty/
         fresh) case; the future-version case (a DB NEWER than the build:
         the named honest refusal, never a silent downgrade)
  - the fixture builder: hand-authored SQL per version (v1, v2, v3) with
         representative rows per table (small, credential-free)
  - docs/platform-support.md (EXTEND, additive section): the upgrade story
         — what migrates, what never does, the backup recommendation, the
         downgrade refusal
  - NO changes to the migrator itself (this order GUARANTEES existing
         behavior; if a test finds a real bug, it becomes a focused fix
         work order per the synchronization law — never a silent fix here)
Non-goals: new migrations; schema changes; the web client's storage
Files/subsystems owned: crates/codex-storage/tests/** (the new migration
         tests + fixtures), docs/platform-support.md (additive section);
         NOTHING else
Inputs: main at the dispatch pin; crates/codex-storage/src/lib.rs migrate()
         (PRAGMA user_version = 3)
Outputs/artifacts: one clean commit branch feat/mig-002-guarantee, git
         bundle, completion report (marker `MIG-002 COMPLETION REPORT`)
Tests: cargo test -p codex-storage (the new suites must pass; a REAL
         migration failure blocks this order and becomes a fix WO)
GUI/lab evidence: n/a
UX journey IDs: n/a
Acceptance criteria:
  1. Every version transition covered by a test with preserved-row
     assertions
  2. The future-version refusal tested (the named error)
  3. The story doc matches the tests exactly
  4. CI green
Rollback/recovery: revert the merge; tests + docs only
Integration notes: fixtures are hand-authored SQL (no production data,
         no credentials — the E2B law)
Status: DISPATCH-READY (Lead pins at dispatch)
```

## COMP-001 — The protocol compatibility story

```
ID: COMP-001
Title: the protocol compatibility guarantee, documented and evidenced:
       the wire-shape freeze test, the version-field semantics, and the
       compat policy doc
Phase: Production (Wave 8b)
Owner: dispatched worker
Dependencies: the Wave-8 trio
Contract(s): as the Wave-8 addendum + the F2 kernel's contract-freeze laws
Problem: the app-server protocol carries version fields (ClientInfo.version
         in the initialize shape) but there is no written compatibility
         policy: what may change, what never may, and how a mismatch is
         surfaced
User-visible outcome: none directly (the policy gates production); the
         doc is the contract for every future protocol change
Scope:
  - docs/PROTOCOL-COMPATIBILITY.md (NEW): the policy — the frozen wire
         shapes (the generated schema as the law), the additive-only
         evolution rule (new optional fields OK; renames/removals NEVER),
         the version-field semantics (what clientInfo.version promises —
         informational, not a gate), the mismatch behavior (the observed
         truth: the runtime tolerates newer clients per the codex-core
         semantics), and the test that enforces the freeze
  - the freeze test (EXTEND the existing schema test in
         codex-protocol): the generated schema artifact must match the
         committed one byte-for-byte (already exists — verify + document
         it as THE gate; extend if it lacks coverage of the initialize
         shape)
  - a compatibility-matrix section: which client versions spoke which
         protocol shapes (git-history-derived, honest)
Non-goals: protocol changes; new fields; runtime behavior changes
Files/subsystems owned: docs/PROTOCOL-COMPATIBILITY.md (NEW),
         crates/codex-protocol/src/lib.rs (test EXTENSION only); NOTHING
         else
Inputs: main at the dispatch pin; the existing wire-shape tests
         (lib.rs:3710 initialize_wire_shape_matches_generated_schema);
         the schema artifact + the export script
Outputs/artifacts: one clean commit branch feat/comp-001-story, git
         bundle, completion report (marker `COMP-001 COMPLETION REPORT`)
Tests: cargo test -p codex-protocol (the freeze test green)
GUI/lab evidence: n/a
UX journey IDs: n/a
Acceptance criteria:
  1. The policy names every frozen surface with its enforcing test
  2. The version-field semantics documented as observed (with code
     pointers)
  3. The freeze test verified/extended and green
  4. CI green
Rollback/recovery: revert the merge; docs + test only
Status: DISPATCH-READY (Lead pins at dispatch)
```

## WEB-REL — The web release channel (W-AUTH dependent)

```
ID: WEB-REL
Title: the web client's release story: the deployable artifact contract +
       the channel doc (BLOCKED on W-AUTH for the authenticated FV scenes)
Phase: Production (Wave 8b, last)
Owner: dispatched worker (after W-AUTH clears) or the Lead
Dependencies: W-AUTH (the operator's credentials) — the 13 blocked FV
             scenes must be green before this closes
Contract(s): as the Wave-8 addendum
Scope (provisional — pinned when W-AUTH clears): the web/dist build
       reproducibility (the build is deterministic; document it), the
       gateway version-matching story, the channel doc (self-host via the
       release artifacts; the operator-owned hosted channel as the named
       bound), and the FV web-lane completion re-run
Status: BLOCKED on W-AUTH (do not dispatch)
```

## Dispatch notes (Lead-only)

- Order: UPD-001 and MIG-002 and COMP-001 are pairwise-disjoint — dispatch
  all three in parallel when platform health returns (3-slot cap).
- The packets follow the W8 wrapper (build_wave8_prompts.py pattern).
- Wave 8 closes when: the trio + 8b merged AND the FV web lane's W-AUTH
  recovered AND the production checklist (F13) re-audited by the Lead.
