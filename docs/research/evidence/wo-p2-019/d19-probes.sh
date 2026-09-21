#!/bin/bash
# D19 probes — the trap + background-Tab legs (app already parked at
# Settings > Browser by d19-phaseC.sh; env/discovery from $OUT files).
set -u
OUT="${1:?outdir}"
DISP=$(cat "$OUT/disp")
export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }
md5() { md5sum "$1" | awk '{print $1}'; }
key() { DISPLAY=$DISP xdotool key "$1"; }
clickat() { DISPLAY=$DISP xdotool mousemove "$1" "$2" click 1; }

echo "--- [C4] open the Clear modal"
clickat 762 650; sleep 2
cap "$OUT/d19-04-modal-open.png"; echo "F04 $(md5 "$OUT/d19-04-modal-open.png")"

echo "--- [C5] Tab #1"
key Tab; sleep 1.2
cap "$OUT/d19-05-tab1.png"; echo "F05 $(md5 "$OUT/d19-05-tab1.png")"

echo "--- [C6] Tab #2 (WRAP — the trap proof)"
key Tab; sleep 1.2
cap "$OUT/d19-06-tab2-wrap.png"; echo "F06 $(md5 "$OUT/d19-06-tab2-wrap.png")"

echo "--- [C7] Tab #3 (alternate)"
key Tab; sleep 1.2
cap "$OUT/d19-07-tab3.png"; echo "F07 $(md5 "$OUT/d19-07-tab3.png")"

echo "--- [C8] Shift+Tab (backward wrap)"
key shift+Tab; sleep 1.2
cap "$OUT/d19-08-shifttab.png"; echo "F08 $(md5 "$OUT/d19-08-shifttab.png")"

echo "--- [C9] Escape closes the modal"
key Escape; sleep 1.5
cap "$OUT/d19-09-modal-closed.png"; echo "F09 $(md5 "$OUT/d19-09-modal-closed.png")"

echo "--- [C10] leave settings (escape ladder)"
key Escape; sleep 1.5
key Escape; sleep 1.5
cap "$OUT/d19-10-main-surface.png"; echo "F10 $(md5 "$OUT/d19-10-main-surface.png")"

echo "--- [C11] composer focus (ladder)"
for Y in 849 829 809 789; do DISPLAY=$DISP xdotool mousemove 802 $Y click 1; sleep 0.3; done
sleep 1
cap "$OUT/d19-11-composer-focus.png"; echo "F11 $(md5 "$OUT/d19-11-composer-focus.png")"

echo "--- [C12] background Tab #1"
key Tab; sleep 1.2
cap "$OUT/d19-12-bg-tab1.png"; echo "F12 $(md5 "$OUT/d19-12-bg-tab1.png")"

echo "--- [C13] background Tab #2"
key Tab; sleep 1.2
cap "$OUT/d19-13-bg-tab2.png"; echo "F13 $(md5 "$OUT/d19-13-bg-tab2.png")"

echo "PROBES DONE — teardown is Lead-driven after VLM adjudication"
