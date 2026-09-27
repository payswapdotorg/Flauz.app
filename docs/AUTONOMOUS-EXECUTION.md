# Autonomous Execution

Flauz.app is designed to be executable without prior chat context.

## Canonical control loop

```
read ACTIVE-EXECUTION-STATE
→ read the active work-order file
→ verify current main SHA
→ choose only actionable work
→ dispatch disjoint work
→ harvest outputs
→ run acceptance gates
→ merge
→ rerun affected verification
→ update ACTIVE-EXECUTION-STATE
```

## Priority order

1. unblock/finish production work already dispatched;
2. recover W-AUTH and complete the missing Web formal scenes;
3. reconcile verification after post-FV changes;
4. close production release gates;
5. stop.

Never create work merely to keep the lead active.

## Repository truth hierarchy

```
current main + merged evidence
        >
active execution state
        >
frozen architecture / roadmap
        >
historical work orders
        >
worker reports
        >
chat memory
```

When documents disagree, verify the repository and update the operational state.

## Current SHA rule

Before dispatch:
`git fetch --all --prune && git checkout main && git pull --ff-only`

Record:
`git rev-parse HEAD`

Do not use a worker's claimed base SHA until it is verified locally.

## Closure rule

A task is CLOSED only when the implementation or artifact is on main and its acceptance evidence exists in the repository.

A docs-only handoff is not implementation closure.

## External blockers

For an external blocker, record:
- blocker name;
- observed failure;
- exact recovery action;
- affected work orders;
- whether work can proceed independently.

Do not convert an external blockage into a fake repository state.

## Final stop

Once production is released and all required verification is current, TL #1 stops continuous execution. Future work proceeds by new repository-recorded work orders.
