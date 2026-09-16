source /home/z/parity-lab/tools/scene_helpers.sh
sleep 2
CAPTURE 01-default-window
# Click-path the Terminal affordance (sidebar footer, lower area)
XDO mousemove 60 940 click 1
sleep 2
CAPTURE 02-terminal-clicked
# Click-path the Browser affordance (sidebar footer)
XDO mousemove 60 975 click 1
sleep 2
CAPTURE 03-browser-clicked
