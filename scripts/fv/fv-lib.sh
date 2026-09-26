#!/bin/bash
# fv-lib.sh — the shared Linux FV scene library (FV-002, Wave 7).
#
# The sealed d-series recipe (the d24/d25/d26 precedents — see
# docs/research/evidence/w4-gate/d26-w4-surfaces.sh) factored once so every
# scripts/fv/fv-lNN.sh scene script stays scene-specific:
#
#   - hard-fail with a NAMED, actionable message when any prerequisite is
#     missing (never silent, never fabricated — WAVE7 addendum §7);
#   - LINUX_GUI_LAB: Xvfb + picom + lavapipe, isolated HOME/XDG_RUNTIME_DIR/
#     CODEX_HOME/CODEX_RS_DATA_DIR, the pinned official CLI via
#     CODEX_RS_CODEX_BIN (the catalog §1 prerequisites);
#   - keyboard-only drives (xdotool key events; the pointer appears only in
#     the boot-render nudge and any scene-named documented exception);
#   - frame captures (ffmpeg x11grab) + md5 frame manifests;
#   - a named-moment action log (PROGRESS.txt) and a run.json lineage record
#     pinning the repo SHA (addendum §5 evidence schema).
#
# This file is a LIBRARY, not a runner: it is sourced by the scene scripts
# and exports the fv_* helpers below. Every scene script owns its own
# CALIBRATION header (the d26 law: every chord/anchor/copy string cited
# against the pinned base).
#
# Prerequisites this library hard-fails on (named):
#   tools   Xvfb, picom, xdotool, ffmpeg, xwininfo, md5sum, git
#   binary  the release-profile codexrs binary passed as $1 (executable)
#   runtime /home/z/parity-lab/runtime/codex (the pinned official CLI
#           0.146.0-alpha.3.1 — the platform-support oracle pin; restore it
#           from npm @openai/codex@0.146.0-alpha.3.1-linux-x64 if lost, the
#           wo-p2-008 README records the procedure)
#   donor   /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 — only for
#           scenes that call fv_need_donor first (the wo-p2-008 pattern)
#
# Scene scripts call, in order:
#   fv_need_runtime / fv_need_donor   (optional, before fv_begin)
#   fv_begin <scene-id> <journey-id> <binary> [outdir]
#   say / moment / key / type_ / cap / frame_md5
#   fv_end
#
# Sourcing guard: this library must be sourced exactly once per scene run.
if [ -n "${FV_LIB_SOURCED:-}" ]; then
  echo "FATAL: fv-lib.sh sourced twice" >&2
  exit 1
fi
FV_LIB_SOURCED=1

FV_HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FV_REPO_ROOT="$(cd "$FV_HERE/../.." && pwd)"
FV_RUNTIME="/home/z/parity-lab/runtime/codex"
FV_DONOR_GLOB="/tmp/d10-home.*/.local/share/codexRS/state.sqlite3"

FV_NEED_RUNTIME=0
FV_NEED_DONOR=0

# The pinned base this harness was authored and calibrated against (the
# dispatch pin; recorded in every run.json for lineage).
FV_PINNED_BASE="7f660c00407a5741eee975b2274570a200ff576b"

fv_fail() {
  echo "FATAL: $*" >&2
  # The action log may not exist yet (prerequisite failures fire before
  # fv_begin creates the evidence dir) — only append when it does.
  if [ -n "${FV_OUT:-}" ] && [ -d "$FV_OUT" ]; then
    echo "FATAL: $*" >> "$FV_OUT/PROGRESS.txt" || true
  fi
  exit 1
}

# Optional prologue: require the pinned official CLI runtime. All catalog
# Linux scenes assume the app-server path is live (the D11 EQ-1 lesson:
# without the runtime the footer stays "Connection failed" and task
# creation cannot run), so every scene calls this.
fv_need_runtime() {
  FV_NEED_RUNTIME=1
}

# Optional prologue: require + seed the donor state fixture (seeded chats —
# the wo-p2-008 pattern). fv_begin copies the donor state.sqlite3 into the
# scene's isolated CODEX_RS_DATA_DIR before the app launches.
fv_need_donor() {
  FV_NEED_DONOR=1
}

