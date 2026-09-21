# Linux GUI — Regression Ledger

**Lane:** Linux GUI usability (second Tech Lead — Linux lane).

Every defect observed in the E2B desktop journeys is registered here with
its classification, the journey that surfaced it, the work order that fixes
it, and the E2B re-run that closes it. A defect is CLOSED only when the
same journey re-runs green (or honestly-unavailable) in E2B against merged
`main`.

Severity classes: **P0** crash / persistent unusable GUI / data-loss risk /
startup failure · **P1** major supported journey broken · **P2** important
interaction, discoverability, focus or state problem · **P3** minor visual /
parity / polish · **BOUNDED** explicit architectural/platform limitation
with a truthful user-facing explanation (a capability is NOT bounded merely
because it is difficult).

| ID | Date | Journey | Class | Summary | Evidence | Work order | Fixed @ | Re-run verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| L-001 | 2026-09-21 | E-00 environment provision | P1 | Linux build fails on Ubuntu 22.04 (E2B desktop standard image): `libspa 0.10.0` binding errors against pipewire 0.3.48 headers via `codex-platform → xcap 0.9.7 → pipewire 0.10 → libspa 0.10`. CI validates Ubuntu 24.04 only; `platform-support.md` "other Linux: source expected" is currently false for 22.04. Worked around in-lane by side-loading pipewire 1.0.5 dev+runtime (Ubuntu 24.04 debs, `~/pw10`, PKG_CONFIG_PATH + LD_LIBRARY_PATH). | prov.log; E2B-ENVIRONMENT doc | LAB-001 (Worker A) | — | OPEN |
| L-002 | 2026-09-21 | J-01 start a project (cold start) | P0 | GUI launches (window mapped, focused, WM class `com.codexrs.CodexRS`, 32-bit ARGB visual) but **never presents rendered frames**: window content stays black on (a) E2B Desktop Xfce4/Xvfb :0 and (b) bare Xvfb :98 (no WM), with `LIBGL_ALWAYS_SOFTWARE=1`. Process healthy: 32 threads, event loop idle (epoll_pwait), 2 Hz timer, zero stderr, no crash, no app-server child spawned. `scripts/linux_desktop_smoke.sh` asserts liveness only (exit 124 at 15 s) — never pixels — so this failure class ships green through CI. | evidence/run1-j01-02 (Xfce), run1-j01-06 (bare Xvfb root all-black), run1-j01-05 (xwd window content black) | LAB-003 (Worker A, queued after LAB-001) | — | OPEN |
| L-003 | 2026-09-21 | E-00 environment provision | P3 | Without `XDG_RUNTIME_DIR` the app runs but logs `error: XDG_RUNTIME_DIR not set in the environment.` (4 lines). The repo's own smoke script sets it; a real desktop session always has it. Minor, but the smoke harness contract should be documented in platform-support. | prov.log | (bundled into LAB-001 scope note) | — | OPEN |
