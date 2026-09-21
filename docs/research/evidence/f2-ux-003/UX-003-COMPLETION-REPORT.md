# UX-003 — Worker D completion report (archived)

> Archived 2026-09-21 by the Tech Lead from the live agents-tab session
> (chat 2f76f2f6-61dc-420d-b436-923d0e8ac604, "UX-003 Accessibility Fixes
> Overview" — recovered twice from sandbox-pod resets via the Lead
> continuation protocol). Raw capture:
> replay2/scripts/worker-reports/ux-003-response-raw.txt.
> Lead gates: bundle verified on base 67e2b74; PR #44 CI green (both
> platforms); merged via PR #44 — worker commit ce74a7c (ui.rs only,
> +478/-14), merge 4378892 = origin/main. Contract deviations: NONE.
> Architecture deviations: NONE. Notable: the N6 root cause is a genuine
> platform finding (Linux xkb reports Ctrl+Shift+] as the shifted keysym
> "}"; the fix extends the canonical physical-position normalization).
> Lab probes (PTY focus, swap re-evidence, promo capture) = Lead gate work
> per the plan.

=== UX-003 COMPLETION REPORT ===
Work order: UX-003 — F2 accessibility carry-overs (N5 PTY focus transfer, N6 bracket-swap chords, first-run NUX modal keyboard honesty)
Branch/commit: feat/ux-003-a11y-carryovers @ ce74a7ccd6f6c7141d203894eb969f0fed380da4 — single clean commit on base 67e2b74c8910c4288cb60b8f757f775c35c4e7b4 (= origin/main HEAD)
Delivery: /home/z/my-project/UX-003-delivery.bundle — git bundle verify OK (contains refs/heads/feat/ux-003-a11y-carryovers @ ce74a7c; requires base 67e2b74). NOT pushed.

Changed contracts: PRODUCT-UX-JOURNEYS.md §7 (accessibility) keyboard guarantees — N5 terminal-dock open-path focus transfer, NUX modal one-Escape contract, N6 chord resolution. No contract documents modified (work order scoped changes to ui.rs only).

Changed files/subsystems: crates/codex-app/src/ui.rs ONLY — +478/−14 (three fix sites + four focused unit tests + house-sorted test-module imports). Nothing else.

ACCEPTANCE CRITERIA MAPPING:

N5 CLOSED: typed input lands in the PTY after open (lab probe).
→ Code-level: every dock-open path (Ctrl+on a selected live chat, palette row, sidebar/footer toggle, bottom-panel toggle) funnels through the single reduce wrapper that armsterminal_dock_focus_pending; the first render that mounts the PTY input (tab.running && !tab.stopping— the input's own mount condition) consumes the arming viaterminal_dock_focus_transfer_due(focus_pending, pty_input_mounted, !overlay_open), captures the prior focus, and focuses the PTY surface; occluding overlays (workspace modal, command palette, Activity view) defer the transfer; the dock close path restores focus through the WO-P2-017 contract helper. Tests: terminal_dock_focus_transfer_requires_pending_input_and_no_overlay(truth table) +terminal_dock_open_path_transfers_focus_and_the_close_path_restores_it` (dispatch arming + render transfer/capture/restore source pins) — both green. Lab probe = Lead gate (work order: "the Lead runs them at the gate").
N6 CLOSED: swap chords positively evidenced in the two-chat fixture.
→ PROVEN code-level cause found, so the fix shipped per the "IF a code-level cause is found" clause: on Linux, GPUI Keystroke::from_xkb reports Ctrl+Shift+] as key "}" (the shifted keysym; vendored platform.rs bracket keysym table) while the registry expresses the chord as Ctrl+Shift+] (unshifted key + Shift modifier); canonical_shortcut_key passed "}" through, so the keystroke interceptor fell through silently — Ctrl+PageDown (no Shift) matched fine, the exact lab asymmetry. Fix: canonical_shortcut_key maps "{"/"}"/braceleft/braceright to the unshifted brackets, extending the established Cyrillic (х/ъ) physical-position normalization. Also verified: no composer-focus guard exists on the chord path (the interceptor is window-global, runs before bindings); the no-selection E-leg no-op is the same mapping gap. Test: bracket_swap_chords_resolve_from_the_shifted_linux_key_forms (canonical mapping + full-chord resolution from a constructed gpui::Keystroke + registry ownership) — green. Two-chat fixture re-evidence = Lead gate.
First-run: keyboard path documented + verified at the current version (no silent swallowing without an escape hatch).
→ The "Introducing GPT-5.6-Sol" NUX modal joins the WO-P2-019 family: model_availability_nux_focus handle (track_focus + tab_group + tab_stop + key_context), scoped tab/shift-tab bindings cycling the dismiss affordances via cycle_model_availability_nux_focus, request-once mount auto-focus, and a scoped escape binding (KeyBinding::new("escape", Escape, Some("ModelAvailabilityNuxModal"))) so ONE Escape dispatches through the modal's focus context to the overlay's existing Escape handler instead of the keystroke-observer last resort. Click paths unchanged; the seen-flag per-version re-display stays untouched (deliberate product behavior). Test: model_availability_nux_modal_owns_the_keyboard_with_one_escape (bindings incl. scoped escape, cycle-handler family shape, panel wiring, request-once mount focus) — green. Fresh-profile promo capture = Lead gate.
Non-empty Contract deviations blocks closure.
→ Contract deviations: NONE. Architecture deviations: NONE.

