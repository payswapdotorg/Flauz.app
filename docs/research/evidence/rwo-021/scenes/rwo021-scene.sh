#!/bin/bash
# RWO-021 — LINUX_GUI_LAB scene (sealed recipe: Xvfb + picom xrender + software GL
# + isolated HOME/XDG/CODEX_RS_DATA_DIR + ffmpeg x11grab + xdotool).
# Phase 0: fixture repo + seeded state.sqlite3 (recent_workspaces restore path, D10b pattern)
# Phase 1: workspace-seeded boot + rubric probes (KAX-C1, KSR-C1/C2/C4, SET-C1/C2/C3,
#          FBK-C5, PRJ-C7, NTR-C5, ACT-C3/D11b four keypresses, BOOT-C5 liveness)
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/rwo021-evidence}"
DISP=":104"
PREFIX=/home/z/sysroot/prefix

export PATH="$PREFIX/usr/bin:$PATH"
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:$PREFIX/usr/lib/llvm-19/lib:${LD_LIBRARY_PATH:-}"

mkdir -p "$OUT"
rm -rf "$OUT"/*.png "$OUT"/*.txt "$OUT"/*.log 2>/dev/null

# ---------- Phase 0a: fixture workspace ----------
FIXTURE=/tmp/rwo021-fixture
rm -rf "$FIXTURE"; mkdir -p "$FIXTURE/src" "$FIXTURE/docs"
echo "# fixture project" > "$FIXTURE/README.md"
echo "fn main() {}" > "$FIXTURE/src/main.rs"
echo "notes" > "$FIXTURE/docs/notes.md"
echo "pub fn helper() {}" > "$FIXTURE/src/lib.rs"

# ---------- Phase 0b: seed state.sqlite3 (boot #1 creates+migrates, then insert row) ----------
DATA=/tmp/rwo021-data
rm -rf "$DATA"; mkdir -p "$DATA"
SEED_HOME=$(mktemp -d /tmp/rwo021-seedhome.XXXXXX)
mkdir -p "$SEED_HOME/xdg"

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb $DISP -screen 0 1600x1000x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 3
pgrep -x picom >/dev/null || echo "WARN: picom not up"

DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$SEED_HOME" XDG_RUNTIME_DIR="$SEED_HOME/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/seedboot.log" 2>&1 &
SEED_PID=$!
sleep 30
kill -0 $SEED_PID 2>/dev/null && echo "seed boot alive after 30s (no fatal, BOOT-C5 seed leg)" || echo "WARN: seed boot exited early"
kill $SEED_PID 2>/dev/null; sleep 2

python3 - "$DATA/state.sqlite3" "$FIXTURE" <<'EOF'
import sqlite3, sys, time
db, fixture = sys.argv[1], sys.argv[2]
con = sqlite3.connect(db)
tables = [r[0] for r in con.execute("SELECT name FROM sqlite_master WHERE type='table'")]
assert "recent_workspaces" in tables, f"recent_workspaces missing; tables={tables}"
con.execute("INSERT INTO recent_workspaces(path, last_opened_at, name, pinned) VALUES (?,?,?,0)",
            (fixture.encode(), int(time.time()*1000), "fixture"))
con.commit()
rows = con.execute("SELECT path, name FROM recent_workspaces").fetchall()
print("seeded recent_workspaces:", rows)
con.close()
EOF
[ $? -eq 0 ] || { echo "FATAL: seeding failed"; exit 1; }

# ---------- Phase 1: scene boot ----------
HOME_D=$(mktemp -d /tmp/rwo021-home.XXXXXX)
mkdir -p "$HOME_D/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="$PREFIX/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_D" XDG_RUNTIME_DIR="$HOME_D/xdg" \
  CODEX_RS_DATA_DIR="$DATA" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1600x1000 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 55 "$1"; }
click() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

rendered=0
for i in $(seq 1 30); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { echo "app exited early (see app.log)"; break; }
  click 800 500 2>/dev/null || true
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  if [ "${SZ:-0}" -gt 30000 ]; then
    echo "UI rendered after $((i*5))s (frame $SZ bytes)"
    rendered=1
    break
  fi
done
[ "$rendered" = "1" ] || echo "WARN — no >30KB frame in 150s; running scene anyway"

WID=$(DISPLAY=$DISP xwininfo -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+")
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
sleep 2

echo "--- [a01] workspace baseline"
cap "$OUT/a01-ws-baseline.png"
echo "--- [a02] Ctrl+P -> Search files palette (KAX-C1/D10b)"
sleep 1; key ctrl+p; sleep 2.5; cap "$OUT/a02-ctrl-p-palette.png"
echo "--- [a03] Escape closes byte-identical (KAX-C1)"
sleep 1; key Escape; sleep 2.5; cap "$OUT/a03-ctrl-p-escape.png"
echo "--- [a04] Ctrl+/ shortcuts overlay (KSR-C1/C2)"
sleep 1; key ctrl+slash; sleep 3; cap "$OUT/a04-ctrl-slash-overlay.png"
sleep 1; key Escape; sleep 1.5
echo "--- [a05] Ctrl+Shift+P palette (KAX-C1)"
key ctrl+shift+p; sleep 2.5; cap "$OUT/a05-ctrl-shift-p.png"
sleep 1; key Escape; sleep 1.5
echo "--- [a06] Ctrl+G chats palette (PRJ-C7)"
key ctrl+g; sleep 2.5; cap "$OUT/a06-ctrl-g.png"
sleep 1; key Escape; sleep 1.5
echo "--- [a07] Ctrl+K -> feedback -> Share feedback dialog (FBK-C5 palette leg)"
key ctrl+k; sleep 1.5; type_ "feedback"; sleep 1.2; cap "$OUT/a07-palette-feedback-query.png"; key Return; sleep 2.5; cap "$OUT/a08-feedback-dialog.png"
sleep 1; key Escape; sleep 1.5
echo "--- [a09] composer typed /feedback (FBK-C5 slash leg)"
click 800 900; sleep 0.8; type_ "/feedback"; sleep 1.2; cap "$OUT/a09-composer-feedback-typed.png"; key Return; sleep 2.5; cap "$OUT/a10-feedback-dialog-slash.png"
sleep 1; key Escape; sleep 1.5
echo "--- [a11] Ctrl+, settings shell (SET-C1/C2)"
key ctrl+comma; sleep 3; cap "$OUT/a11-settings-shell.png"
echo "--- [a12] settings search 'import' (SET-C3)"
click 573 165; sleep 0.6; type_ "import"; sleep 1.8; cap "$OUT/a12-settings-search-import.png"
echo "--- [a13] clear; palette -> keyboard shortcuts settings page (KSR-C4)"
key ctrl+a; key Delete; sleep 0.8; key ctrl+k; sleep 1.5; type_ "keyboard"; sleep 1.2; key Return; sleep 2.5; cap "$OUT/a13-settings-kbshortcuts.png"
echo "--- [a14] bounded query 'search' in shortcuts page (KSR-C4)"
sleep 0.5; type_ "search"; sleep 1.8; cap "$OUT/a14-kbshortcuts-filtered.png"
echo "--- [a15] sidebar: Pull requests page (NTR-C5 gh banner)"
key Escape; sleep 0.6; click 300 222; sleep 3; cap "$OUT/a15-sidebar-pullrequests.png"
echo "--- [a16] sidebar: Plugins page (NTR-C5)"
click 300 258; sleep 3; cap "$OUT/a16-sidebar-plugins.png"
echo "--- [a17] D11b four keypresses (ACT-C3)"
click 800 500; sleep 1
key ctrl+shift+u; sleep 2.5; cap "$OUT/a17-d11b-mark-unread.png"
key ctrl+alt+a;   sleep 2.5; cap "$OUT/a18-d11b-next-unread.png"
key ctrl+alt+u;   sleep 2.5; cap "$OUT/a19-d11b-activity-view.png"
key shift+Escape; sleep 2.5; cap "$OUT/a20-d11b-clear-all.png"

kill -0 $APP_PID 2>/dev/null && echo "APP ALIVE AT SCENE END (BOOT-C5: no fatal under no-runtime)" || echo "APP EXITED BEFORE SCENE END"
kill $APP_PID 2>/dev/null; sleep 1; pkill -f "Xvfb $DISP" 2>/dev/null

for f in "$OUT"/a*.png; do echo "$(basename "$f"): $(md5 "$f")"; done > "$OUT/scene-md5.txt"
cat "$OUT/scene-md5.txt"
echo "--- app.log tail ---"
tail -8 "$OUT/app.log"
echo "RWO021_SCENE_COMPLETE"
