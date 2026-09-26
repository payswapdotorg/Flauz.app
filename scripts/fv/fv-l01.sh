#!/bin/bash
# FV-L01 — J-01 Start any project (Linux desktop lane, FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b — the d26 law: every chord,
# anchor, and copy string below was read at this base):
#   Chords (crates/codex-app/src/ui.rs):
#     - palette: Ctrl+K — KeyBinding::new(&shortcut("k"), OpenCommandMenu)
#       at ui.rs:5739 (shortcut() prefixes "ctrl-" on non-macOS, ui.rs:5731-5736)
#     - new task: Ctrl+N — KeyBinding::new(&shortcut("n"), NewChatShortcut)
#       at ui.rs:5743
#     - workspace nav family: Ctrl+Alt+1..4 — FlauzProjectsTasksShortcut …
#       FlauzActivityShortcut at ui.rs:5787-5790
#     - composer submit: Ctrl+Return (the D8c/D9/D11r calibration — plain
#       Return does NOT submit; wo-p2-008 README "FW-5 key fix")
#   Anchors/copy (crates/codex-app/src/ui/flauz_shell/mod.rs):
#     - nav labels "Projects & tasks" / "Reusable workflows" / "Artifacts" /
#       "Activity" (WorkspaceNavSurface::nav_label, mod.rs:96-99)
#     - surface descriptions mod.rs:121-124; honest empty titles mod.rs:151-154
#   Title-bar palette entry (WO-P2-018): render_command_palette_entry_button
#   at ui.rs:15581 (the palette + Activity bell + members + conflicts +
#   needs-you buttons ride the title bar, ui.rs:15571-15596).
#   Fresh-profile promo modal (UX-003): the model-availability NUX modal
#   owns one scoped Escape (ui.rs:5943, ModelAvailabilityNuxModal); the
#   45576/55832 test region pins the one-Escape + no-swallowed-keyboard
#   contract (a typed probe must land in the composer after the close).
#
# Moments (the catalog FV-L01 row):
#   01 wake click → first frame (entry surface, fresh profile)
#   02 the promo modal (if fresh profile) dismissed with ONE Escape —
#      then a typed probe lands in the composer (no swallowed keyboard)
#   03 Ctrl+K palette open (title-bar entry visible in chrome all along)
#   04 palette new-task row
#   05 Ctrl+N new-task entry + objective typed
#   06 Ctrl+Return → the task surface lands (objective echoed)
#   07-10 workspace navigation: Ctrl+Alt+1..4 (the four nav surfaces)
#
# PASS (Lead-adjudicated, VLM reads on the frames):
#   - the four nav labels + the title-bar palette entry visible in chrome;
#   - the new-task objective echoed on the task surface;
#   - the promo modal (if fresh profile) gone after ONE Escape, with the
#     typed probe landing in the composer.
#
# Usage: scripts/fv/fv-l01.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

# FV-L01 owns the promo-modal moment: skip the library's defensive boot
# dismissal so the scene can observe the one-Escape contract itself.
FV_SKIP_BOOT_DISMISS=1
fv_begin fv-l01 "J-01" "${1:?binary path required}" "${2:-}"

moment 01 "wake click already applied by the boot loop — first frame (entry surface, fresh profile)"
cap fvl01-01-entry

moment 02 "promo modal (if fresh profile) dismissed with ONE Escape (the UX-003 contract)"
key Escape; sleep 2
cap fvl01-02-after-one-escape
type_ "fv probe lands"; sleep 1.5
cap fvl01-02b-typed-probe
key ctrl+a; key Delete; sleep 1

moment 03 "Ctrl+K palette open (title-bar entry visible in chrome)"
key ctrl+k; sleep 2.5
cap fvl01-03-palette-open

moment 04 "palette new-task row (type-ahead filter)"
type_ "new task"; sleep 2
cap fvl01-04-palette-newtask-row
key Escape; sleep 1.5

moment 05 "Ctrl+N new-task entry + objective typed"
key ctrl+n; sleep 2.5
cap fvl01-05-new-task-surface
type_ "FV-L01 anchor task for the community garden plan"; sleep 1.5
cap fvl01-05b-objective-typed

moment 06 "Ctrl+Return submit — the task surface lands (objective echoed)"
key ctrl+Return; sleep 6
cap fvl01-06-task-surface
sleep 3
cap fvl01-06b-task-surface-settled

moment 07 "Ctrl+Alt+1 — Projects & tasks surface"
key ctrl+alt+1; sleep 2.5
cap fvl01-07-nav-projects-tasks

moment 08 "Ctrl+Alt+2 — Reusable workflows surface"
key ctrl+alt+2; sleep 2.5
cap fvl01-08-nav-reusable-workflows

moment 09 "Ctrl+Alt+3 — Artifacts surface"
key ctrl+alt+3; sleep 2.5
cap fvl01-09-nav-artifacts

moment 10 "Ctrl+Alt+4 — Activity surface"
key ctrl+alt+4; sleep 2.5
cap fvl01-10-nav-activity

moment 11 "Escape settle — final frame (return to the task surface)"
key Escape; sleep 2
cap fvl01-11-settle

fv_end
