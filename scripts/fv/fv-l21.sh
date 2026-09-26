#!/bin/bash
# FV-L21 — FR/PKG slice (the fresh-machine contract; Linux desktop lane,
# FV-002 / Wave 7): FR-1 the About window, FR-3 the empty-workspace
# "Open folder" offer + the honest no-workspace guard, PKG-2
# --install-desktop-entry.
#
# CALIBRATION (2026-09-25, verified against the pinned base
# 7f660c00407a5741eee975b2274570a200ff576b):
#   FR-1 (the About window):
#     - Help → "About codexRS" (PopupMenuItem::new("About codexRS") at
#       crates/codex-app/src/ui.rs:15378) opens the floating,
#       fixed-size, centered window: ABOUT_WINDOW_WIDTH 380.0 /
#       ABOUT_WINDOW_HEIGHT 360.0 (ui.rs:214-215), WindowKind::Floating,
#       non-resizable, titled "About codexRS" (ui.rs:11547-11558), with
#       the scoped Escape (ui.rs:5875, AboutDialog) and the OK afford-
#       ance (the AboutView, ui.rs:6103-6129).
#     - The FR binding precedent (f1-sweep/binding/FR-BINDING-VERDICT.md
#       fr1d): xwininfo Width 380 / Height 360 exactly; refocus on a
#       second open; the menu drive used the calibrated click placement
#       (Help (375,65) → About (416,275)) — THIS scene's one documented
#       pointer exception (menus; the fr1d placement law), named here
#       per the d-series discipline.
#   FR-3 (the empty-workspace offer + guard):
#     - "Open folder" affordance: the registry row "openFolder"
#       (CmdOrCtrl+O; ui.rs:3510/3965) + the File-menu entry (ui.rs
#       :19410 "Open folder" label) + the composer picker guard
#       ("composer.openProjectPicker", ui.rs:12257-12259).
#     - the honest no-workspace guard: Ctrl+P → "Select a workspace
#       before searching files." (files_palette_command_status,
#       ui.rs:48202-48209).
#     - BOUND (named, catalog gap L-5): no native folder picker on Xvfb
#       (no portal backend in the lab image) — FR-3's picker step stays
#       the documented headless bound.
#   PKG-2 (--install-desktop-entry):
#     - main.rs:47 routes the flag to run_install_desktop_entry
#       (main.rs:57-65) → install_linux_desktop_entry
#       (codex-platform/src/linux_desktop_entry.rs:73-108): the per-user
#       entry lands at $XDG_DATA_HOME/applications/com.codexrs.CodexRS
#       .desktop (LINUX_DESKTOP_ENTRY_FILE_NAME, linux_desktop_entry.rs:
#       11) and NEVER overwrites an existing entry (install_desktop_
#       entry_at :129-141 writes via a temp file + rename only when the
#       destination is absent — the "never overwrites" contract).
#
# Moments (the catalog FV-L21 row):
#   01 entry baseline (fresh profile — the fresh-machine contract)
#   02 the empty-workspace "Open folder" offer (the entry surface /
#      File-menu affordance; the Ctrl+O chord fires the honest bound)
#   03 the honest no-workspace guard (Ctrl+P → the named status)
#   04 the Help menu → "About codexRS" (the fr1d placement-law clicks —
#      the scene's named pointer exception)
#   05 the About window open — xwininfo Width 380 / Height 360 exactly;
#      VLM: the version string of THIS build
#   06 the About window closes via Escape (the scoped binding)
#   07 (separate process run) --install-desktop-entry: the per-user
#      entry created; a second run leaves it UNCHANGED (md5 + content
#      compare)
#
# PASS (Lead-adjudicated):
#   - xwininfo: Width 380 / Height 360 exactly;
#   - VLM: the version string of THIS build;
#   - the entry file created (and unchanged on the second run);
#   - the picker stays the documented headless bound (gap L-5).
#
# Usage: scripts/fv/fv-l21.sh <binary-path> [outdir]
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/fv-lib.sh"
fv_need_runtime

fv_begin fv-l21 "FR/PKG" "${1:?binary path required}" "${2:-}"

moment 01 "entry baseline (fresh profile — the fresh-machine contract)"
cap fvl21-01-entry

moment 02 "the empty-workspace 'Open folder' offer (Ctrl+O fires the affordance; the picker is the documented L-5 bound)"
key ctrl+o; sleep 2.5
cap fvl21-02-open-folder-offer
key Escape; sleep 1.5

moment 03 "the honest no-workspace guard (Ctrl+P → 'Select a workspace before searching files.')"
key ctrl+p; sleep 2.5
cap fvl21-03-no-workspace-guard
key Escape; sleep 1.5

