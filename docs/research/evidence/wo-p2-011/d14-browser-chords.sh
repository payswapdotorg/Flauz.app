#!/bin/bash
# D14 — WO-P2-011 GUI verification (context-scoped browser chords), OFFLINE-HONEST
# scope (D12 doctrine): the browser-PANE-focused chord resolution (reload /
# force-reload / copy-URL with a loaded page, and the no-page honest guards)
# requires a live browser surface context that the runtime-less lab does not
# expose — recorded NOT RUN, covered by the three CI-green unit tests
# (browser_pane_chords_resolve_only_inside_browser_focus,
# browser_pane_copy_url_copies_page_urls_and_guides_otherwise,
# browser_reload_chords_report_honestly_without_a_loaded_page).
#
# What this scene DOES verify on the real binary:
#   01 entry baseline
#   02 Ctrl+R on entry            -> surface unchanged (no global hijack)
#   03 Ctrl+Shift+R on entry      -> surface unchanged (no global hijack)
#   04 Ctrl+P palette             -> opens (interceptor leaves other chords alone)
#   05 "copy working" query       -> copyWorkingDirectory row visible with its
#                                    Ctrl+Shift+C binding (registry meaning intact)
#   06 create chat via composer   -> chat workspace surface
#   07 Ctrl+Shift+C in workspace  -> "Copied working directory" status (POSITIVE
#                                    fall-through: the dual-meaning chord keeps
#                                    its registry meaning outside browser focus)
#   08 Ctrl+R in workspace        -> surface unchanged (no browser pane -> no reload)
#   09 Ctrl+Shift+R in workspace  -> surface unchanged
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d14}"
DISP=":107"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D14DATA=/tmp/d14-data
rm -rf "$D14DATA"; mkdir -p "$D14DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -z "$DONOR" ] && { echo "FATAL: no donor state"; exit 1; }
cp "$DONOR" "$D14DATA/state.sqlite3"
D14REPO=/tmp/d14-repo
rm -rf "$D14REPO"; mkdir -p "$D14REPO"
echo "d14 fixture" > "$D14REPO/README.md"
python3 - "$D14DATA/state.sqlite3" "$D14REPO" <<'PY2'
import sqlite3, sys, time
db, repo = sys.argv[1], sys.argv[2]
c = sqlite3.connect(db)
c.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?1, ?2, ?3, 0)",
    (repo.encode(), int(time.time()*1000), "d14 fixture repo"))
c.commit()
print("seeded recent_workspaces:", c.execute("SELECT name FROM recent_workspaces").fetchall())
PY2

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

HOME_D=$(mktemp -d /tmp/d14-home.XXXXXX); mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$D14DATA" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_text() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }

# wait for a rendered frame (D10 doctrine: >30KB frame)
rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 "$APP_PID" 2>/dev/null || { echo "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/frame-live.jpg" 2>/dev/null || true
  if [ -s "$OUT/frame-live.jpg" ] && [ "$(stat -c%s "$OUT/frame-live.jpg")" -gt 30000 ]; then
    rendered=1; break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [01] entry baseline"
sleep 3
cap "$OUT/d14-01-entry.png"; B1=$(md5 "$OUT/d14-01-entry.png")

echo "--- [02] Ctrl+R on entry (expect unchanged)"
cap "$OUT/d14-02a-pre.png"; P2=$(md5 "$OUT/d14-02a-pre.png")
key ctrl+r; sleep 2.5
cap "$OUT/d14-02b-post.png"; B2=$(md5 "$OUT/d14-02b-post.png")

echo "--- [03] Ctrl+Shift+R on entry (expect unchanged)"
key ctrl+shift+r; sleep 2.5
cap "$OUT/d14-03-post.png"; B3=$(md5 "$OUT/d14-03-post.png")

echo "--- [04] Ctrl+P palette (expect palette surface)"
key ctrl+p; sleep 2.5
cap "$OUT/d14-04-palette.png"; B4=$(md5 "$OUT/d14-04-palette.png")

echo "--- [05] palette query 'copy working' (row + Ctrl+Shift+C binding visible)"
type_text "copy working"; sleep 2
cap "$OUT/d14-05-palette-copywd.png"; B5=$(md5 "$OUT/d14-05-palette-copywd.png")
key Escape; sleep 1.5

echo "--- [06] create chat via composer (workspace surface)"
DISPLAY=$DISP xdotool mousemove 720 700 click 1 2>/dev/null || true
sleep 0.5
type_text "d14 chord fixture chat"
sleep 1.0
key Return
sleep 6
cap "$OUT/d14-06-workspace.png"; B6=$(md5 "$OUT/d14-06-workspace.png")

echo "--- [07] Ctrl+Shift+C in workspace (expect 'Copied working directory' status)"
sleep 2
key ctrl+shift+c; sleep 2.5
cap "$OUT/d14-07-copied-wd.png"; B7=$(md5 "$OUT/d14-07-copied-wd.png")

echo "--- [08] Ctrl+R in workspace (expect unchanged)"
sleep 3
cap "$OUT/d14-08a-pre.png"; P8=$(md5 "$OUT/d14-08a-pre.png")
key ctrl+r; sleep 2.5
cap "$OUT/d14-08b-post.png"; B8=$(md5 "$OUT/d14-08b-post.png")

echo "--- [09] Ctrl+Shift+R in workspace (expect unchanged)"
key ctrl+shift+r; sleep 2.5
cap "$OUT/d14-09-post.png"; B9=$(md5 "$OUT/d14-09-post.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d14-01-entry:          $B1"
  echo "d14-02a-pre:           $P2"
  echo "d14-02b-post(ctrl+R):  $B2   (hijack if != pre AND visible change)"
  echo "d14-03-post(c+s+r):    $B3   (vs 02b: expect unchanged entry)"
  echo "d14-04-palette:        $B4"
  echo "d14-05-copywd-row:     $B5"
  echo "d14-06-workspace:      $B6"
  echo "d14-07-copied-wd:      $B7   (VLM-read for 'Copied working directory')"
  echo "d14-08a-pre:           $P8"
  echo "d14-08b-post(ctrl+R):  $B8   (hijack if != pre AND visible change)"
  echo "d14-09-post(c+s+r):    $B9   (vs 08b: expect unchanged workspace)"
} > "$OUT/d14-md5.txt"
cat "$OUT/d14-md5.txt"
echo "--- app.log tail ---"
tail -8 "$OUT/app.log"
echo "D14 SCENE COMPLETE — VLM-read frames 04/05/06/07 for the palette + status evidence; md5-compare 02/03/08/09"
