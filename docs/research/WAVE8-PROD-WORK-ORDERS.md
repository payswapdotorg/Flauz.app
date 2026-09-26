# Wave 8 Work Orders — Production Hardening (the F13 release-readiness wave)

> **Status: PINNED 2026-09-26 — opens after Wave 7 (FV) closed at `af2210d`.**
>
> Shared-contract authority:
> [F2-CONTRACT-KERNEL.md](../F2-CONTRACT-KERNEL.md) (frozen) + the Wave-2..7
> addenda (all frozen) + **the Wave-8 kernel addendum below**. Every work order
> follows [WORK-ORDER-TEMPLATE.md](../WORK-ORDER-TEMPLATE.md). Non-empty
> `Contract deviations` in a worker report blocks closure.
>
> **The phase this wave serves** (ROADMAP F13 + ACTIVE-EXECUTION-STATE
> immediate order item 7): the ten production items — signed releases, Linux
> distribution, web release channel, automatic updates, migrations,
> observability, recovery, security review, provider outage behavior, protocol
> compatibility. The 2026-09-23 sequencing amendment governs: macOS/Mobile
> stay deferred; the web channel is in scope only as the web client's release
> story (deployment surfaces remain operator-owned).

## Wave-8 kernel addendum (frozen by the Tech Lead before dispatch)

1. **Production hardening never weakens the verified state.** Every change
   rides current main; the FV-002 harness scenes and the CI gates must stay
   green (additive-only workflow edits; any product-source change that could
   alter a verified surface requires the synchronization law: focused PR →
   merge → fresh environment → rerun the affected journey).
2. **Honest bounds over theater.** Where the environment lacks a production
   primitive (code-signing certificates, an update infrastructure, a deployed
   update feed), the work order delivers the SCAFFOLD plus a NAMED gap with
   its recovery path — never a fake signature, never a stub that claims
   success, never an undocumented narrowing. The FV wave's W-AUTH record is
   the pattern.
3. **The credential law is absolute** (E2B harness law + the kernel): no
   credential ever appears in scripts, logs, URLs, evidence, or the
   diagnostics output. Anything that exports state must scrub by allowlist,
   not by blocklist guess.
4. **One bounded commit-branch delivery per worker** (the Wave-5/6/7
   contract): pinned full SHA, STEP ZERO clone+checkout+branch, RE-ENTRY law,
   one clean commit + git bundle, the 11-field completion report, the
   honest-absence rules.
5. **The Lead executes the verification passes** at the merge-gate station
   (CI, the calibration spot-runs, the security re-audit greps) — worker
   reports gate, they never close.

## REL-001 — Signed release artifacts + the Linux distribution story

```
ID: REL-001
Title: the signed-release scaffold: SHA256SUMS manifest + the GPG-ready
       signing step + the archive verification mode + the Linux
       distribution strategy doc
Phase: Production (Wave 8, item 1 of the trio)
Owner: dispatched worker (Wave-8 Worker A)
Dependencies: Wave 7 CLOSED (main past af2210d); release.yml green
Contract(s): F2-CONTRACT-KERNEL + wave addenda + this file's addendum
Problem: the release pipeline packages archives but produces no checksum
         manifest, no signing step, and no verification path; the Linux
         distribution strategy exists only implicitly (tar.gz + smokes)
User-visible outcome: a user can verify a downloaded release archive
         against a published checksum manifest (and, once the operator
         provides a signing key, a detached signature)
Scope:
  - release.yml (ADDITIVE steps only): a "Checksums + signing" step after
    packaging — generate SHA256SUMS per artifact + a detached-signature
    step that is GPG-ready but conditionally skipped with a NAMED log line
    when the signing key secret is absent (the honest bound: no key in this
    environment; the recovery path = the operator provisioning the secret)
  - scripts/verify_release_archive.sh (NEW): verifies a local archive
    against a SHA256SUMS file (+ the detached signature when present);
    hard-fails with named messages
  - docs/platform-support.md (EXTEND, additive section): the Linux
    distribution strategy — the tar.gz + checksums + signature story as the
    supported channel, the AppImage/deb evaluation with the honest
    environment bound (no packaging toolchain in CI today), the recovery
    path for each
  - THIRD-PARTY/LICENSE/README surfaces: NO changes (the packaging lists
    stay as-is)
Non-goals: actual code signing (no key exists — the named gap); AppImage/deb
         builds; Windows Authenticode; the web channel; auto-updates
Files/subsystems owned: .github/workflows/release.yml (additive steps only),
         scripts/verify_release_archive.sh, docs/platform-support.md
         (additive section); NOTHING else
Inputs: main at the dispatch pin; .github/workflows/release.yml (the
         packaging steps); scripts/linux_desktop_smoke.sh (the verify
         precedent); the FV gate record's honest-bounds pattern
Outputs/artifacts: one clean commit branch feat/rel-001-signing, git
         bundle, completion report (11-field contract)
Tests: bash -n on the new script; a local self-test (checksum a fixture,
         tamper it, verify the failure); the Lead re-runs the workflow YAML
         parse + the strategy-doc gates
GUI/lab evidence: n/a (pipeline + docs) — the Lead gates by workflow-parse
         + script self-test + the honest-bounds review
UX journey IDs: n/a (no product-surface change)
Acceptance criteria:
  1. SHA256SUMS generated for every packaged artifact in the workflow
  2. The signing step skips with a NAMED log line when the key is absent
     (never fails the build, never fakes a signature)
  3. verify_release_archive.sh verifies + hard-fails on tamper (self-test
     evidence in the report)
  4. The strategy doc names every channel with its status + recovery path
Rollback/recovery: revert the merge; pipeline/docs only
Integration notes: the checksum step must run AFTER both package steps and
         BEFORE the upload; keep the matrix labels intact
Status: DISPATCHED 2026-09-26 (base pinned at dispatch)
```

