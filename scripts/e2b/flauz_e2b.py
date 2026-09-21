#!/usr/bin/env python3
"""flauz_e2b.py — E2B Desktop harness for the Flauz Linux GUI lane.

The E2B desktop is the PRIMARY Linux UX test environment for this lane
(operator directive). This module provides:

  - sandbox lifecycle (create / reconnect / persist id in state.json)
  - reproducible provisioning (apt deps, rustup 1.97.1, Flauz clone @ pinned
    commit, release build) with idempotent marker files inside the sandbox
  - GUI launch of the real Flauz binary under the desktop's X server
  - real user interaction primitives: screenshot / move / click / double
    click / drag / scroll / type / press
  - evidence capture (PNG + JSON action log per journey)

State lives in /home/z/flauz-lane/e2b/state.json; evidence in
/home/z/flauz-lane/e2b/evidence/. Credentials come from ~/.secrets/env.sh
(never committed, never logged, never placed in evidence).

Usage (from a Python shell or script):
    from flauz_e2b import FlauzDesktop
    fd = FlauzDesktop.create()      # or reconnect() if state.json has an id
    fd.provision()                  # idempotent
    fd.launch_gui()
    fd.shot("j01-01-cold-start")
    fd.click(x, y); fd.type_text("...")
"""
from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path

BASE = Path("/home/z/flauz-lane/e2b")
STATE = BASE / "state.json"
EVID = BASE / "evidence"
FLAUZ_SRC = "/root/Flauz.app"
BIN = f"{FLAUZ_SRC}/target/release/codexrs"

# Reproducibility pins (recorded in LINUX-GUI-E2B-ENVIRONMENT.md)
E2B_TEMPLATE = "e2b-desktop"  # resolved default; verified at create() time
RUST_TOOLCHAIN = "1.97.1"     # from rust-toolchain.toml
FLAUZ_REPO = "https://github.com/payswapdotorg/Flauz.app.git"

APT_DEPS = [
    "clang", "libclang-dev", "libdrm-dev", "libegl1-mesa-dev",
    "libfontconfig1-dev", "libfreetype6-dev", "libgbm-dev",
    "libgl1-mesa-dev", "libpipewire-0.3-dev", "libwayland-dev",
    "libx11-dev", "libx11-xcb-dev", "libxcb-render0-dev",
    "libxcb-shape0-dev", "libxcb-xfixes0-dev", "libxcb1-dev",
    "libxkbcommon-dev", "libxkbcommon-x11-dev", "pkg-config",
    "build-essential", "git", "curl", "unzip", "tar",
]


