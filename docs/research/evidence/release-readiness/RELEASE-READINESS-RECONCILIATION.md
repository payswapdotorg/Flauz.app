# Release-Readiness Reconciliation — Wave 8 / 8b close-out

**Date:** 2026-09-28
**Author:** TL #1 (the Lead)
**Main at authoring:** `8012ed9dfeed9e7eb298ebcbf8819da79c1f0287`
**Role:** the rule-10 reconciliation — every actionable work order is merged
and gated; no new wave is invented. This record states what is verified on
current main, what is stale, and what is operator-owned before production
release.

## 1. What landed (the Wave-8/8b set, all merged)

| Order | PR | Merge | Content |
| --- | --- | --- | --- |
| REL-001 | #62 | `354449c` | signed-release scaffold: SHA256SUMS coverage gate + GPG-ready signing step + verify script + Linux distribution strategy |
| SEC-001 | #63 | `7b434ca` | production security review (credential-law audit + gateway review) |
| MIG-002 | #64 | `fb5c862` | migration guarantee tests (every schema version, preserved rows, future-version refusal) + upgrade story |
| COMP-001 | #65 | `5cbfbd8` | protocol compatibility law (`docs/PROTOCOL-COMPATIBILITY.md`) + the initialize-with-capabilities freeze test |
| OBS-001 | #66 | `84b9c64` | credential-scrubbed allowlist diagnostics export (palette + `--diagnostics-out`) + the provider-outage record |
| UPD-001 | #67 | `8012ed9` | the notify-only update check (bounded feed reader, 24 h cadence, persisted knob, truthful status row) |

Every merge was gated on both CI legs (ubuntu-24.04 + windows-latest) green
at the merge head. WEB-REL is NOT landed: it remains blocked on W-AUTH by
law (§4 below).

## 2. What is verified on current main (`8012ed9`)

- The full CI battery — dependency policy, `cargo fmt --check`, the patched
  markdown renderer gate, `cargo clippy --locked --workspace --all-targets
  -D warnings`, `cargo test --locked --workspace`, the release build, the
  Linux desktop startup smoke, and the Windows FV journey smoke — ran green
  on the head of every merge above (the check-run evidence lives on each PR
  head SHA: `9553de6`, `48bf288`, `1cd4684`).
- The workspace suite at the UPD-001 head: 357 codex-app tests (incl. the
  10 OBS-001 diagnostics tests and the 28 UPD-001 update-check tests),
  26 codex-storage tests, and the full remaining workspace set — all green
  in CI on both legs.
- Migration guarantee: MIG-002's forward-migration tests run in the
  workspace battery on every merge (green at `8012ed9`).
- Release artifacts: the release pipeline (release.yml) is the REL-001
  scaffold — SHA256SUMS coverage enforced, detached signing SKIPPED with a
  named log line until the operator provisions
  `FLAUZ_RELEASE_SIGNING_KEY`. Tags `v0.1.0-rc.13`/`v0.1.0-rc.14` exist
  from that pipeline.

## 3. What is stale (the post-FV reconciliation debt)

The formal-verification gate record (`4ed0a0c`, docs/research/evidence/
fv-gate/) predates the Wave-8/8b product-source changes. Affected evidence
and its current state:

- FV Windows journey battery (FV-002 harness): rerun by CI on every merge
  head, green at `8012ed9` (the CI step "Windows FV journey smoke" IS the
  harness). The formal gate RECORD, however, still cites the pre-Wave-8
  pin; a Lead pass should refresh the record's pin table to `8012ed9` when
  closing Wave 8 for audit purposes.
- Linux desktop startup smoke: same — rerun green at every merge head
  including `8012ed9`.
- Additive-surface audit (Lead judgment): OBS-001/UPD-001 added palette
  commands (93→95), a footer status row, a settings card, two backend
  commands, and read-only storage accessors. No FV scene asserts the
  palette command COUNT (the registry test is in-app and updated with the
  changes; all green); no scene asserts the footer row set. No verified
  surface was altered (additive-only, per the addendum §1).

## 4. Operator-owned gaps (named, with recovery paths — never fabricated)

1. **W-AUTH** — the authenticated-environment credentials for the 13 web
   FV scenes (E01–E13). Blocks: WEB-REL, the web-lane FV completion, and
   therefore the full production sequence. Recovery: the operator
   provisions the environment; then the FV web lane reruns.
2. **`FLAUZ_RELEASE_SIGNING_KEY`** — no signing key exists in this
   environment; release artifacts carry SHA256SUMS (corruption detection)
   but no publisher signature (authenticity). Recovery: the operator
   provisions the secret; the REL-001 step picks it up with no further
   code change.
3. **GUI station passes** — the OBS-001 palette-export drive and the
   UPD-001 palette/footer drive were not drivable in the headless worker
   environment (named in both completion reports, §9.1). Recovery: one
   display-equipped pass over a release artifact; CI smokes + the unit
   suites cover the code paths meanwhile.

## 5. Release-readiness verdict

NOT READY — by exactly the named operator-owned gaps above. Ready the
moment they clear:

1. W-AUTH recovery → FV web-lane completion (E01–E13).
2. The FV gate record refresh to current main (Lead pass, §3).
3. The operator provisions the signing key (or formally accepts the
   unsigned-preview bound for the next release).
4. The production release audit (F13 checklist) against the final main.

## 6. TL #1 lane status

Per `docs/AUTONOMOUS-EXECUTION.md` (the final-stop law) and the handoff:
continuous autonomous execution STOPS here — every dispatchable work order
is merged and gated; the remaining items are operator-owned or gated on
them. TL #1 moves to the maintenance/release-review role: respond to
operator-owned gap recoveries, review future work orders, and own the
release audit when its preconditions clear. The repository (this record,
ACTIVE-EXECUTION-STATE.md, the completion reports, and the gate records)
is the continuation surface for any future lead or worker.
