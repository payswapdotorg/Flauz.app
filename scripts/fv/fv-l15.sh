#!/bin/bash
# FV-L15 — J-14 Switch model (the model picker; Linux desktop lane,
# FV-002 / Wave 7).
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   Chords:
#     - model picker: Ctrl+Alt+Shift+M — KeyBinding::new(
#       &shortcut("alt-shift-m"), FlauzModelPickerShortcut) at
#       crates/codex-app/src/ui.rs:5806 (the MOD-001 + RT-001 chord)
#     - scoped Escape: KeyBinding::new("escape", Escape,
#       Some("FlauzModelPicker")) at ui.rs:5896
#     - the composer model selector: the composer's model picker
#       (composer_model_picker_items, ui.rs:14862 — the composer-side
#       control; visible on the task surface)
#   Copy (crates/codex-app/src/ui/flauz_model_picker.rs):
#     - MODEL_PICKER_HEADING "Choose a model" :115; the identity
#       promise MODEL_PICKER_DESCRIPTION "The model is the engine that
#       reads your instructions and does the work. Switching models
#       keeps this task — its objective, its plan, and everything it
#       produced — exactly as it is." :119-121
#     - the honest empty: MODEL_EMPTY_TITLE "No models connected yet"
#       :124; MODEL_EMPTY_BODY "A model is what does the work of a task
#       — reading your instructions, writing, analyzing, and planning.
#       Different models bring different strengths; for example, some
#       can understand images. Models become available when you connect
#       a provider." :127-130 (what a model provides)
#     - the Settings→Connections next step: MODEL_EMPTY_NEXT_STEP "To
#       make models available, connect a provider: open Settings, then
#       Connections, and follow the steps there. …" :135-137 (+ the "Open
#       Connections settings" button :141)
#     - MODEL_PICKER_ESCAPE_HINT "Escape closes this panel" :167
#   Donor-state bound (named): with no provider connected in the lab
#   (the auth wall, gap L-1), a model switch cannot run live; the
#   identity-promise copy + the composer model selector are the
#   verifiable truth — the switch-keeps-task frame-compare is bounded to
#   the copy's unit-pinned contract (MODEL_SWITCHED_CONFIRMATION :148-
#   149).
#
# Moments (the catalog FV-L15 row):
#   01 entry baseline
#   02 anchor task (the composer model selector visible on the task
#      surface)
#   03 Ctrl+Alt+Shift+M — the model picker (the honest empty + what a
#      model provides + the identity promise)
#   04 settle/detail frame (the Settings→Connections next step readable)
#   05 scoped Escape — the picker closes (the task surface stands)
#   06 the composer model selector (settle frame — VLM read)
#
# PASS (Lead-adjudicated, VLM reads):
#   - "No models connected yet" + what-a-model-provides + the
#     Settings→Connections next-step + "Switching models keeps this task
#     — its objective, its plan, and everything it produced — exactly as
#     it is" verbatim;
#   - the composer model selector visible (02/06);
#   - donor-state: the task surface is unchanged across the switch —
#     BOUNDED (the auth wall; the run notes name it; the copy's
#     identity promise is the current verifiable truth).
#
# Usage: scripts/fv/fv-l15.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l15 "J-14" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline"
cap fvl15-01-entry

moment 02 "anchor task (the composer model selector visible on the task surface)"
key ctrl+n; sleep 2.5
type_ "FV-L15 anchor task"; sleep 1.5
key ctrl+Return; sleep 6
cap fvl15-02-task-surface-composer-selector
TASK_FRAME="$(frame_md5 fvl15-02-task-surface-composer-selector.png)"
say "task-surface frame md5 (for the unchanged-across-switch compare): $TASK_FRAME"

moment 03 "Ctrl+Alt+Shift+M — the model picker (the honest empty + the identity promise)"
key ctrl+alt+shift+m; sleep 2.5
cap fvl15-03-model-picker-empty

moment 04 "settle/detail frame (the what-a-model-provides body + the Settings→Connections next step)"
sleep 1.5
cap fvl15-04-model-picker-detail

moment 05 "scoped Escape — the picker closes (the task surface stands)"
key Escape; sleep 2
cap fvl15-05-picker-closed
AFTER_FRAME="$(frame_md5 fvl15-05-picker-closed.png)"
say "post-close frame md5 (compare vs the task frame): $AFTER_FRAME"

moment 06 "the composer model selector (settle frame — VLM read)"
sleep 2
cap fvl15-06-composer-model-selector

say "RUN NOTE (catalog gap L-1, the auth wall): no provider is connected in the lab, so a live model switch cannot run; the identity promise ('Switching models keeps this task — …') + the composer model selector are the verifiable truth at this base; the switch-keeps-task behavior is unit-pinned (MODEL_SWITCHED_CONFIRMATION, flauz_model_picker.rs:148-149) — no data invented"

fv_end
