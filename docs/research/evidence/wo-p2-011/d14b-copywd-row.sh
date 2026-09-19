#!/bin/bash
# D14b — WO-P2-011 follow-up: the copyWorkingDirectory registry evidence.
#   1) create chat via composer (workspace surface, same as D14-06)
#   2) Ctrl+K command palette -> "copy working directory" query, 3s settle
#      -> the row + its Ctrl+Shift+C binding (the dual-meaning chord's
#         registry meaning, visible)
#   3) Escape -> settle -> pre-chord frame
#   4) Ctrl+Shift+C -> capture at ~1.2s and ~2.5s (catch a transient toast)
# Honest outcome either way: toast render needs the command ENABLED (a
# cwd-bearing chat); a silent fall-through with the row+binding visible is
# the honest-negative record (unit test carries the enabled-path semantics).
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d14}"
DISP=":108"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D14DATA=/tmp/d14b-data
rm -rf "$D14DATA"; mkdir -p "$D14DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -z "$DONOR" ] && { echo "FATAL: no donor state"; exit 1; }
cp "$DONOR" "$D14DATA/state.sqlite3"
D14REPO=/tmp/d14b-repo
rm -rf "$D14REPO"; mkdir -p "$D14REPO"
echo "d14b fixture" > "$D14REPO/README.md"
python3 - "$D14DATA/state.sqlite3" "$D14REPO" <<'PY2'
import sqlite3, sys, time
db, repo = sys.argv[1], sys.argv[2]
c = sqlite3.connect(db)
c.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?1, ?2, ?3, 0)",
    (repo.encode(), int(time.time()*1000), "d14b fixture repo"))
c.commit()
print("seeded recent_workspaces:", c.execute("SELECT name FROM recent_workspaces").fetchall())
PY2

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb-d14b.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom-d14b.log" 2>&1 &
sleep 3

HOME_D=$(mktemp -d /tmp/d14b-home.XXXXXX); mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$D14DATA" \
  "$BIN" >"$OUT/app-d14b.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_text() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 "$APP_PID" 2>/dev/null || { echo "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/frame-live-b.jpg" 2>/dev/null || true
  if [ -s "$OUT/frame-live-b.jpg" ] && [ "$(stat -c%s "$OUT/frame-live-b.jpg")" -gt 30000 ]; then
    rendered=1; break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"

WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1

echo "--- [b1] create chat via composer"
DISPLAY=$DISP xdotool mousemove 720 700 click 1 2>/dev/null || true
sleep 0.5
type_text "d14b copywd fixture chat"
sleep 1.0
key Return
sleep 6
cap "$OUT/d14b-01-workspace.png"; B1=$(md5 "$OUT/d14b-01-workspace.png")

echo "--- [b2] Ctrl+K command palette, query 'copy working directory' (3s settle)"
key ctrl+k; sleep 1.5
type_text "copy working directory"; sleep 3
cap "$OUT/d14b-02-palette-copywd-row.png"; B2=$(md5 "$OUT/d14b-02-palette-copywd-row.png")

echo "--- [b3] Escape, settle, pre-chord frame"
key Escape; sleep 2
cap "$OUT/d14b-03-pre-chord.png"; B3=$(md5 "$OUT/d14b-03-pre-chord.png")

echo "--- [b4] Ctrl+Shift+C, capture at 1.2s"
key ctrl+shift+c; sleep 1.2
cap "$OUT/d14b-04-post-early.png"; B4=$(md5 "$OUT/d14b-04-post-early.png")

echo "--- [b5] capture at 2.5s (toast may persist)"
sleep 1.3
cap "$OUT/d14b-05-post-late.png"; B5=$(md5 "$OUT/d14b-05-post-late.png")

kill $APP_PID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null

{
  echo "d14b-01-workspace:        $B1"
  echo "d14b-02-palette-row:      $B2   (VLM-read: row + Ctrl+Shift+C binding)"
  echo "d14b-03-pre-chord:        $B3"
  echo "d14b-04-post-early(1.2s): $B4   (delta vs B3 = toast?)"
  echo "d14b-05-post-late(2.5s):  $B5"
} > "$OUT/d14b-md5.txt"
cat "$OUT/d14b-md5.txt"
echo "--- app-d14b.log tail ---"
tail -5 "$OUT/app-d14b.log"
echo "D14B SCENE COMPLETE — VLM-read b2 (row+binding) and b4/b5 (toast delta)"