## OBS-001 — Observability: the diagnostics export + provider-outage record

```
ID: OBS-001
Title: the diagnostics export command (credential-scrubbed, allowlist-based)
       + the provider-outage behavior record
Phase: Production (Wave 8, item 2 of the trio)
Owner: dispatched worker (Wave-8 Worker B)
Dependencies: Wave 7 CLOSED; the four-state connection machine + retry
         cadence merged (Wave 2/3)
Contract(s): as REL-001 + the E2B harness law (credential scrubbing by
         allowlist)
Problem: a production user cannot export a support-ready diagnostics bundle
         (app log tail, state summary, connection history, environment
         facts), and the provider-outage behaviors live only in code
User-visible outcome: a palette command + CLI flag that writes a
         diagnostics bundle to a user-chosen path, with a visible
         confirmation and a truthful "what's included / what's never
         included" statement
Scope:
  - the diagnostics collector (NEW, in the app's support path): app version
         + pinned-CLI version, OS/toolchain facts, the connection-state
         history (already tracked), the log TAIL (bounded, e.g. last 200
         lines), the state.sqlite3 SCHEMA VERSION + counts (never contents)
  - credential scrubbing by ALLOWLIST: the collector emits ONLY the named
     fields; a test proves a seeded credential-like string in the log tail
     is NOT exported unless explicitly allowlisted (the E2B law)
  - the palette entry + the CLI flag (--diagnostics-out <path>) wired the
     d-series keyboard path (Ctrl+K → "Export diagnostics")
  - docs/platform-support.md or SUPPORT.md (EXTEND, additive): the
     provider-outage behavior record — what the app does when the runtime/
     provider is down (the four-state machine, the retry cadence, the
     graceful degradation, the draft-loss protection), each with its
     verified evidence pointer (the w2..w7 gate records)
Non-goals: telemetry/remote reporting (never — the privacy law); new
         product behaviors beyond the export command; log-file rotation;
         the web client (the desktop app only this order)
Files/subsystems owned: crates/codex-app/src/** (the collector + wiring,
         additive), crates/codex-storage/src/lib.rs (a read-only schema-
         version accessor if not already exposed), docs/SUPPORT.md
         (additive section); NOTHING else
Inputs: main at the dispatch pin; the ui.rs palette precedents; the
         four-state machine + retry cadence (crates/codex-app/src/backend.rs);
         the FV-CATALOG scenes for the keyboard wiring pattern
Outputs/artifacts: one clean commit branch feat/obs-001-diagnostics, git
         bundle, completion report
Tests: cargo test (the scrubbing test + the collector unit tests); bash -n
         n/a; the Lead spot-runs the export at the station + greps the
         output for credential patterns
GUI/lab evidence: the Lead runs one palette-driven export at the station
         (frame + the exported bundle's head) as the gate evidence
UX journey IDs: J-13 (recover/continue) adjacent; the palette
         discoverability law applies
Acceptance criteria:
  1. The export writes every named field and NOTHING else (the allowlist
     test proves a seeded secret stays out)
  2. The palette entry + the CLI flag both work (the keyboard path)
  3. The outage record documents every state with evidence pointers
  4. CI green (the workspace tests include the new ones)
Rollback/recovery: revert the merge; additive feature, no state changes
Integration notes: keep the collector in its own module for review clarity;
         the log-tail reader must handle the absent-log case (named honest
         bound, not an error)
Status: DISPATCHED 2026-09-26 (base pinned at dispatch)
```