TESTS + EXACT RESULTS (entire ladder re-run this turn on the surviving commit):

cargo fmt --all --check: CLEAN.
cargo clippy --workspace --all-targets: zero workspace warnings (only the pre-existing third-party proc-macro-error2 future-incompat note, present on base).
cargo check -p codex-app: Finished.
cargo test (all linkable binaries): codex-app 244 passed / 0 failed (includes the 4 new UX-003 tests, each also run individually and passing); codex-core 235; codex-protocol 60; codex-storage 19; codex-platform 114 + 2 pre-existing ignored + 2 boundary; flauz-world 31 + 6 + 9 — aggregate 720 passed, 0 failed.

Journey IDs exercised: J-01, J-03 (keyboard recovery), J-17.
GUI discoverability layers covered: N/A (Worker C's lane).

EVIDENCE PRODUCED:

Single clean commit ce74a7c (ui.rs only, +478/−14) on feat/ux-003-a11y-carryovers.
Bundle /home/z/my-project/UX-003-delivery.bundle (verified; requires base 67e2b74). NOT pushed.
N6 root-cause proof: vendored platform.rs keysym table vs registry chord-form mismatch, with pre-fix fall-through and post-fix resolution pinned by test.
Worklog: two sections appended to /home/z/my-project/worklog.md (original run + this recovery re-verification).

KNOWN LIMITATIONS:

Lab/GUI evidence (PTY focus probe, two-chat swap re-evidence, fresh-profile promo capture) remains Lead-gate work per the work order; the code-level contracts are implemented and unit-pinned.
Responsive-resize dock-restore intentionally does NOT arm the focus transfer (resize is not a user open intent).
The keystroke-observer Escape fallback stays in place as defense-in-depth behind the contracted binding path (contract path wins; the observer's no-double-dismiss guard is intact).
Sandbox GUI linking uses user-space stubs (no root on pod); validation ran on all linkable test binaries — no behavioral difference expected.
The NUX keyboard contract should be re-verified at each future seen-flag version bump (the re-display itself is intentional).

RECOMMENDED FOLLOW-UP:

Lead gate: run the three lab scenes (PTY probe, swap re-evidence, promo capture).
Re-check the NUX one-Escape contract at each version bump.
Upstream-GPUI platform-bound a11y items stay parked at F11 per the work order non-goals.
=== END REPORT ===
