# Work Order Template / Execution Protocol

Every implementation change must map to one bounded work order.

## Required fields

```
ID:
Title:
Phase:
Owner:
Dependencies:
Contract(s):
Problem:
User-visible outcome:
Scope:
Non-goals:
Files/subsystems owned:
Inputs:
Outputs/artifacts:
Tests:
GUI/lab evidence:
Acceptance criteria:
Rollback/recovery:
Integration notes:
Status:
```

## Closure gates

A work order is CLOSED only when applicable:

1. Contract — frozen architecture/schema is respected.
2. Behavior — acceptance criteria demonstrated.
3. Tests — focused tests plus required repository checks pass.
4. Boundary — no unauthorized trust/data/runtime boundary crossed.
5. Evidence — UI/provider/lab behavior is reproducible.
6. Integration — Tech Lead reviewed merged impact.

## Worker rules

Before coding, read:

- `AGENTS.md`
- `docs/FLAUZ-SOURCE-OF-TRUTH.md`
- `docs/IMPLEMENTATION-ROADMAP.md`
- this file
- `docs/architecture.md`
- the relevant parity rows/work-order history

Workers must preserve behavior outside scope, inspect existing code first, keep
ownership bounded, and report exact files/tests/evidence.

Workers must not invent competing abstractions, silently redefine frozen
architecture, hard-code providers into product contracts, or commit credentials.

## Tech Lead rules

- Verify current `main` before dispatch.
- Keep up to three active implementation work orders by default.
- Give workers non-overlapping ownership.
- Merge focused PRs to `main`.
- Reconcile worker output against repository contracts, not worker prose.
- Update status only after evidence.
- Record architecture amendments before implementation.
- Re-run affected journeys after UI/provider/client changes.

## Branch names

```
feat/<work-order-id>-<short-name>
fix/<work-order-id>-<short-name>
test/<work-order-id>-<short-name>
docs/<work-order-id>-<short-name>
```

## Worker completion report

```
Work order:
Implementation summary:
Files changed:
Tests run:
GUI/lab evidence:
Known limitations:
Contract deviations:
Follow-up work:
```

Non-empty contract deviations block closure until the Tech Lead resolves them.
