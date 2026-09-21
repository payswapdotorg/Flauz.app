#!/usr/bin/env python3
"""run3_diag.py — L-002 P0 (black window) root-cause discrimination run.

Environment hypothesis matrix (to be discriminated empirically):
  H1  Vulkan missing/broken        → vulkaninfo fails / no ICD
  H2  Vulkan present-mode/WSI fail → VK_LOADER_DEBUG shows surface/swapchain errors
  H3  ARGB visual + no compositor  → window shows content ONLY after xcompmgr
  H4  first frame never drawn      → 0% CPU on codexrs while black
  H5  app-server missing blocks UI → codex CLI now pinned-installed; if UI
                                      appears it shows an app-server state

Everything is captured to evidence/ + a JSON summary in run3-summary.json.
"""
import json
import time
from pathlib import Path

from flauz_e2b import FlauzDesktop, EVID

SUMMARY = Path("/home/z/flauz-lane/e2b/run3-summary.json")
out = {"phases": {}}

print("=== RUN-3 L-002 DIAGNOSTIC ===")
fd = FlauzDesktop.create()
out["sandbox_id"] = fd.sid
fd.provision()

# --- phase 1: environment inventory --------------------------------------
print("\n--- phase 1: render_probe ---")
out["phases"]["probe"] = fd.render_probe()

# --- phase 2: launch GUI with loader debug, NO compositor -----------------
print("\n--- phase 2: launch_gui_debug (no compositor) ---")
r = fd.launch_gui_debug(compositor=False)
out["phases"]["launch_nocomp"] = (r.stdout or "")[:1500]
time.sleep(6)
p1 = fd.shot("run3-j01-nocomp")
out["shot_nocomp"] = str(p1)

cpu = fd.run("top -b -n 1 | grep codexrs | head -2", timeout=30).stdout or ""
out["cpu_nocomp"] = cpu.strip()[:300]
print("[cpu-nocomp]", out["cpu_nocomp"])

wininfo = fd.run(
    "export DISPLAY=" + fd._display() + " && xwininfo -root -tree 2>/dev/null | "
    "grep -iE 'codexrs|codex' | head -3; "
    "export DISPLAY=" + fd._display() + " && xwininfo -id $(xdotool search --class "
    "CodexRS 2>/dev/null | head -1) 2>/dev/null | grep -E 'Depth|Visual|Geometry|Map State'",
    timeout=30).stdout or ""
out["wininfo"] = wininfo.strip()[:500]
print("[wininfo]", out["wininfo"])

# --- phase 3: compositor variant ------------------------------------------
print("\n--- phase 3: launch_gui_debug (WITH xcompmgr) ---")
r = fd.launch_gui_debug(compositor=True)
out["phases"]["launch_comp"] = (r.stdout or "")[:1500]
time.sleep(6)
p2 = fd.shot("run3-j01-compositor")
out["shot_comp"] = str(p2)
cpu2 = fd.run("top -b -n 1 | grep codexrs | head -2", timeout=30).stdout or ""
out["cpu_comp"] = cpu2.strip()[:300]

# --- phase 4: raw pixel analysis of both shots -----------------------------
for key, shot in (("pixels_nocomp", p1), ("pixels_comp", p2)):
    ana = fd.run(
        f"python3 - <<'EOF'\n"
        f"import struct, zlib, subprocess\n"
        f"# read via imagemagick stats on the sandbox screenshot instead\n"
        f"EOF\n"
        f"identify -format '%[mean] %[standard_deviation]' {shot} 2>/dev/null || true",
        timeout=30).stdout or ""
    out[key] = ana.strip()

SUMMARY.write_text(json.dumps(out, indent=2))
print("\n=== RUN-3 SUMMARY written to", SUMMARY, "===")
print("sandbox:", fd.sid, "— kept alive for interactive follow-up")
