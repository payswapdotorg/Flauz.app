#!/bin/bash
# rg-a11y-01.sh — WO-UX-001 scene RG-A11Y-01: keyboard-only full journey
# across the six F1 surfaces (palette -> settings -> terminal -> browser ->
# history -> attention), NO POINTER INPUT after window placement.
#
# Per recommendations.md WO-UX-001 + accessibility-inventory §source-
# unverifiable notes. Every leg: keyboard chords only; captures + md5s;
# VLM adjudication. (Baseline run at the current tree; the binding F1 pass
# re-runs this scene at the final RC SHA post 017/019 merges.)
#
# Legs:
#   00 entry baseline
#   1  PALETTE    Ctrl+K open -> filter "settings" -> Escape close
#   2  SETTINGS   Ctrl+, open -> Ctrl+F search "appearance" -> Escape return
#   3  TERMINAL   Ctrl+` open -> type echo (PTY receives) -> close
#   4  BROWSER    Ctrl+Shift+B toggle -> Ctrl+L address focus -> Escape -> close
#   5  HISTORY    Ctrl+, -> Ctrl+F "browsing" -> nav filters (Browser section)
#   6  ATTENTION  keyboard chat create -> Ctrl+Shift+U -> Ctrl+Alt+A -> Shift+Esc
#   7  CHAT SWAP  Ctrl+Shift+] / Ctrl+Shift+[
#   8  OVERLAY    Ctrl+/ open -> Escape ladder close
#
# Usage: rg-a11y-01.sh <binary-path> [evidir]
set -u
BIN="${1:?binary path required}"
OUT="${2:-/home/z/parity-lab/evidence/battery-rg/a11y-01}"
DISP=":124"
RUNTIME=/home/z/parity-lab/runtime/codex

export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/bin/../lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

[ -x "$RUNTIME" ] || { echo "FATAL: pinned runtime missing"; exit 1; }
mkdir -p "$OUT"

