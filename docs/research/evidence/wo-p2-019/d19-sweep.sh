#!/bin/bash
# d19-sweep.sh — click-sweep to find the Clear button by modal-overlay detection.
# A modal opening adds a dark scrim over the page -> mean brightness drops.
set -u
OUT=/home/z/parity-lab/evidence/d19
DISP=$(cat "$OUT/disp")
export PATH="/home/z/parity-lab/sysroot/usr/bin:/home/z/.local/desktop-tools/usr/bin:$PATH"
export LD_LIBRARY_PATH="/home/z/parity-lab/sysroot/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/usr/lib/x86_64-linux-gnu:/home/z/.local/desktop-tools/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"

meanbright() { python3 -c "
from PIL import Image
import sys
im = Image.open('$1').convert('L')
im = im.resize((180,112))
px = list(im.getdata())
print(sum(px)//len(px))" 2>/dev/null; }

cap() { ffmpeg -y -loglevel error -f x11grab -video_size 1440x900 -i $DISP -frames:v 1 "$1"; }

cap "$OUT/_sweep_base.png"
BASE=$(meanbright "$OUT/_sweep_base.png")
echo "baseline brightness: $BASE"

for Y in 560 575 590 605; do
  for X in 780 840 900 960 1020 1080 1140; do
    DISPLAY=$DISP xdotool mousemove $X $Y click 1
    sleep 0.9
    cap "$OUT/_sweep_try.png"
    B=$(meanbright "$OUT/_sweep_try.png")
    DELTA=$((BASE - B))
    echo "click ($X,$Y) brightness=$B delta=$DELTA"
    if [ "$DELTA" -gt 8 ]; then
      echo "MODAL-LIKE DROP at ($X,$Y) — saving"
      cp "$OUT/_sweep_try.png" "$OUT/d19-sweep-hit.png"
      echo "$X $Y" > "$OUT/_sweep_hit_coords"
      exit 0
    fi
    # if the page navigated away (big change but not a scrim), re-navigate
    if [ "$DELTA" -lt -8 ]; then
      echo "page changed at ($X,$Y) — re-navigating"
      DISPLAY=$DISP xdotool key Escape; sleep 1
      DISPLAY=$DISP xdotool key ctrl+k; sleep 1.5
      DISPLAY=$DISP xdotool type --delay 60 "browser settings"; sleep 1.5
      DISPLAY=$DISP xdotool key Return; sleep 2
      cap "$OUT/_sweep_base.png"; BASE=$(meanbright "$OUT/_sweep_base.png")
      echo "new baseline: $BASE"
    fi
  done
done
echo "SWEEP DONE — no modal detected"
exit 1
