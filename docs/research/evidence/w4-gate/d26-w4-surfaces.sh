#!/bin/bash
# D26 — Wave-4 Gate B: the provider-accounts surface (PROV-001, J-14's
# account/tier attribution dimension), the members + presence surface
# (COL-001, J-13), and the per-task sharing surface (COL-001) at the
# merged Wave-4 binary (526d53c code head, staged at f761252).
#
# CALIBRATION (2026-09-23, verified against the merged source at 526d53c):
#   Providers (PROV-001): Ctrl+Alt+Shift+P chord (letter family, verified
#     conflict-free). Panel heading "Your provider accounts"; honest empty
#     title "No provider accounts connected yet" + what connecting adds;
#     the connect form ("Connect a provider account" + OpenAI + Account
#     name + API key + "Connect account" + "The key is stored securely;
#     Flauz never shows it again."); the routing-policy surface "Which
#     account Flauz uses" + "Order today: free accounts first" + "Change
#     the order"; the limits caption "Spending and task limits" + the
#     honest no-limits line. Palette rows "Connect a provider account" /
#     "See which account a task uses". The task-surface entry label
#     "Provider accounts" with "None connected yet". Scoped Escape
#     ("FlauzProviders").
#   Members (COL-001): Ctrl+Alt+Shift+U chord (the verified-free letter —
#     the work order's suggested M is owned by the model picker). Panel
#     heading "Who is on this workspace"; the honest solo state "Just you"
#     + what inviting adds; the not-wired invite/role states ("Inviting
#     isn't connected yet" / "Changing roles isn't connected yet" — say so
#     plainly); "Invite someone". Palette rows "See who is on this
#     workspace" / "Change a task's sharing". Scoped Escape
#     ("FlauzMembers").
#   Sharing (COL-001): the per-task sharing surface — heading "Sharing";
#     "Files" + "Isolated copy — your files stay separate"; "Remembered
#     notes" + "Only me — remembered notes stay visible only you"; "Work
#     products"; "Change sharing". With no chat selected the honest
#     guidance "Open a chat to see its sharing."
#   The denial-with-why ("You can't change this sharing — your role here
#     is …") is F10+ wiring (pinned by the module's copy tests + the
#     why_not_for_role seam; the GUI renders the owner's can-change
#     posture at cold start — nothing invented). The palette is Ctrl+K
#     (d20-proven); the anchor task is Ctrl+N + type + ctrl+Return
#     (d23/d24/d25-proven).
#
#   01 entry baseline (cold start)
#   02 anchor task — the live task surface (the providers/sharing
#      affordances ride the task context)
#   03 J-14 Ctrl+Alt+Shift+P — the providers panel (empty + connect form
#      + policy + limits)
#   04 settle/detail frame
#   05 scoped Escape — the panel closes; the task surface stands
#   06 palette "Connect a provider account" + Return lands (B06b)
#   07 palette "See which account a task uses" + Return lands (B07b: the
#      honest not-set-up guidance + the panel)
#   08 J-13 Ctrl+Alt+Shift+U — the members panel (Just you + the
#      not-wired honesty)
#   09 settle/detail frame
#   10 scoped Escape — the panel closes
#   11 palette "See who is on this workspace" + Return lands (B11b)
#   12 palette "Change a task's sharing" + Return lands (B12b: the
#      sharing panel with the consequence lines)
#   13 settle frame
#   14 Escape settle — final frame
#
# PASS: VLM reads show (a) the providers panel at 03 with "Your provider
# accounts" + "No provider accounts connected yet" + the connect form +
# "Which account Flauz uses"/"free accounts first", (b) the panel closed
# at 05, (c) the panel back at 06b, (d) the honest not-set-up guidance at
# 07b, (e) the members panel at 08 with "Who is on this workspace" +
# "Just you" + "Inviting isn't connected yet", (f) the members panel back
# at 11b, (g) the sharing panel at 12b with "Sharing" + "Isolated copy —
# your files stay separate" + the notes consequence line. PASS/FAIL is
# adjudicated by the Lead from the frames + VLM reads.
# Usage: d26-w4-surfaces.sh <binary-path> [outdir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d26}"
DISP=":126"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D26DATA=/tmp/d26-data
rm -rf "$D26DATA"; mkdir -p "$D26DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D26DATA/state.sqlite3"
mkdir -p "$D26DATA/codex-home"

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D26=$(mktemp -d /tmp/d26-home.XXXXXX)
mkdir -p "$HOME_D26/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D26" XDG_RUNTIME_DIR="$HOME_D26/xdg" \
  CODEX_RS_DATA_DIR="$D26DATA" \
  CODEX_HOME="$D26DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
