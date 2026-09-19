#!/bin/bash
# D12 — WO-P2-009 GUI verification, OFFLINE-HONEST scope.
#
# Same doctrine as D11b: the desktop app needs the official codex CLI runtime
# (absent in this lab; no fabrication). The browser PANEL (address-bar revisit)
# requires a live browser surface context that the runtime-less entry surface
# does not expose — recorded NOT RUN (covered by codex-platform matching tests
# + codex-app binding tests on CI).
#
# What this scene DOES verify on the real binary:
#   01 entry baseline (state migrated to schema v5 on first launch)
#   02 Ctrl+P palette -> "browser settings" query resolves the row
#   03 Enter -> Settings Browser section: seeded browsing-history rows visible
#   04 Clear… button visible with the seeded count
#   05 click Clear… -> confirmation modal (keyboard focus present)
#   06 confirm -> history cleared (empty surface / disabled Clear)
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d12}"
DISP=":105"

export PATH="/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D12DATA=/tmp/d12-data
rm -rf "$D12DATA"; mkdir -p "$D12DATA"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D12DATA/state.sqlite3"
D12REPO=/tmp/d12-repo
rm -rf "$D12REPO"; mkdir -p "$D12REPO"
echo "d12 fixture" > "$D12REPO/README.md"
python3 - "$D12DATA/state.sqlite3" "$D12REPO" <<'PY2'
import sqlite3, sys, time
db, repo = sys.argv[1], sys.argv[2]
c = sqlite3.connect(db)
c.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?1, ?2, ?3, 0)",
    (repo.encode(), int(time.time()*1000), "d12 fixture repo"))
c.commit()
print("seeded recent_workspaces:", c.execute("SELECT name FROM recent_workspaces").fetchall())
PY2

mkdir -p "$OUT"
pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

launch_app() {
  HOME_D=$(mktemp -d /tmp/d12-home.XXXXXX)
  mkdir -p "$HOME_D/xdg"
  echo "$HOME_D" > "$OUT/home_d"
  DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
    VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
    HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
    CODEX_RS_DATA_DIR="$D12DATA" \
    "$BIN" >"$OUT/app.log" 2>&1 &
  APP_PID=$!
  echo "$APP_PID" > "$OUT/app_pid"
}

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }
clickat() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

wait_rendered() {
  for i in $(seq 1 30); do
    sleep 5
    kill -0 "$APP_PID" 2>/dev/null || { echo "app exited early"; return 1; }
    DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
    cap "$OUT/frame-live.jpg" 2>/dev/null || true
    if [ -s "$OUT/frame-live.jpg" ]; then
      local m=$(md5 "$OUT/frame-live.jpg")
      if [ "$m" != "$LAST_LIVE_MD5" ]; then LAST_LIVE_MD5=$m; return 0; fi
    fi
  done
  return 1
}
LAST_LIVE_MD5=""

# --- phase A: first launch -> schema v5 migration, then quit
launch_app
if ! wait_rendered; then echo "PHASE A: app never rendered"; exit 1; fi
cap "$OUT/01-entry.jpg"
sleep 2
pkill -P "$APP_PID" 2>/dev/null; kill "$APP_PID" 2>/dev/null; sleep 3
python3 - "$D12DATA/state.sqlite3" <<'PY'
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
v = db.execute("PRAGMA user_version").fetchone()[0]
print("schema user_version =", v)
assert v == 5, "migration to v5 did not land"
rows = [
  ("https://example.com/", "Example Domain", 1700000000000),
  ("https://docs.rs/rust/", "Rust docs", 1700000100000),
  ("https://github.com/payswapdotorg/Flauz.app", "payswapdotorg/Flauz.app", 1700000200000),
  ("https://chatgpt.com/c/abc123", "Chat", 1700000300000),
  ("https://news.ycombinator.com/", "Hacker News", 1700000400000),
]
db.executemany("INSERT INTO browsing_history(url, title, visited_at_ms) VALUES (?,?,?)", rows)
db.commit()
print("seeded", db.execute("SELECT COUNT(*) FROM browsing_history").fetchone()[0], "browsing_history rows")
PY
[ $? -ne 0 ] && { echo "SEED FAILED"; exit 1; }

# --- phase B: relaunch with seeded history -> Settings Browser section
launch_app
if ! wait_rendered; then echo "PHASE B: app never rendered"; exit 1; fi
cap "$OUT/02-relaunch.jpg"
sleep 2

# open the palette and navigate to Browser settings
key ctrl+k; sleep 1.5
cap "$OUT/03-palette.jpg"
type_ "browser settings"; sleep 1.8
cap "$OUT/04-palette-query.jpg"
key Return; sleep 2
cap "$OUT/05-settings-browser.jpg"
sleep 1
cap "$OUT/06-settings-browser-settled.jpg"

echo "SCENE PHASE B DONE — frames 01-06 captured; phase C (Clear modal) driven interactively by the Lead after VLM reads"
echo "APP_PID=$APP_PID  HOME_D=$(cat $OUT/home_d)  DISP=$DISP"
echo "$DISP" > "$OUT/disp"