cleanup() {
  kill "${APP_PID:-0}" 2>/dev/null; pkill -P "${APP_PID:-0}" 2>/dev/null
  pkill -x picom 2>/dev/null; pkill -f "Xvfb $DISP" 2>/dev/null
}
trap cleanup EXIT
say() { echo "[$(date -u +%H:%M:%S)] $*" | tee -a "$OUT/PROGRESS.txt"; }
cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i "$DISP" -frames:v 1 "$1" 2>/dev/null; }
md5() { md5sum "$1" 2>/dev/null | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
type_() { DISPLAY=$DISP xdotool type --delay 100 "$1"; }

# state: donor + a seeded long-history chat (for the attention/switch legs)
DATA=/tmp/rg-a11y-data
rm -rf "$DATA"; mkdir -p "$DATA/codex-home"
DONOR=$(ls -1 /tmp/d10-home.*/.local/share/codexRS/state.sqlite3 2>/dev/null | head -1)
[ -n "$DONOR" ] || { echo "FATAL: no donor"; exit 1; }
cp "$DONOR" "$DATA/state.sqlite3"
python3 - "$DATA" <<'EOF'
import json, os, sys, time, uuid, glob
data = sys.argv[1]
tpl = sorted(glob.glob("/tmp/rg-soak-data/codex-home/sessions/2026/*/*/*.jsonl"))[0]
lines = [json.loads(l) for l in open(tpl)]
meta_line, ts_line = lines[0], next(l for l in lines if l.get("type")=="event_msg" and l["payload"]["type"]=="task_started")
tc_line = next(l for l in lines if l["type"]=="turn_context")
uri_line = next(l for l in lines if l["type"]=="response_item" and l["payload"]["role"]=="user")
um_line = next(l for l in lines if l.get("type")=="event_msg" and l["payload"]["type"]=="user_message")
fin_line = next(l for l in lines if l.get("type")=="event_msg" and l["payload"]["type"]=="task_complete")
def iso(ts):
    t=time.gmtime(ts); return f"{t.tm_year:04d}-{t.tm_mon:02d}-{t.tm_mday:02d}T{t.tm_hour:02d}:{t.tm_min:02d}:{t.tm_sec:02d}.000Z"
base = time.time() - 5400
d = time.gmtime(base); sid = str(uuid.uuid4())
day = os.path.join(data,"codex-home","sessions",f"{d.tm_year:04d}",f"{d.tm_mon:02d}",f"{d.tm_mday:02d}")
os.makedirs(day, exist_ok=True)
meta = json.loads(json.dumps(meta_line)); m=meta["payload"]
m["session_id"]=sid; m["id"]=sid; m["timestamp"]=iso(base); m["cwd"]="/tmp/d10-seed-ws"; meta["timestamp"]=iso(base)
out=[meta]; ts=base
for t in range(1, 9):
    turn=str(uuid.uuid4()); ts+=5
    a=json.loads(json.dumps(ts_line)); a["payload"]["turn_id"]=turn; a["payload"]["started_at"]=ts; a["timestamp"]=iso(ts); out.append(a)
    b=json.loads(json.dumps(tc_line)); b["payload"]["turn_id"]=turn; b["timestamp"]=iso(ts); out.append(b)
    text=f"a11y seeded chat — turn {t} of eight turns"
    c=json.loads(json.dumps(uri_line)); c["payload"]["id"]=f"msg_{turn[:18]}"
    c["payload"]["content"]=[{"type":"input_text","text":text}]
    c["payload"]["internal_chat_message_metadata_passthrough"]={"turn_id":turn}; c["timestamp"]=iso(ts); out.append(c)
    e=json.loads(json.dumps(um_line)); e["payload"]["message"]=text; e["timestamp"]=iso(ts); out.append(e)
    f=json.loads(json.dumps(fin_line)); f["payload"]["turn_id"]=turn
    f["payload"]["started_at"]=ts; f["payload"]["completed_at"]=ts+3; f["timestamp"]=iso(ts+3); out.append(f)
name=f"rollout-{iso(base).replace(':','-').replace('.','-')}-{sid}.jsonl"
open(os.path.join(day,name),"w").write("\n".join(json.dumps(l) for l in out)+"\n")
print("seeded 1 long chat (8 turns)")
EOF

pkill -f "Xvfb $DISP" 2>/dev/null; sleep 1
Xvfb "$DISP" -screen 0 1440x900x24 -nolisten tcp -noreset >"$OUT/xvfb.log" 2>&1 &
XPID=$!
sleep 1.5
DISPLAY=$DISP LD_LIBRARY_PATH="$LD_LIBRARY_PATH" picom --backend xrender --config /dev/null >"$OUT/picom.log" 2>&1 &
sleep 2

HOME_K=$(mktemp -d /tmp/rg-a11y-home.XXXXXX); mkdir -p "$HOME_K/xdg"
DISPLAY=$DISP LIBGL_ALWAYS_SOFTWARE=1 \
  VK_ICD_FILENAMES="/home/z/parity-lab/sysroot/usr/share/vulkan/icd.d/lvp_icd.json" \
  HOME="$HOME_K" XDG_RUNTIME_DIR="$HOME_K/xdg" \
  CODEX_RS_DATA_DIR="$DATA" CODEX_HOME="$DATA/codex-home" \
  CODEX_RS_CODEX_BIN="$RUNTIME" \
  "$BIN" >"$OUT/app.log" 2>&1 &
APP_PID=$!
say "boot APP_PID=$APP_PID (keyboard-only journey)"

rendered=0
for i in $(seq 1 40); do
  sleep 5
  kill -0 $APP_PID 2>/dev/null || { say "app exited early"; break; }
  DISPLAY=$DISP xdotool mousemove 720 450 click 1 2>/dev/null || true   # boot nudge only (pre-journey)
  cap "$OUT/_probe.png" 2>/dev/null
  SZ=$(stat -c%s "$OUT/_probe.png" 2>/dev/null || echo 0)
  [ "${SZ:-0}" -gt 30000 ] && { say "UI rendered after $((i*5))s"; rendered=1; break; }
done
rm -f "$OUT/_probe.png"
WID=$(xwininfo -display "$DISP" -root -children 2>/dev/null | grep -oE "^     0x[0-9a-f]+ \"codexRS\"" | grep -oE "0x[0-9a-f]+" | head -1)
[ -n "$WID" ] && DISPLAY=$DISP xdotool windowfocus --sync "$WID" 2>/dev/null
for i in $(seq 1 20); do CPID=$(pgrep -P $APP_PID -f "runtime/codex" | head -1); [ -n "$CPID" ] && break; sleep 2; done
sleep 8

say "LEG-00 entry baseline"
key Escape; sleep 1.5
cap "$OUT/a01-00-entry.png"; say "00 $(md5 "$OUT/a01-00-entry.png")"

say "LEG-1 PALETTE (Ctrl+K -> filter -> Escape)"
key ctrl+k; sleep 2.5
cap "$OUT/a01-01-palette-open.png"; say "01a $(md5 "$OUT/a01-01-palette-open.png")"
type_ "settings"; sleep 2
cap "$OUT/a01-02-palette-filtered.png"; say "01b $(md5 "$OUT/a01-02-palette-filtered.png")"
key Escape; sleep 1.5
cap "$OUT/a01-03-palette-closed.png"; say "01c $(md5 "$OUT/a01-03-palette-closed.png")"

say "LEG-2 SETTINGS (Ctrl+, -> Ctrl+F search -> Escape return)"
key ctrl+comma; sleep 2.5
cap "$OUT/a01-04-settings-open.png"; say "02a $(md5 "$OUT/a01-04-settings-open.png")"
key ctrl+f; sleep 1
type_ "appearance"; sleep 2
cap "$OUT/a01-05-settings-search.png"; say "02b $(md5 "$OUT/a01-05-settings-search.png")"
key Escape; sleep 1; key Escape; sleep 2
cap "$OUT/a01-06-settings-return.png"; say "02c $(md5 "$OUT/a01-06-settings-return.png")"

say "LEG-3 TERMINAL (Ctrl+\` -> type lands in PTY -> close)"
key ctrl+grave; sleep 3
cap "$OUT/a01-07-terminal-open.png"; say "03a $(md5 "$OUT/a01-07-terminal-open.png")"
type_ "echo a11y-terminal-focus"; sleep 0.8
key Return; sleep 2.5
cap "$OUT/a01-08-terminal-echo.png"; say "03b $(md5 "$OUT/a01-08-terminal-echo.png")"
key ctrl+grave; sleep 2
cap "$OUT/a01-09-terminal-closed.png"; say "03c $(md5 "$OUT/a01-09-terminal-closed.png")"

say "LEG-4 BROWSER (Ctrl+Shift+B -> Ctrl+L address -> Escape -> close)"
key ctrl+shift+b; sleep 3
cap "$OUT/a01-10-browser-open.png"; say "04a $(md5 "$OUT/a01-10-browser-open.png")"
key ctrl+l; sleep 1.5
cap "$OUT/a01-11-browser-address.png"; say "04b $(md5 "$OUT/a01-11-browser-address.png")"
key Escape; sleep 1
key ctrl+shift+b; sleep 2
cap "$OUT/a01-12-browser-closed.png"; say "04c $(md5 "$OUT/a01-12-browser-closed.png")"

say "LEG-5 HISTORY (Ctrl+, -> Ctrl+F browsing -> nav filter)"
key ctrl+comma; sleep 2.5
key ctrl+f; sleep 1
type_ "browsing"; sleep 2
cap "$OUT/a01-13-history-nav.png"; say "05a $(md5 "$OUT/a01-13-history-nav.png")"
key Escape; sleep 1; key Escape; sleep 2
cap "$OUT/a01-14-history-return.png"; say "05b $(md5 "$OUT/a01-14-history-return.png")"

say "LEG-6 ATTENTION (keyboard chat create -> mark -> jump -> clear)"
key ctrl+n; sleep 2.5
type_ "a11y attention chat"; sleep 1
key ctrl+Return; sleep 12
cap "$OUT/a01-15-chat-created.png"; say "06a $(md5 "$OUT/a01-15-chat-created.png")"
key ctrl+shift+u; sleep 2.5
cap "$OUT/a01-16-marked-unread.png"; say "06b $(md5 "$OUT/a01-16-marked-unread.png")"
key ctrl+alt+a; sleep 2.5
cap "$OUT/a01-17-jump-clear.png"; say "06c $(md5 "$OUT/a01-17-jump-clear.png")"
key ctrl+shift+u; sleep 2
key shift+Escape; sleep 2.5
cap "$OUT/a01-18-clear-all.png"; say "06d $(md5 "$OUT/a01-18-clear-all.png")"

say "LEG-7 CHAT SWAP (Ctrl+Shift+] / [)"
key ctrl+shift+bracketright; sleep 2.5
cap "$OUT/a01-19-next-chat.png"; say "07a $(md5 "$OUT/a01-19-next-chat.png")"
key ctrl+shift+bracketleft; sleep 2.5
cap "$OUT/a01-20-prev-chat.png"; say "07b $(md5 "$OUT/a01-20-prev-chat.png")"

say "LEG-8 OVERLAY (Ctrl+/ -> Escape ladder)"
key ctrl+slash; sleep 2.5
cap "$OUT/a01-21-overlay-open.png"; say "08a $(md5 "$OUT/a01-21-overlay-open.png")"
key Escape; sleep 1.5
cap "$OUT/a01-22-overlay-esc1.png"; say "08b $(md5 "$OUT/a01-22-overlay-esc1.png")"
key Escape; sleep 1.5
cap "$OUT/a01-23-overlay-esc2.png"; say "08c $(md5 "$OUT/a01-23-overlay-esc2.png")"

say "RG-A11Y-01 COMPLETE — 24 frames, keyboard-only after window placement"
