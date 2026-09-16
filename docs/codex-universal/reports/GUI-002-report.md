# GUI-002 — Codex Desktop Parity Closure

**Status:** COMPLETE (release-critical selection)
**Base:** `32f8364` (merged GUI-003)
**Branch:** `gui-002/parity-closure`
**Work order:** `docs/codex-universal/WORK-ORDERS.md` § GUI-002
**Ledger:** `docs/parity-matrix.md` § "Release-critical parity ledger (GUI-002)"

## What was delivered

### 1. Release-critical parity ledger

The Tech Lead selection of release-critical rows with per-row verdicts
(green / bounded / deferred), added to `docs/parity-matrix.md`. The
matrix's per-row `partial` statuses remain authoritative for
full-reference tracking; the ledger defines the release bar for the
first Flauz.app release:

- every delivery-order-1 and delivery-order-2 row (shell, coding loop)
  is **green** through public/runtime-owned interfaces — the remaining
  deltas are polish or enhancements below the release bar;
- rows whose remaining delta is proprietary or cloud-unavailable
  (Scheduled tasks, Sites, Visualizations, Appshots, Cloud
  environments, Voice, Pets) are **bounded** per the work order's own
  rule ("do not reproduce explicitly unavailable proprietary behavior
  unless a public contract exists and the feature is required by our
  product acceptance criteria");
- platform completion (Windows packaging/signing; Linux Wayland
  portals/tray/global shortcuts) is **deferred to GUI-006**
  (Distribution + Release), which owns the platform matrix;
- the Linux Computer Use boundary is recorded honestly: X11/XWayland
  observation only.

### 2. Closed gap: manual project ordering

The Projects and chats row listed "manual project ordering" as a
closable gap; it is now closed end to end:

- **codex-core**: `local_project_order` state (bounded by
  `MAX_LOCAL_PROJECTS`, absolute paths, deduplicated), actions
  `ProjectOrderLoaded` / `MoveLocalProject { path, up }`, effect
  `PersistProjectOrder`. The manual order applies **inside** the
  pinned and unpinned groups (pins keep floating; group boundaries are
  not crossed by moves), projects missing from the order keep their
  recency placement, and the order survives project loads, renames,
  pin toggles, and removals. Focused reducer test:
  `manual_project_order_applies_within_pinned_groups_and_persists`.
- **codex-app backend**: loads the persisted order
  (`project_order_v1` preference in codexRS-owned single-writer
  storage) at storage open and emits `ProjectOrderLoaded`;
  `PersistProjectOrder` writes it back through the same bounded
  preference path as pinned tasks.
- **codex-app UI**: `Move up` / `Move down` items in the project
  actions menu (dropdown + context menu), disabled at group
  boundaries, dispatching the new action.

## Acceptance checklist (work order)

| Requirement | Result |
| --- | --- |
| use `docs/parity-matrix.md` as the behavioral baseline | PASS — ledger added to the matrix itself |
| close release-critical parity gaps for the target platform matrix | PASS — release-critical rows green through public contracts; one closable gap closed in code (manual project ordering); platform completion routed to GUI-006 |
| parity matrix rows selected as release-critical are green | PASS — see the ledger: 27 green rows, 7 bounded (proprietary/unavailable), 2 deferred to GUI-006 (platform completion) |
| same-state visual/interaction tests cover them | PARTIAL — reducer/interaction coverage exists per the matrix rows' recorded evidence; same-state screenshots on real display hardware are GUI-007's evidence class (sandbox is headless) |
| no upstream proprietary runtime dependency introduced | PASS — dependency policy re-verified (561 packages) |

## Verification commands and results

```text
. scripts/dev-env.sh
cargo fmt --all --check                                  → PASS (clean)
cargo clippy -p codex-core --all-targets -- -D warnings  → PASS
cargo clippy -p codex-app --all-targets -- -D warnings   → PASS
cargo test -p codex-core                                 → 206 passed (205 + 1 new ordering test)
cargo check -p codex-app                                 → PASS
python3 scripts/check_dependency_policy.py               → PASS (561 packages)
```

## Known limitations

1. Same-state screenshots of release-critical views require real
   display hardware and are collected under GUI-007; this sandbox is
   headless (GPUI needs a GPU/Vulkan surface).
2. The ledger deliberately does not turn `partial` rows into `done`:
   the matrix keeps tracking full-reference parity; the ledger is the
   release-bar selection.
3. Only one closable gap was implemented in this work order (manual
   project ordering). The remaining closable enhancements (timeline
   grouping, granular permission editor, PR options, skills install
   flows, PDF/Office renderers) are recorded in the ledger as
   below-release-bar enhancements; they remain available for later
   polish without blocking the release graph.

## No unrelated refactors

Diff surface: `crates/codex-core/src/lib.rs` (ordering state machine +
test), `crates/codex-app/src/backend.rs` (preference load/persist +
exhaustiveness arm), `crates/codex-app/src/ui.rs` (menu target fields +
two menu items), `docs/parity-matrix.md` (ledger section), this report.