def _secret(var: str) -> str:
    """Read a credential from ~/.secrets/env.sh without printing it."""
    txt = Path(os.path.expanduser("~/.secrets/env.sh")).read_text()
    for line in txt.splitlines():
        if line.strip().startswith(f"export {var}="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    return ""


class FlauzDesktop:
    def __init__(self, sb, sid: str):
        self.sb = sb
        self.sid = sid

    # ------------------------------------------------------------ lifecycle
    @classmethod
    def create(cls, timeout_s: int = 6 * 3600, resolution=(1920, 1080)):
        from e2b_desktop import Sandbox
        sb = Sandbox.create(timeout=timeout_s, resolution=resolution)
        sid = sb.sandbox_id
        cls._save_state(sid, created=int(time.time()))
        print(f"[e2b] sandbox created: {sid}")
        print(f"[e2b] template: {getattr(sb, 'default_template', E2B_TEMPLATE)}")
        return cls(sb, sid)

    @classmethod
    def reconnect(cls):
        from e2b_desktop import Sandbox
        st = json.loads(STATE.read_text()) if STATE.exists() else {}
        sid = st.get("sandbox_id")
        if not sid:
            raise RuntimeError("no sandbox in state.json — call create()")
        sb = Sandbox.connect(sid, timeout=3600)
        print(f"[e2b] reconnected: {sid}")
        return cls(sb, sid)

    @classmethod
    def current(cls):
        """Return live harness if state exists, else None."""
        if not STATE.exists():
            return None
        try:
            return cls.reconnect()
        except Exception as e:
            print(f"[e2b] reconnect failed: {e}")
            return None

    # --- resolve the sandbox home (desktop template runs as non-root user) ---
    def home(self) -> str:
        r = self.run("echo $HOME", timeout=15).stdout.strip()
        return r or "/home/user"

    def _src(self) -> str:
        return f"{self.home()}/Flauz.app"

    def _flauz_dir(self) -> str:
        return f"{self.home()}/.flauz"

    @classmethod
    def _save_state(cls, sid: str, **extra):
        st = json.loads(STATE.read_text()) if STATE.exists() else {}
        st.update({"sandbox_id": sid, **extra})
        STATE.write_text(json.dumps(st, indent=2))

    def save_state(self, **kw):
        self._save_state(self.sid, **kw)

    # ------------------------------------------------------------ commands
    def run(self, cmd: str, timeout: int = 600):
        """Run a command; NEVER raises on non-zero exit (e2b raises
        CommandExitException by default — we degrade to a result object)."""
        try:
            return self.sb.commands.run(cmd, timeout=timeout)
        except Exception as e:
            code = getattr(e, "code", None)
            err = getattr(e, "error", None) or str(e)
            out = getattr(e, "result", None)

            class _R:
                pass
            r = _R()
            r.exit_code = code if isinstance(code, int) else 1
            r.stdout = getattr(out, "stdout", "") if out is not None else ""
            r.stderr = (err or "")[:800]
            return r

    def run_text(self, cmd: str, timeout: int = 600) -> str:
        return (self.run(cmd, timeout=timeout).stdout or "").strip()

    # ------------------------------------------------------------ provision
    def provision(self, commit: str = ""):
        """Idempotent: apt deps, rustup, clone, build. Marker-gated."""
        t0 = time.time()
        H = self.home()
        FLAUZ_SRC = f"{H}/Flauz.app"
        FL = f"{H}/.flauz"
        def done(marker: str) -> bool:
            return self.run(f"test -f {FL}/{marker} && echo OK",
                            timeout=15).stdout.strip() == "OK"

        self.run(f"mkdir -p {FL}")

        if not done("apt"):
            print("[prov] apt deps …")
            r = self.run(
                "sudo -n env DEBIAN_FRONTEND=noninteractive apt-get update -qq && "
                "sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends " + " ".join(APT_DEPS),
                timeout=900)
            if r.exit_code != 0:
                r = self.run(
                    "env DEBIAN_FRONTEND=noninteractive apt-get update -qq && "
                    "env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends " + " ".join(APT_DEPS),
                    timeout=900)
            assert r.exit_code == 0, f"apt failed: {r.stderr[-500:]}"
            self.run(f"touch {FL}/apt")

        if not done("rust"):
            print("[prov] rustup + toolchain 1.97.1 …")
            r = self.run(
                "curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain "
                f"{RUST_TOOLCHAIN} --profile minimal -c clippy,rustfmt", timeout=900)
            assert r.exit_code == 0, "rustup failed"
            self.run(f"touch {FL}/rust")

        # Codex CLI (runtime dependency; signed-out state is an honest journey)
        if not done("codex-cli"):
            print("[prov] codex CLI via npm …")
            r = self.run(
                "command -v npm >/dev/null 2>&1 || "
                "sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y "
                "--no-install-recommends npm >/dev/null 2>&1 || true; "
                "npm install -g @openai/codex 2>&1 | tail -1; codex --version || true",
                timeout=900)
            self.run(f"touch {FL}/codex-cli")
            print(f"[prov] codex cli: {r.stdout.strip()[-120:]}")

        if not done("clone"):
            pat = _secret("PAYSWAP_PAT")
            print("[prov] cloning Flauz.app …")
            url = FLAUZ_REPO.replace("https://", f"https://x-access-token:{pat}@")
            r = self.run(
                f"git clone --depth 50 {url} {FLAUZ_SRC} 2>&1 | sed 's/x-access-token:[^@]*@/****@/g' | tail -2 && "
                f"git -C {FLAUZ_SRC} config advice.detachedHead false", timeout=600)
            assert r.exit_code == 0, f"clone failed: {r.stderr[-300:]}"
            self.run(f"touch {FL}/clone")

        # Ubuntu 22.04 (E2B desktop image) ships pipewire 0.3.48 headers that
        # break libspa 0.10 (lane finding L-001). Side-install Ubuntu 24.04's
        # pipewire 1.0.5 dev+runtime into ~/pw10 (user-writable) and pin
        # pkg-config/LD_LIBRARY_PATH to it. Recorded in E2B-ENVIRONMENT doc.
        if not done("pw10") and not self.run_text(
                "test -f $HOME/pw10/usr/lib/x86_64-linux-gnu/pkgconfig/libspa-0.2.pc"
                " && echo OK") == "OK":
            print("[prov] pipewire 1.0.5 side-install (~/pw10) …")
            r = self.run(
                "set -e; mkdir -p ~/pw10 && cd ~/pw10; "
                "B=http://archive.ubuntu.com/ubuntu/pool/main/p/pipewire; "
                "for p in libspa-0.2-dev_1.0.5-1ubuntu3.3_amd64.deb "
                "libpipewire-0.3-dev_1.0.5-1ubuntu3.3_amd64.deb "
                "libpipewire-0.3-0t64_1.0.5-1ubuntu3.3_amd64.deb; do "
                "curl -sO $B/$p && dpkg -x $p ~/pw10; done; "
                "find ~/pw10 -name '*.pc' | while read f; do "
                "sed -i \"s|^prefix=/usr$|prefix=$HOME/pw10/usr|\" \"$f\"; done; "
                "echo PW10-OK", timeout=300)
            assert "PW10-OK" in (r.stdout or ""), f"pw10 failed: {r.stderr[-300:]}"
            self.run(f"touch {FL}/pw10")

        if commit:
            self.run(f"git -C {FLAUZ_SRC} fetch --depth 50 origin {commit} 2>/dev/null; "
                     f"git -C {FLAUZ_SRC} checkout -q {commit}", timeout=300)

        cur = self.run(f"git -C {FLAUZ_SRC} rev-parse HEAD", timeout=30).stdout.strip()
        print(f"[prov] Flauz @ {cur[:12]}")

        build_marker = f"build-{cur[:12]}"
        if not done(build_marker):
            print("[prov] cargo build --release -p codex-app (long) …")
            r = self.run(
                f"source $HOME/.cargo/env && cd {FLAUZ_SRC} && "
                f"export PKG_CONFIG_PATH=$HOME/pw10/usr/lib/x86_64-linux-gnu/pkgconfig && "
                "set -o pipefail && cargo build --release -p codex-app "
                "> /tmp/build.log 2>&1; ec=$?; echo BUILD_EXIT=$ec; "
                "tail -8 /tmp/build.log; test $ec -eq 0",
                timeout=3600)
            assert r.exit_code == 0, f"build failed:\n{(r.stdout or '')[-800:]}\n{(r.stderr or '')[-300:]}"
            self.run(f"touch {FL}/{build_marker}")
        else:
            print(f"[prov] build marker {build_marker} present")

        ok = self.run(f"test -x {FLAUZ_SRC}/target/release/codexrs && echo OK", timeout=15).stdout.strip()
        assert ok == "OK", f"binary missing at {FLAUZ_SRC}/target/release/codexrs"

        self.save_state(provisioned=int(time.time()), flauz_commit=cur,
                        template=E2B_TEMPLATE)
        print(f"[prov] done in {int(time.time()-t0)}s @ {cur[:12]}")
        return cur

    # ------------------------------------------------------------ gui launch
    def launch_gui(self, fresh_data: bool = False):
        """Kill old instances and start the real Flauz GUI on the desktop."""
        H = self.home()
        BIN = f"{H}/Flauz.app/target/release/codexrs"
        self.run("pkill -f 'target/release/codexrs' 2>/dev/null; sleep 1; true",
                 timeout=20)
        if fresh_data:
            self.run("rm -rf $HOME/.local/share/codexRS $HOME/.codexRS 2>/dev/null; true",
                     timeout=20)
        r = self.run(
            f"export DISPLAY={self._display()} && "
            f"export LD_LIBRARY_PATH=$HOME/pw10/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH && "
            f"export CODEX_RS_CODEX_BIN=$(command -v codex || echo /usr/local/bin/codex) && "
            f"nohup {BIN} > {H}/.flauz/gui.log 2>&1 & sleep 3; "
            f"pgrep -af codexrs | head -2", timeout=60)
        print("[gui]", (r.stdout or r.stderr).strip()[:300])
        self.save_state(gui_launched=int(time.time()))
        return r

    def _display(self) -> str:
        r = self.run("echo $DISPLAY", timeout=15).stdout.strip()
        return r or ":1"

    def gui_log(self, tail: int = 30) -> str:
        return self.run(f"tail -{tail} $HOME/.flauz/gui.log 2>/dev/null || true",
                        timeout=20).stdout

    # ------------------------------------------------------------ interaction
    def shot(self, name: str) -> Path:
        """Screenshot to evidence dir; returns path."""
        EVID.mkdir(parents=True, exist_ok=True)
        path = EVID / f"{int(time.time())}-{name}.png"
        data = self.sb.screenshot(format="bytes")
        path.write_bytes(data)
        return path

    def click(self, x: int, y: int, name: str = ""):
        self.sb.move_mouse(x, y)
        self.sb.left_click()
        time.sleep(0.8)
        if name:
            self.shot(f"{name}-after-click")

    def double_click(self, x: int, y: int):
        self.sb.move_mouse(x, y)
        self.sb.double_click()
        time.sleep(0.5)

    def drag(self, x1: int, y1: int, x2: int, y2: int):
        self.sb.drag(x1, y1, x2, y2)
        time.sleep(0.5)

    def scroll(self, direction="down", amount=3):
        self.sb.scroll(direction=direction, amount=amount)
        time.sleep(0.4)

    def type_text(self, text: str):
        for line in text.split("\n"):
            if line:
                self.sb.write(line)
            time.sleep(0.15)

    def key(self, k):
        self.sb.press(k)
        time.sleep(0.3)

    def screen_size(self):
        return self.sb.get_screen_size()

    def windows(self):
        try:
            return self.sb.get_application_windows()
        except Exception as e:
            return f"windows-query-failed: {e}"

    def window_title(self):
        try:
            return self.sb.get_window_title()
        except Exception:
            return ""

    # ------------------------------------------------------------ info
    def env_info(self) -> dict:
        info = {
            "sandbox_id": self.sid,
            "template": E2B_TEMPLATE,
        }
        for k, cmd in {
            "os": "cat /etc/os-release | head -2",
            "kernel": "uname -r",
            "display": "echo $DISPLAY",
            "desktop": "ls /usr/share/xsessions/ 2>/dev/null || ls /etc/xdg/autostart >/dev/null 2>&1 && echo unknown",
            "rust": "source ~/.cargo/env && rustc --version",
            "codex_cli": "codex --version 2>/dev/null || echo not-found",
            "flauz": f"git -C {self.home()}/Flauz.app log --oneline -1",
            "cpu": "nproc",
            "mem": "free -h | head -2 | tail -1",
            "renderer": "glxinfo -B 2>/dev/null | head -5 || echo no-glxinfo",
        }.items():
            try:
                info[k] = self.run(cmd, timeout=30).stdout.strip()[:200]
            except Exception as e:
                info[k] = f"error: {e}"
        return info


if __name__ == "__main__":
    what = sys.argv[1] if len(sys.argv) > 1 else "info"
    commit = sys.argv[2] if len(sys.argv) > 2 else ""
    fd = FlauzDesktop.current() or FlauzDesktop.create()
    if what == "provision":
        fd.provision(commit)
    elif what == "launch":
        fd.launch_gui()
        time.sleep(4)
        fd.shot("launch")
        print(fd.gui_log(15))
    elif what == "info":
        print(json.dumps(fd.env_info(), indent=2))
    elif what == "reconnect":
        print("connected:", fd.sid)