fv_check_prereqs() {
  local missing=()
  local tool
  for tool in Xvfb picom xdotool ffmpeg xwininfo md5sum; do
    command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
  done
  command -v git >/dev/null 2>&1 || missing+=("git")
  if [ "${#missing[@]}" -gt 0 ]; then
    fv_fail "prerequisite tools missing at this station: ${missing[*]} — install them (the warm Lead station has them under /home/z/parity-lab + desktop-tools) or run the scene from the Lead station"
  fi
  if [ ! -x "$FV_BIN" ]; then
    fv_fail "the pinned-SHA codexrs binary is missing or not executable: $FV_BIN — build the release profile at the pinned base first (cargo build --locked --release -p codex-app)"
  fi
  if [ "$FV_NEED_RUNTIME" = "1" ] && [ ! -x "$FV_RUNTIME" ]; then
    fv_fail "the pinned official CLI runtime is missing: $FV_RUNTIME (0.146.0-alpha.3.1, the platform-support oracle pin) — restore it from npm @openai/codex@0.146.0-alpha.3.1-linux-x64 (the wo-p2-008 README records the procedure)"
  fi
  if [ "$FV_NEED_DONOR" = "1" ]; then
    FV_DONOR="$(ls -1 $FV_DONOR_GLOB 2>/dev/null | head -1)"
    if [ -z "$FV_DONOR" ]; then
      fv_fail "the donor state fixture is missing (glob $FV_DONOR_GLOB) — produce the wo-p2-008 donor state at the Lead station (a state.sqlite3 with seeded chats) before running this scene"
    fi
  fi
}

fv_begin() {
  FV_SCENE_ID="${1:?scene id required}"
  FV_JOURNEY_ID="${2:?journey id required}"
  FV_BIN="${3:?binary path required (the release-profile codexrs at the pinned base)}"
  FV_OUT="${4:-$FV_REPO_ROOT/docs/research/evidence/fv-gate/linux/$FV_SCENE_ID}"

  fv_check_prereqs
  mkdir -p "$FV_OUT"

  # The run lineage record (addendum §5): the repo SHA this run executed at.
  FV_RUN_SHA="$(git -C "$FV_REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")"
  FV_RUN_ID="fv_$(date -u +%Y%m%dT%H%M%SZ)_$$"
  FV_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  FV_FRAME_MANIFEST="$FV_OUT/md5.txt"
  : > "$FV_FRAME_MANIFEST"

  say "scene $FV_SCENE_ID (journey $FV_JOURNEY_ID) run $FV_RUN_ID starting at $FV_STARTED_AT"
  say "pinned base (authored/calibrated): $FV_PINNED_BASE"
  say "executing SHA: $FV_RUN_SHA"
  say "binary: $FV_BIN"
  say "evidence dir: $FV_OUT"

  # Isolated per-scene data/home (the sealed session recipe).
  FV_DATA="$(mktemp -d /tmp/fv-${FV_SCENE_ID}-data.XXXXXX)"
  FV_HOME="$(mktemp -d /tmp/fv-${FV_SCENE_ID}-home.XXXXXX)"
  mkdir -p "$FV_DATA/codex-home" "$FV_HOME/xdg"
  if [ "$FV_NEED_DONOR" = "1" ]; then
    cp "$FV_DONOR" "$FV_DATA/state.sqlite3"
    say "donor state seeded: $FV_DONOR -> $FV_DATA/state.sqlite3"
  fi

  # The warm-station sysroot paths (d-series precedent). Absent paths are
  # harmless: the hard-failed tool checks above already named what matters.
  export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:${PATH}"
  export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

  # One display per scene run (the d-series :12x family; the scene id
  # hashes to a stable number so re-runs clean up their own display).
  FV_DISP=":$(( ( $(echo "$FV_SCENE_ID" | cksum | cut -d' ' -f1) % 20 ) + 120 ))"
  pkill -f "Xvfb $FV_DISP" 2>/dev/null || true
  sleep 1
  Xvfb "$FV_DISP" -screen 0 1440x900x24 -nolisten tcp -noreset >"$FV_OUT/xvfb.log" 2>&1 &
  FV_XPID=$!
  sleep 1.5
  DISPLAY="$FV_DISP" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" \
    picom --backend xrender --config /dev/null >"$FV_OUT/picom.log" 2>&1 &
  sleep 2
  pgrep -x picom >/dev/null || fv_fail "picom did not start on display $FV_DISP — check $FV_OUT/picom.log (the sealed LINUX_GUI_LAB recipe requires the compositor)"

  DISPLAY="$FV_DISP" LIBGL_ALWAYS_SOFTWARE=1 \
    VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
    HOME="$FV_HOME" XDG_RUNTIME_DIR="$FV_HOME/xdg" \
    CODEX_RS_DATA_DIR="$FV_DATA" \
    CODEX_HOME="$FV_DATA/codex-home" \
    CODEX_RS_CODEX_BIN="$FV_RUNTIME" \
    "$FV_BIN" >"$FV_OUT/app.log" 2>&1 &
  FV_APP_PID=$!
  say "app launched: pid $FV_APP_PID on display $FV_DISP (isolated HOME/CODEX_HOME/CODEX_RS_DATA_DIR)"

  # Boot-render wait (the d26 probe loop; the click is the pre-journey boot
  # nudge, not a scene drive — the only pointer events before the legs).
  FV_RENDERED=0
  local i
  for i in $(seq 1 30); do
    sleep 5
    kill -0 "$FV_APP_PID" 2>/dev/null || { say "WARN: app exited during boot wait"; break; }
    DISPLAY="$FV_DISP" xdotool mousemove 720 450 click 1 2>/dev/null || true
    cap_raw "$FV_OUT/_probe.png" 2>/dev/null || true
    local sz
    sz="$(stat -c%s "$FV_OUT/_probe.png" 2>/dev/null || echo 0)"
    if [ "${sz:-0}" -gt 30000 ]; then
      say "UI rendered after $((i*5))s (frame $sz bytes)"
      FV_RENDERED=1
      break
    fi
  done
  rm -f "$FV_OUT/_probe.png"
  if [ "$FV_RENDERED" != "1" ]; then
    say "WARN: no >30KB frame in 150s — running the scene anyway (honest bound; the Lead adjudicates the frames)"
  fi
  sleep 2

  # Window focus (the d26 refocus law: every key event must land in the
  # codexRS window, so refocus before each drive).
  FV_WID="$(xwininfo -display "$FV_DISP" -root -children 2>/dev/null | grep -oE '^     0x[0-9a-f]+ "codexRS"' | grep -oE '0x[0-9a-f]+' | head -1)"
  if [ -n "$FV_WID" ]; then
    DISPLAY="$FV_DISP" xdotool windowfocus --sync "$FV_WID" 2>/dev/null || true
    say "window focused: $FV_WID"
  else
    say "WARN: codexRS window not found on $FV_DISP — drives will rely on the focused root window"
  fi

  # D19 lesson: fresh-profile promo modal defensive dismissal (no-op when
  # absent) — a single Escape, NEVER a click, so the promo-modal legs stay
  # keyboard-only. Scenes that must observe the promo modal itself
  # (FV-L01's UX-003 one-Escape contract) set FV_SKIP_BOOT_DISMISS=1 and
  # own their own dismissal moment.
  if [ "${FV_SKIP_BOOT_DISMISS:-0}" != "1" ]; then
    key_raw Escape
    sleep 2
  fi
}