refocus() {
  W=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
  [ -n "$W" ] && DISPLAY=$DISP xdotool windowfocus --sync "$W" 2>/dev/null || true
}
key() { refocus; DISPLAY=$DISP xdotool key "$1"; }
type_text() { refocus; DISPLAY=$DISP xdotool type --delay 60 "$1"; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  if [ "${SZ:-0}" -gt 30000 ]; then
    echo "UI rendered after $((i*5))s (frame $SZ bytes)"
    rendered=1
    break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"
sleep 5

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

# D19 lesson: fresh-profile promo modal defensive dismissal (no-op absent).
DISPLAY=$DISP xdotool mousemove 580 619 click 1
sleep 2
DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
sleep 1

echo "--- [01] entry baseline (cold start)"
key Escape; sleep 2
cap "$OUT/d26-01-entry.png"; echo "B01 $(md5 "$OUT/d26-01-entry.png")"

echo "--- [02] anchor task — the live task surface"
key ctrl+n; sleep 2.5
type_text "Wave 4 gate anchor task"; sleep 1
key ctrl+Return; sleep 4
cap "$OUT/d26-02-task-surface.png"; echo "B02 $(md5 "$OUT/d26-02-task-surface.png")"

echo "--- [03] J-14 Ctrl+Alt+Shift+P — the providers panel (empty + connect + policy + limits)"
key ctrl+alt+shift+p; sleep 2.5
cap "$OUT/d26-03-providers-open.png"; echo "B03 $(md5 "$OUT/d26-03-providers-open.png")"
sleep 1.5
cap "$OUT/d26-04-providers-detail.png"; echo "B04 $(md5 "$OUT/d26-04-providers-detail.png")"

echo "--- [05] scoped Escape — the panel closes; the task surface stands"
key Escape; sleep 2
cap "$OUT/d26-05-providers-closed.png"; echo "B05 $(md5 "$OUT/d26-05-providers-closed.png")"

echo "--- [06] palette: 'Connect a provider account' + Return lands"
key ctrl+k; sleep 2
type_text "Connect a provider account"; sleep 2
cap "$OUT/d26-06-palette-connect.png"; echo "B06 $(md5 "$OUT/d26-06-palette-connect.png")"
key Return; sleep 2.5
cap "$OUT/d26-06b-palette-connect-landed.png"; echo "B06b $(md5 "$OUT/d26-06b-palette-connect-landed.png")"
key Escape; sleep 1.5

echo "--- [07] palette: 'See which account a task uses' + Return lands"
key ctrl+k; sleep 2
type_text "See which account a task uses"; sleep 2
cap "$OUT/d26-07-palette-see.png"; echo "B07 $(md5 "$OUT/d26-07-palette-see.png")"
key Return; sleep 2.5
cap "$OUT/d26-07b-palette-see-landed.png"; echo "B07b $(md5 "$OUT/d26-07b-palette-see-landed.png")"
key Escape; sleep 1.5

echo "--- [08] J-13 Ctrl+Alt+Shift+U — the members panel (Just you + not-wired honesty)"
key ctrl+alt+shift+u; sleep 2.5
cap "$OUT/d26-08-members-open.png"; echo "B08 $(md5 "$OUT/d26-08-members-open.png")"
sleep 1.5
cap "$OUT/d26-09-members-detail.png"; echo "B09 $(md5 "$OUT/d26-09-members-detail.png")"

echo "--- [10] scoped Escape — the members panel closes"
key Escape; sleep 2
cap "$OUT/d26-10-members-closed.png"; echo "B10 $(md5 "$OUT/d26-10-members-closed.png")"

echo "--- [11] palette: 'See who is on this workspace' + Return lands"
key ctrl+k; sleep 2
type_text "See who is on this workspace"; sleep 2
cap "$OUT/d26-11-palette-members.png"; echo "B11 $(md5 "$OUT/d26-11-palette-members.png")"
key Return; sleep 2.5
cap "$OUT/d26-11b-palette-members-landed.png"; echo "B11b $(md5 "$OUT/d26-11b-palette-members-landed.png")"
key Escape; sleep 1.5

echo "--- [12] palette: 'Change a task's sharing' + Return lands (the sharing panel)"
key ctrl+k; sleep 2
type_text "Change a task"; sleep 2
cap "$OUT/d26-12-palette-sharing.png"; echo "B12 $(md5 "$OUT/d26-12-palette-sharing.png")"
key Return; sleep 2.5
cap "$OUT/d26-12b-palette-sharing-landed.png"; echo "B12b $(md5 "$OUT/d26-12b-palette-sharing-landed.png")"
sleep 1.5
cap "$OUT/d26-13-sharing-detail.png"; echo "B13 $(md5 "$OUT/d26-13-sharing-detail.png")"

echo "--- [14] Escape settle — final frame"
key Escape; sleep 2
cap "$OUT/d26-14-settle.png"; echo "B14 $(md5 "$OUT/d26-14-settle.png")"

kill $APP_PID 2>/dev/null; kill $XPID 2>/dev/null; pkill -f "picom" 2>/dev/null
echo "D26 SCENE COMPLETE — evidence in $OUT"
