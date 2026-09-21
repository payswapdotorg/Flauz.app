#!/bin/bash
# D19 (full) — WO-P2-019 pre-merge GUI regression scene. Self-contained:
# donor + browsing-history seed (the d12 recipe) + Browser-settings nav +
# the Clear-modal trap probes + background-Tab leg. See d19-phaseB.sh header
# for the full frame ladder and scope rationale.
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/d19}"
DISP=":110"
RUNTIME=/home/z/parity-lab/runtime/codex

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing: $RUNTIME"; exit 1; }

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

D19DATA=/tmp/d19-data
rm -rf "$D19DATA"; mkdir -p "$D19DATA" "$D19DATA/codex-home"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
if [ -z "$DONOR" ]; then echo "FATAL: no donor state.sqlite3"; exit 1; fi
cp "$DONOR" "$D19DATA/state.sqlite3"
D19REPO=/tmp/d19-repo
rm -rf "$D19REPO"; mkdir -p "$D19REPO"
echo "d19 fixture" > "$D19REPO/README.md"
python3 - "$D19DATA/state.sqlite3" "$D19REPO" <<'PY2'
import sqlite3, sys, time
db, repo = sys.argv[1], sys.argv[2]
c = sqlite3.connect(db)
c.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?1, ?2, ?3, 0)",
    (repo.encode(), int(time.time()*1000), "d19 fixture repo"))
c.commit()
print("seeded recent_workspaces:", c.execute("SELECT name FROM recent_workspaces").fetchall())
PY2

mkdir -p "$OUT"
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 60 "$1"; }
clickat() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

launch_app() {
  HOME_D=$(mktemp -d /tmp/d19-home.XXXXXX)
  mkdir -p "$HOME_D/xdg"
  echo "$HOME_D" > "$OUT/home_d"
  DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
    VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
    HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
    CODEX_RS_DATA_DIR="$D19DATA" \
    CODEX_HOME="$D19DATA/codex-home" \
    CODEX_RS_CODEX_BIN="$RUNTIME" \
    "$BIN" >"$OUT/app.log" 2>&1 &
  APP_PID=$!
  echo "$APP_PID" > "$OUT/app_pid"
}

wait_rendered() {
  for i in $(seq 1 30); do
    sleep 5
    kill -0 "$APP_PID" 2>/dev/null || { echo "app exited early"; return 1; }
    DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true
    cap "$OUT/_probe.png" 2>/dev/null || true
    SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
    [ "${SZ:-0}" -gt 30000 ] && return 0
  done
  return 1
}

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!; echo "$XPID" > "$OUT/xpid"
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

# --- phase A: first launch (schema migration), quit, seed browsing history
launch_app
if ! wait_rendered; then echo "PHASE A: app never rendered"; exit 1; fi
cap "$OUT/d19-01-entry.png"; echo "F01 $(md5 "$OUT/d19-01-entry.png")"
sleep 2
pkill -P "$APP_PID" 2>/dev/null; kill "$APP_PID" 2>/dev/null; sleep 3
python3 - "$D19DATA/state.sqlite3" <<'PY'
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
v = db.execute("PRAGMA user_version").fetchone()[0]
print("schema user_version =", v)
assert v == 5, "migration to v5 did not land"
rows = [
  ("https://example.com/", "Example Domain", 1700000000000),
  ("https://docs.rs/rust/", "Rust docs", 1700000100000),
  ("https://github.com/payswapdotorg/Flauz.app", "payswapdotorg/Flauz.app", 1700000200000),
  ("https://chat.z.ai/c/abc123", "Chat", 1700000300000),
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
sleep 3
WID=$(xwininfo -display $DISP -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 1
key ctrl+k; sleep 2
type_ "browser settings"; sleep 2
key Return; sleep 2.5
cap "$OUT/d19-02-settings-browser.png"; echo "F02 $(md5 "$OUT/d19-02-settings-browser.png")"
sleep 1.5
cap "$OUT/d19-03-settings-settled.png"; echo "F03 $(md5 "$OUT/d19-03-settings-settled.png")"

# --- phase C: Clear modal + trap probes (Clear button center ~757,652)
echo "--- [04] open the Clear modal (click Clear…)"
clickat 757 652; sleep 2
cap "$OUT/d19-04-modal-open.png"; echo "F04 $(md5 "$OUT/d19-04-modal-open.png")"

echo "--- [05] Tab #1 (focus to the other button)"
key Tab; sleep 1.2
cap "$OUT/d19-05-tab1.png"; echo "F05 $(md5 "$OUT/d19-05-tab1.png")"

echo "--- [06] Tab #2 (WRAP — the trap proof)"
key Tab; sleep 1.2
cap "$OUT/d19-06-tab2-wrap.png"; echo "F06 $(md5 "$OUT/d19-06-tab2-wrap.png")"

echo "--- [07] Tab #3 (alternate again)"
key Tab; sleep 1.2
cap "$OUT/d19-07-tab3.png"; echo "F07 $(md5 "$OUT/d19-07-tab3.png")"

echo "--- [08] Shift+Tab (backward wrap)"
key shift+Tab; sleep 1.2
cap "$OUT/d19-08-shifttab.png"; echo "F08 $(md5 "$OUT/d19-08-shifttab.png")"

echo "--- [09] Escape closes the modal"
key Escape; sleep 1.5
cap "$OUT/d19-09-modal-closed.png"; echo "F09 $(md5 "$OUT/d19-09-modal-closed.png")"

# --- LEG B: background Tab outside any modal
echo "--- [10] leave settings (escape ladder)"
key Escape; sleep 1.5
key Escape; sleep 1.5
cap "$OUT/d19-10-main-surface.png"; echo "F10 $(md5 "$OUT/d19-10-main-surface.png")"

echo "--- [11] composer focus (ladder)"
for Y in 849 829 809 789; do DISPLAY=$DISP xdotool mousemove 802 $Y click 1; sleep 0.3; done
sleep 1
cap "$OUT/d19-11-composer-focus.png"; echo "F11 $(md5 "$OUT/d19-11-composer-focus.png")"

echo "--- [12] background Tab #1 (focus advances, not captured)"
key Tab; sleep 1.2
cap "$OUT/d19-12-bg-tab1.png"; echo "F12 $(md5 "$OUT/d19-12-bg-tab1.png")"

echo "--- [13] background Tab #2"
key Tab; sleep 1.2
cap "$OUT/d19-13-bg-tab2.png"; echo "F13 $(md5 "$OUT/d19-13-bg-tab2.png")"

echo "--- teardown"
kill $APP_PID 2>/dev/null; pkill -P $APP_PID 2>/dev/null
sleep 1
pkill -x picom 2>/dev/null
kill $XPID 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
echo "d19 done (artifacts in $OUT)"