## SEC-001 — The security review: the credential-law audit + the gateway review

```
ID: SEC-001
Title: the production security review: a grep-evidenced credential-law
       audit across every crate + the gateway's security surface review +
       SECURITY.md + the threat model
Phase: Production (Wave 8, item 3 of the trio)
Owner: dispatched worker (Wave-8 Worker C)
Dependencies: Wave 7 CLOSED; the gateway merged (WEB-001)
Contract(s): as REL-001 + the E2B harness law + the WEB-001 kernel laws
             (transparent transport, localhost-only default, the session
             token law)
Problem: the production gate needs an evidenced security review; the
         credential law and the gateway's security posture are enforced by
         convention but not by an auditable record
User-visible outcome: none directly (the review gates production); the
         artifacts are SECURITY.md (the honest posture) + the threat model
Scope:
  - the credential-law AUDIT (evidenced): a grep/ripgrep sweep across the
         repo for credential patterns in logs/URLs/storage/tests (Bearer,
         token=, api[_-]?key, sk-, auth.json references, hard-coded
         secrets) — every hit triaged: legitimate-named-constant / test
         fixture / violation; violations become focused fix work orders
         (NOT fixed silently in this review)
  - the gateway review: the localhost-only bind law, the non-loopback
         session-token requirement, the origin/WSS handling, the request
         timeout + inflight caps (DoS posture), the static-file serving
         path traversal — each with the code pointer + the verdict
  - SECURITY.md (REWRITE from the current stub if needed): the reporting
         policy, the supported versions, the security posture summary
  - docs/research/SECURITY-REVIEW-2026-09.md (NEW): the threat model
         (the assets: credentials, session state, the user's repo files;
         the actors; the surfaces: the app, the gateway, the supervised
         runtime, the packaged archives) + the audit table + the findings
         register with severity + the recovery paths
  - NO code changes: findings become fix work orders through the
         synchronization law (this review VERIFIES, it does not fix —
         the FV-wave pattern)
Non-goals: fixing violations (separate WOs); penetration testing beyond
         the static review; the web client's auth (W-AUTH operator item);
         CI changes
Files/subsystems owned: docs/research/SECURITY-REVIEW-2026-09.md (NEW),
         SECURITY.md; NOTHING else (the audit evidence lives INSIDE the
         review doc)
Inputs: main at the dispatch pin; every crate's source; the F2 kernel's
         credential laws; the WEB-001 gateway docs; the E2B parity doc
Outputs/artifacts: one clean commit branch feat/sec-001-review, git
         bundle, completion report
Tests: n/a (docs) — the Lead gates by re-running the audit greps (spot-
         check ≥10 of the documented hits) + the threat-model review
GUI/lab evidence: n/a
UX journey IDs: n/a
Acceptance criteria:
  1. Every grep pattern documented with its command + hit count + the
     per-hit triage
  2. The gateway review covers every named surface with code pointers
  3. SECURITY.md states the honest posture (what's reviewed, what's
     operator-owned, what's deferred)
  4. Zero source changes (git diff proves it)
Rollback/recovery: revert the merge; docs-only
Integration notes: violations found MUST be recorded as findings with
         severity, never silently patched in the same PR
Status: DISPATCHED 2026-09-26 (base pinned at dispatch)
```

## Dispatch notes (Lead-only)

- Three concurrent workers (the operator's 3-slot cap): REL-001 (pipeline/
  docs), OBS-001 (Rust feature), SEC-001 (audit docs) — pairwise disjoint
  owned paths, no shared files, merge-order independent.
- The prompt packets follow the proven Wave-5/6/7 wrapper (single
  self-contained message; STEP ZERO up top; the one-commit + bundle
  contract; the 11-field report; FENCE markers; the completion-gate marker
  per work order).
- The completion gate markers: `REL-001 COMPLETION REPORT`,
  `OBS-001 COMPLETION REPORT`, `SEC-001 COMPLETION REPORT` (the FV gate
  discipline: marker ≥2 + a real-hex Base-SHA line).
- Wave 8b (after this trio): UPD-001 (the auto-update check, honest
  notify-don't-install bound), MIG-002 (the migration guarantee tests +
  doc), COMP-001 (the protocol compatibility story), WEB-REL (the web
  release channel, W-AUTH-dependent).
- Base pin at dispatch: af2210d2dab689f12914ecd7c28fa6e955345382.
