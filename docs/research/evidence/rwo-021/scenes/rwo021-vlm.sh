#!/bin/bash
# RWO-021 — VLM reads of scene frames (z-ai CLI, house vlm-*.json pattern)
set -u
EV=/home/z/rwo021-evidence
cd "$EV"

vlm() { # file prompt out
  local f="$1" p="$2" o="$3"
  if [ ! -s "$o" ]; then
    z-ai vision -p "$p" -i "$f" -o "$o" >/dev/null 2>&1 || echo "VLM_FAIL $f"
  fi
  echo "=== $o ==="
  python3 -c "import json,sys; d=json.load(open('$o')); print(d.get('content') or d.get('choices',[{}])[0].get('message',{}).get('content') or d)" 2>/dev/null | head -22
}

vlm a01-ws-baseline.png "Describe this desktop app window precisely: 1) left sidebar contents (buttons/rows verbatim), 2) the main surface (empty state text verbatim if any), 3) the bottom footer status area (status text verbatim, connection state), 4) any indication of an open workspace/project." vlm-a01.json

vlm a02-ctrl-p-palette.png "A keyboard interaction just happened (Ctrl+P). Describe exactly what overlay/palette is visible: its title or input placeholder (verbatim), any section headings (verbatim), listed items (verbatim), and whether an input field is focused." vlm-a02.json

vlm a04-ctrl-slash-overlay.png "A 'Keyboard shortcuts' overlay was opened with Ctrl+/. List: 1) the overlay title/search placeholder verbatim, 2) ALL section/group headings in order, 3) the number of rows per group and total rows, 4) the first and last row text verbatim." vlm-a04.json

vlm a06-ctrl-g.png "Describe the visible palette/overlay after Ctrl+G: input placeholder verbatim, sections, rows." vlm-a06.json

vlm a08-feedback-dialog.png "Is a 'Share feedback' dialog/modal visible? Describe: title verbatim, category buttons/labels verbatim, any checkbox for logs (state), details text field, validation hints, submit button." vlm-a08.json

vlm a09-composer-feedback-typed.png "Describe exactly what is on screen: is there a message composer with typed text (what text verbatim)? Is a slash-command dropdown visible (which rows verbatim)? Any modal open?" vlm-a09.json

vlm a10-feedback-dialog-slash.png "Compare to the previous frame: after pressing Return on the typed '/feedback', what changed? Is a 'Share feedback' dialog open now? Describe title, categories, log checkbox, details field verbatim." vlm-a10.json