# ---- the drive helpers (keyboard-only; refocus before every event) ----

say() {
  echo "[$(date -u +%H:%M:%S)] $*" | tee -a "$FV_OUT/PROGRESS.txt"
}

moment() {
  say "--- [$1] $2"
}

refocus() {
  if [ -n "${FV_WID:-}" ]; then
    DISPLAY="$FV_DISP" xdotool windowfocus --sync "$FV_WID" 2>/dev/null || true
  fi
}

key() {
  refocus
  DISPLAY="$FV_DISP" xdotool key "$1"
}

key_raw() {
  DISPLAY="$FV_DISP" xdotool key "$1"
}

type_() {
  refocus
  DISPLAY="$FV_DISP" xdotool type --delay 60 "$1"
}

# Raw capture (no manifest) — used by the boot probe.
cap_raw() {
  ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i "$FV_DISP" -frames:v 1 "$1"
}

# Named-moment capture: file + md5 manifest line + PROGRESS echo.
cap() {
  local name="$1"
  local file="$FV_OUT/$name.png"
  cap_raw "$file" || say "WARN: capture failed for $name"
  if [ -f "$file" ]; then
    local digest
    digest="$(md5sum "$file" | awk '{print $1}')"
    echo "$name.png $digest" >> "$FV_FRAME_MANIFEST"
    say "frame $name $(stat -c%s "$file") bytes md5 $digest"
  fi
}

frame_md5() {
  md5sum "$FV_OUT/$1" 2>/dev/null | awk '{print $1}'
}

# ---- teardown + the run lineage record ----

fv_write_run_json() {
  cat > "$FV_OUT/run.json" <<EOF
{
  "v": 1,
  "kind": "flauz.fv.linux-scene.run",
  "scene_id": "$FV_SCENE_ID",
  "journey_id": "$FV_JOURNEY_ID",
  "run_id": "$FV_RUN_ID",
  "pinned_base": "$FV_PINNED_BASE",
  "sha": "$FV_RUN_SHA",
  "binary": "$FV_BIN",
  "runtime": "$FV_RUNTIME",
  "donor_state": $([ "$FV_NEED_DONOR" = "1" ] && echo "\"$FV_DONOR\"" || echo "null"),
  "display": "$FV_DISP",
  "started_at": "$FV_STARTED_AT",
  "finished_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "frame_manifest": "md5.txt",
  "action_log": "PROGRESS.txt",
  "app_log": "app.log"
}
EOF
  say "run.json lineage written (sha $FV_RUN_SHA)"
}

fv_end() {
  fv_write_run_json
  say "scene $FV_SCENE_ID COMPLETE — evidence in $FV_OUT"
  kill "${FV_APP_PID:-0}" 2>/dev/null || true
  pkill -P "${FV_APP_PID:-0}" 2>/dev/null || true
  kill "${FV_XPID:-0}" 2>/dev/null || true
  pkill -x picom 2>/dev/null || true
  rm -rf "${FV_DATA:-}" "${FV_HOME:-}" 2>/dev/null || true
}
