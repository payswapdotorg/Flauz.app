#!/bin/bash
# D13 — WO-P2-010 GUI verification (palette rows for the evidenced registry
# commands). Runs on the POST-MERGE binary (main with 009+010).
#
# Verifiable on the runtime-less entry surface (D10/D11b doctrine — the
# palette opens with Ctrl+P and queries resolve visibly):
#   01 entry baseline
#   02 Ctrl+P baseline (palette surface)
#   03 "toggle review"       -> row present
#   04 "fork"                -> row present ("Continue in new chat")
#   05 "copy deeplink"       -> row present
#   06 "approve request"     -> row ABSENT (no pending approval — guard)
#   07 "rename chat"         -> row present (guarded by selected chat; on the
#                               entry surface may be absent — record honestly)
#   08 "go to chat 1"        -> row present (guarded by existing chats)
#   09 "maximize side panel" -> row ABSENT (no maximizable panel on entry)
# State-guarded absences are POSITIVE evidence (the no-visible-dead-rows rule).
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d13}"
DISP=":106"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D13DATA=/tmp/d13-data
rm -rf "$D13DATA"; mkdir -p "$D13DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -z "$DONOR" ] && { echo "FATAL: no donor state"; exit 1; }
cp "$DONOR" "$D13DATA/state.sqlite3"
D13REPO=/tmp/d13-repo
rm -rf "$D13REPO"; mkdir -p "$D13REPO"
echo "d13 fixture" > "$D13REPO/README.md"
python3 - "$D13DATA/state.sqlite3" "$D13REPO" <<'PY2'
import sqlite3, sys, time
db, repo = sys.argv[1], sys.argv[2]
c = sqlite3.connect(db)
c.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?1, ?2, ?3, 0)",
    (repo.encode(), int(time.time()*1000), "d13 fixture repo"))
c.commit()
print("seeded recent_workspaces:", c.execute("SELECT name FROM recent_workspaces").fetchall())
PY2

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3

HOME_D=$(mktemp -d /tmp/d13-home.XXXXXX); mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$D13DATA" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }

for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early"; exit 1; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
  cap "$OUT/live.jpg" 2>/dev/null || true
done
cap "$OUT/01-entry.jpg"

sleep 12; key ctrl+k; sleep 1.5; cap "$OUT/02-palette.jpg"

query() { # query <label> <text> [esc]
  key ctrl+a 2>/dev/null; key BackSpace 4
  sleep 0.3
  type_ "$2"; sleep 1.2
  cap "$OUT/03-$1.jpg"
}
query toggle-review "toggle review"
query fork "fork"
query copy-deeplink "copy deeplink"
query copy-session "copy session id"
query copy-workdir "copy working directory"
query approve "approve request"
query decline "decline request"
query rename "rename chat"
query goto1 "go to chat 1"
query maximize "maximize side panel"
key Escape; sleep 0.8
cap "$OUT/99-after-escape.jpg"
echo "D13 frames captured: $(ls $OUT/*.jpg | wc -l)"
echo "APP_PID=$APP_PID DISP=$DISP"
pkill -P "$APP_PID" 2>/dev/null; kill "$APP_PID" 2>/dev/null
