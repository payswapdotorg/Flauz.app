# ADR-001: Context, Harness, World-State and UX Discoverability

**Status:** Accepted  
**Date:** 2026-09-19  
**Scope:** Flauz platform evolution after Codex Desktop parity  
**Supersedes:** none

## Decision

Flauz will evolve from a Codex-compatible agent desktop into a general-purpose
agent workspace with first-class:

- Context Engine;
- Harness;
- Task Execution Graph;
- Resource Graph;
- Evidence/Verification plane;
- resource leases/conflict handling;
- reusable Procedures;
- collaboration state;
- domain-neutral Project/Task/Artifact concepts;
- GUI discoverability contracts for all major platform capabilities.

The detailed runtime contract is docs/CONTEXT-HARNESS-ARCHITECTURE.md.

The corresponding GUI journey contract is docs/PRODUCT-UX-JOURNEYS.md.

## Why

Recent agent-system research increasingly treats long-horizon state,
context management, harness design, evaluator loops, tool/resource
coordination, and human collaboration as distinct engineering concerns rather
than properties of the underlying model.

The current Flauz architecture already separates Client, Model, AgentRuntime,
Skill, Environment and Provider. The approved extension preserves that
separation and adds the missing state/context/execution boundaries required for
multi-environment and multi-domain work.

## Architectural consequences

1. Session history is durable state; model context is a projection of that state.
2. Context compilation is model-aware and permission-aware.
3. Harnesses own execution loops, recovery, verification and context assembly;
   models remain interchangeable intelligence providers.
4. A Resource may have multiple access surfaces (browser/API/CLI/MCP/native).
5. Claims and observations are not equivalent to verified evidence.
6. Parallel reads and proposals are normal; writes and commits require explicit
   coordination where resources conflict.
7. Procedures are durable reusable product objects, not hidden agent memory.
8. Users and agents collaborate through shared canonical task state while
   preserving private context.
9. Environment changes do not change logical task identity.
10. Every major capability has a GUI journey, visible entry, contextual
    affordance, search/palette fallback, empty-state guidance and
    success/next-step affordance where applicable.

## Rollout constraint

Codex Desktop parity remains the immediate implementation priority. During F1,
the new architecture is limited to contracts/documentation and narrow seams
needed to prevent hard coupling. Runtime implementation begins through the
ordered roadmap and bounded work orders.

## Non-goals

This ADR does not:

- choose one model vendor;
- make E2B canonical;
- require a vector database;
- require a particular orchestration framework;
- make browser, terminal or MCP a special universal execution path;
- make software development the canonical project type;
- authorize implementation outside a named work order.

## Acceptance principle

Architecture work is not product-complete until the corresponding user journey
can be discovered through the GUI and verified without requiring knowledge of
Flauz's internal terminology.