moment 04 "the Help menu → 'About codexRS' (the fr1d placement-law clicks — this scene's one named pointer exception: menus)"
say "POINTER EXCEPTION (named): the Help dropdown menu drive uses the FR-BINDING fr1d calibrated placement (Help (375,65) → About (416,275) at 1440x900); every other leg is keyboard-only"
DISPLAY="$FV_DISP" xdotool mousemove 375 65 click 1; sleep 1.5
cap fvl21-04a-help-menu-open
DISPLAY="$FV_DISP" xdotool mousemove 416 275 click 1; sleep 2.5
cap fvl21-04b-about-menu-clicked

moment 05 "the About window open — xwininfo Width 380 / Height 360 exactly (the FR-1 contract)"
ABOUT_WID="$(xwininfo -display "$FV_DISP" -root -children 2>/dev/null | grep -B1 -A0 '"About codexRS"' | grep -oE '0x[0-9a-f]+' | head -1)"
if [ -z "$ABOUT_WID" ]; then
  ABOUT_WID="$(xwininfo -display "$FV_DISP" -root -children 2>/dev/null | grep -iE 'about' | grep -oE '^ *0x[0-9a-f]+' | head -1 | tr -d ' ')"
fi
if [ -n "$ABOUT_WID" ]; then
  xwininfo -display "$FV_DISP" -id "$ABOUT_WID" > "$FV_OUT/about-window-full-xwininfo.txt" 2>/dev/null
  AW="$(xwininfo -display "$FV_DISP" -id "$ABOUT_WID" 2>/dev/null | awk '/Width:/ {print $2}')"
  AH="$(xwininfo -display "$FV_DISP" -id "$ABOUT_WID" 2>/dev/null | awk '/Height:/ {print $2}')"
  say "About window geometry: Width ${AW:-?} / Height ${AH:-?} (expect 380 / 360 exactly — ui.rs:214-215)"
  if [ "${AW:-0}" = "380" ] && [ "${AH:-0}" = "360" ]; then
    say "FR-1 GEOMETRY: PASS (380x360)"
  else
    say "FR-1 GEOMETRY: MISMATCH — the Lead adjudicates (the full xwininfo is archived)"
  fi
else
  say "WARN: About window not found via xwininfo — the Lead adjudicates from the frames (the fr1d VLM honest-note precedent)"
fi
cap fvl21-05-about-open

moment 06 "the About window closes via Escape (the scoped AboutDialog binding)"
key Escape; sleep 2
cap fvl21-06-about-closed

moment 07 "(separate process run) --install-desktop-entry: created once; the second run leaves it UNCHANGED"
PKG_HOME="$(mktemp -d /tmp/fvl21-pkg-home.XXXXXX)"
PKG_DATA="$PKG_HOME/.local/share"
DISPLAY="$FV_DISP" HOME="$PKG_HOME" XDG_DATA_HOME="$PKG_DATA" "$FV_BIN" --install-desktop-entry > "$FV_OUT/pkg-entry-run1.log" 2>&1
ENTRY="$PKG_DATA/applications/com.codexrs.CodexRS.desktop"
if [ -f "$ENTRY" ]; then
  say "PKG-2: entry created at $ENTRY"
  md5sum "$ENTRY" > "$FV_OUT/pkg-entry-md5.txt"
  cp "$ENTRY" "$FV_OUT/pkg-entry-run1.desktop"
  sleep 1
  DISPLAY="$FV_DISP" HOME="$PKG_HOME" XDG_DATA_HOME="$PKG_DATA" "$FV_BIN" --install-desktop-entry > "$FV_OUT/pkg-entry-run2.log" 2>&1
  M1="$(md5sum "$FV_OUT/pkg-entry-run1.desktop" | awk '{print $1}')"
  M2="$(md5sum "$ENTRY" | awk '{print $1}')"
  if [ "$M1" = "$M2" ]; then
    say "PKG-2 NEVER-OVERWRITES: PASS (second run left the entry byte-identical)"
  else
    say "PKG-2 NEVER-OVERWRITES: MISMATCH ($M1 vs $M2) — the Lead adjudicates"
  fi
  echo "$M2" >> "$FV_OUT/pkg-entry-md5.txt"
else
  say "WARN: PKG-2 entry not found at $ENTRY — the run logs are archived (pkg-entry-run1.log); the Lead adjudicates"
fi
rm -rf "$PKG_HOME"

say "RUN NOTE (catalog gap L-5): no native folder picker on Xvfb (no portal backend) — FR-3's picker step stays the documented headless bound; the honest no-workspace guard + the offer affordance are the verifiable truth"

fv_end
