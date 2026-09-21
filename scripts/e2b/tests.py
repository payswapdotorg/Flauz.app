#!/usr/bin/env python3
"""LAB-002 — unit tests for the E2B journey battery.

Stdlib-runnable:  python3 scripts/e2b/tests.py
(also pytest-compatible: plain test_* functions with asserts)

Coverage (acceptance criterion 2):
  * spec parsing / structural validation / cold-start protocol
  * verdict computation from step results (all 8 vocabulary values)
  * evidence naming determinism
  * matrix / ledger Markdown emission (golden-string compare)
  * dry-run plan generation (subprocess: exit 0, no e2b import, no files)
  * full battery against a FakeSandbox that models the rc.14 signed-out
    surface (run-4 ground truth) — verdicts, evidence, actions.jsonl,
    verdicts.json, matrix/ledger application, resume, baseline compare,
    and the no-PIL degradation path.
No network, no E2B, no credentials.
"""
from __future__ import annotations

import io
import json
import os
import re
import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import journey_specs as jspec  # noqa: E402
import journeys as jrn  # noqa: E402

PASS = FAIL = 0

# ---------------------------------------------------------------------------
# tiny stdlib PNG encoder (grayscale) for the FakeSandbox screen
W, H = 1920, 1080


def _png_chunk(tag: bytes, data: bytes) -> bytes:
    return (struct.pack(">I", len(data)) + tag + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))


def encode_png(rows) -> bytes:
    raw = b"".join(b"\x00" + bytes(r) for r in rows)
    ihdr = struct.pack(">IIBBBBB", W, H, 8, 0, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + _png_chunk(b"IHDR", ihdr)
            + _png_chunk(b"IDAT", zlib.compress(raw, 1))
            + _png_chunk(b"IEND", b""))


class FakeScreen:
    """Deterministic fake of the rc.14 signed-out surface (run-4 ground truth).

    Models: sidebar rows, composer (+ L-010 draft lost on restart), honest
    sign-in gate on New chat, sidebar tool views, honest task-required toasts
    (L-006), the L-009 silent Projects '+', the L-012 first-run promo modal,
    pinned toasts across restart (L-011), and the command palette with a
    query-independent empty state (so the calibrated differential works).
    """

    KNOWN_COMMANDS = [
        "new chat", "new standalone chat", "open folder", "settings",
        "appearance", "terminal", "browser", "toggle sidebar",
        "search chats", "back", "forward", "select model", "workflows",
        "teach a workflow", "open project picker", "add new project",
    ]

    def __init__(self):
        self.sandbox_id = "fake-sbx-lab002"
        self.events = []
        self.mouse = (960, 540)
        self.view = "main"          # main|gate|repository|pullrequests|plugins|
                                    # workflows|settings|settings-cat|model-picker|
                                    # chat-search
        self.composer_text = ""
        self.composer_focused = False
        self.palette_open = False
        self.palette_query = ""
        self.palette_entries = []
        self.promo_open = True      # first-run modal (L-012)
        self.toast = None           # pinned (L-011), survives relaunch
        self.launch_count = 0

    # ---------------- raw e2b-desktop Sandbox API ------------------------
    def move_mouse(self, x, y):
        self.mouse = (x, y)
        self.events.append(("move_mouse", x, y))

    def left_click(self):
        self.events.append(("left_click", *self.mouse))
        self._click(*self.mouse)

    def double_click(self):
        self.events.append(("double_click", *self.mouse))
        self._click(*self.mouse)

    def right_click(self):
        self.events.append(("right_click", *self.mouse))

    def drag(self, x1, y1, x2, y2):
        self.events.append(("drag", x1, y1, x2, y2))

    def scroll(self, direction="down", amount=3):
        self.events.append(("scroll", direction, amount))

    def write(self, text):
        self.events.append(("write", text))
        if self.palette_open:
            self.palette_query += text
            self._refresh_palette()
        elif self.composer_focused and self.view == "main":
            self.composer_text += text

    def press(self, key):
        k = key if isinstance(key, str) else "+".join(key)
        self.events.append(("press", k))
        self._key(k)

    def screenshot(self, format="bytes"):
        assert format == "bytes"
        return self._render()

    def get_window_title(self):
        return "codexRS"

    def get_application_windows(self):
        return ["codexRS"]

    def set_timeout(self, seconds):
        self.events.append(("set_timeout", seconds))

    # ---------------- behaviour model -------------------------------------
    def _relaunch(self):
        self.launch_count += 1
        self.view = "main"
        self.palette_open = False
        self.palette_query = ""
        self.composer_focused = False
        self.composer_text = ""     # L-010: draft lost on restart
        # promo + toast persist (first-run modal / L-011 pinned toast)

    def _refresh_palette(self):
        q = self.palette_query.strip().lower()
        if not q:
            self.palette_entries = ["new chat", "open folder", "settings",
                                    "search chats"]
        else:
            words = q.split()
            self.palette_entries = [c for c in self.KNOWN_COMMANDS
                                    if all(w in c for w in words)]

    def _key(self, k):
        if k == "ctrl+shift+p":
            if not self.promo_open:          # modal blocks the palette
                self.palette_open = True
                self.palette_query = ""
                self._refresh_palette()
            return
        if k == "Escape":
            if self.palette_open:
                self.palette_open = False
            elif self.view == "model-picker":
                self.view = "main"
            elif self.view == "chat-search":
                self.view = "main"
            elif self.view == "gate":
                self.view = "main"
            return
        if k == "ctrl+g":
            if not self.promo_open and not self.palette_open:
                self.view = "chat-search"
            return
        if k == "Return":
            if self.palette_open:
                if self.palette_entries:
                    self._exec_command(self.palette_entries[0])
            elif self.composer_focused and self.composer_text \
                    and self.view == "main":
                self.view = "gate"           # send attempt -> honest gate
            return

    def _exec_command(self, cmd):
        self.palette_open = False
        self.palette_query = ""
        if "new chat" in cmd:
            self.view = "gate"
        elif cmd == "search chats":
            self.view = "chat-search"
        elif cmd == "terminal":
            self.toast = "Select a task before opening a terminal."
        elif cmd == "browser":
            self.toast = "Open a chat before opening the Browser."
        elif cmd == "select model":
            self.view = "model-picker"
        elif "workflow" in cmd:
            self.view = "workflows"
        elif cmd == "settings":
            self.view = "settings"
        else:
            self.view = "main"

    def _near(self, x, y, tx, ty, tol=25):
        return abs(x - tx) <= tol and abs(y - ty) <= tol

    def _click(self, x, y):
        # promo modal primary (L-012): only dismiss path; also switches model
        if self.promo_open and self._near(x, y, 778, 560):
            self.promo_open = False
            return
        if self._near(x, y, 60, 116):            # New chat -> honest gate
            self.view = "gate"
        elif self._near(x, y, 60, 152):          # Repository
            self.view = "repository"
        elif self._near(x, y, 60, 186):          # Pull requests (+ gh toast)
            self.view = "pullrequests"
            self.toast = "GitHub CLI (gh) is not installed"
        elif self._near(x, y, 60, 224):          # Plugins
            self.view = "plugins"
        elif self._near(x, y, 60, 259):          # Workflows
            self.view = "workflows"
        elif self._near(x, y, 252, 314, 12):     # Projects '+' — L-009 no-op
            pass
        elif self._near(x, y, 60, 745):          # Terminal -> honest toast
            self.toast = "Select a task before opening a terminal."
        elif self._near(x, y, 60, 775):          # Browser -> honest toast
            self.toast = "Open a chat before opening the Browser."
        elif self._near(x, y, 60, 822):          # Settings
            self.view = "settings"
        elif self._near(x, y, 95, 112):          # Back to app
            self.view = "main"
        elif self._near(x, y, 700, 740):         # composer focus
            if self.view == "main":
                self.composer_focused = True
        elif self._near(x, y, 855, 822):         # model pill
            if self.view == "main":
                self.view = "model-picker"
        elif self._near(x, y, 430, 260):         # settings category
            if self.view == "settings":
                self.view = "settings-cat"

    # ---------------- rendering --------------------------------------------
    def _render(self) -> bytes:
        rows = [bytearray([221]) * W for _ in range(H)]

        def hline(x0, x1, y, v):
            if 0 <= y < H:
                rows[y][max(0, x0):min(W, x1)] = bytes([v]) * (min(W, x1) - max(0, x0))

        def block(x0, y0, x1, y1, v):
            for y in range(max(0, y0), min(H, y1)):
                hline(x0, x1, y, v)

        # sidebar (persistent sections)
        for yy in (116, 152, 186, 224, 259, 314, 349, 745, 775, 822):
            hline(32, 120, yy, 60)
            hline(32, 120, yy + 4, 90)
        hline(10, 285, 96, 190)
        hline(240, 262, 314, 60)  # the '+' affordance

        # main area per view
        v = self.view
        if v == "main":
            hline(400, 900, 300, 60)   # hero "What should we work on?"
            hline(430, 860, 330, 100)
            hline(500, 780, 520, 120)
        elif v == "gate":
            block(450, 250, 850, 650, 235)
            for i in range(9):
                hline(480, 820, 280 + i * 38, 60)
        elif v == "repository":
            for i in range(6):
                hline(350, 1200, 160 + i * 34, 70)
        elif v == "pullrequests":
            for i in range(5):
                hline(350, 1150, 150 + i * 36, 80)
        elif v == "plugins":
            for i in range(14):
                hline(320, 1270, 130 + i * 48, 75)
        elif v == "workflows":
            for i in range(5):
                hline(350, 1250, 140 + i * 30, 65)
            for i in range(5):
                hline(350, 1250, 420 + i * 34, 95)
        elif v == "settings":
            hline(40, 280, 112, 60)            # "Back to app"
            hline(40, 250, 172, 120)           # search settings
            for i in range(9):
                hline(350, 700, 210 + i * 35, 70)
        elif v == "settings-cat":
            hline(40, 280, 112, 60)
            for i in range(11):
                hline(720, 1270, 150 + i * 40, 85)
        elif v == "model-picker":
            for i in range(7):
                hline(500, 1000, 310 + i * 50, 55)
        elif v == "chat-search":
            for i in range(6):
                hline(400, 1100, 160 + i * 45, 65)

        # composer (main view only)
        if v == "main":
            hline(300, 1285, 699, 190)
            hline(300, 1285, 870, 190)
            if self.composer_text:
                hline(320, 320 + 9 * len(self.composer_text), 718, 40)
            hline(816, 900, 820, 150)   # model pill
            hline(975, 1030, 820, 150)  # effort pill
            hline(1110, 1160, 820, 150)  # mode pill
            hline(1220, 1250, 820, 40)  # send

        # pinned toast (L-011) — main-area, survives navigation + restart
        if self.toast:
            block(620, 505, 1010, 545, 30)
            off = sum(ord(c) for c in self.toast) % 12
            hline(640, 990, 515 + off, 220)

        # command palette overlay (blocks the main view)
        if self.palette_open:
            block(410, 247, 900, 715, 245)
            hline(430, 880, 255, 60)             # search input caret line
            hline(430, 430 + 8 * len(self.palette_query), 258, 90)
            if self.palette_entries:
                for i, cmd in enumerate(self.palette_entries[:8]):
                    hline(430, 880, 350 + i * 40, 55)
                    hline(430, 700, 368 + i * 40, 110)
            else:
                hline(430, 700, 350, 60)         # fixed "No results" (no echo)

        # first-run promo modal (L-012)
        if self.promo_open:
            block(410, 380, 870, 650, 250)
            hline(450, 830, 420, 60)
            hline(450, 830, 450, 100)
            block(695, 540, 860, 585, 40)        # primary button

        return encode_png(rows)


class FakeDesktop:
    """FlauzDesktop-alike for the battery executor."""

    def __init__(self, screen=None):
        self.sb = screen or FakeScreen()
        self.relaunch_count = 0

    def relaunch(self):
        self.relaunch_count += 1
        self.sb._relaunch()

    def provision(self, commit=""):
        return "89cdbe0aa111"   # fake pinned commit

    def gui_log(self, tail=30):
        return "fake gui log (app-server online)\n"


def make_ctx(repo_root: Path, run_id="t-run", battery="baseline"):
    ctx = jrn.RunContext(repo_root=repo_root, run_id=run_id, battery=battery)
    return ctx


def run_fake_battery(repo_root: Path, run_id="t-run", journeys_arg="",
                     desktop=None, battery="baseline"):
    desktop = desktop or FakeDesktop()
    args = type("A", (), {"battery": battery, "run_id": run_id,
                          "journeys": journeys_arg, "commit": ""})()
    plan = jrn.build_plan(args)
    ctx = make_ctx(repo_root, run_id, battery)
    ctx.sandbox_id = desktop.sb.sandbox_id   # mirrors live_desktop()
    rr = jrn.execute_battery(desktop, ctx, plan)
    ctx.close()
    return rr, desktop, ctx


def no_sleep():
    real = sys.modules["time"].sleep
    sys.modules["time"].sleep = lambda s: None
    return real


# ---------------------------------------------------------------------------
# 1. spec parsing / structure
def test_specs_parse():
    errs = jspec.validate_all()
    assert errs == [], f"spec validation errors: {errs}"
    assert len(jspec.SPECS) == 18
    assert [s.jid for s in jspec.SPECS] == [f"J{i:02d}" for i in range(1, 19)]
    for s in jspec.SPECS:
        assert s.title and s.kind in jspec.KINDS
        assert s.expected_verdict in jspec.VERDICT_VOCAB
        assert s.steps and s.palette_steps, f"{s.jid}: both passes required"
        assert s.entries, f"{s.jid}: entries summary required (matrix row)"
        for st in (*s.steps, *s.palette_steps):
            assert st.action in jspec.ACTIONS
            assert st.slug and re.fullmatch(r"[a-z0-9][a-z0-9-]*", st.slug), \
                f"{s.jid}: bad slug {st.slug!r}"
            assert st.on_fail in ("defect", "abort")


def test_cold_start_protocol():
    # primary pass = normal UI first: no palette action, no palette combo
    for s in jspec.SPECS:
        for st in s.steps:
            assert st.action != "palette_probe", \
                f"{s.jid}: palette action leaked into the primary pass"
            if isinstance(st.key, (tuple, list)):
                assert [k.lower() for k in st.key] != jspec.PALIN, \
                    f"{s.jid}: palette combo in the primary pass"
        # the palette pass must exercise the search fallback
        assert any(st.action == "palette_probe" for st in s.palette_steps) or \
            any(isinstance(st.key, (tuple, list)) and "g" in [k.lower() for k in st.key]
                for st in s.palette_steps), \
            f"{s.jid}: palette pass does not exercise the search fallback"


def test_expected_verdicts_match_run4_ground_truth():
    run4 = {
        "J01": "WORKS", "J02": "UNAVAILABLE-BUT-HONEST",
        "J03": "WORKS-WITH-DEFECTS", "J04": "WORKS-WITH-DEFECTS",
        "J05": "SILENT-NO-OP", "J06": "UNAVAILABLE-BUT-HONEST",
        "J07": "UNAVAILABLE-BUT-HONEST", "J08": "UNAVAILABLE-BUT-HONEST",
        "J09": "UNAVAILABLE-BUT-HONEST", "J10": "UNAVAILABLE-BUT-HONEST",
        "J11": "WORKS", "J12": "UNAVAILABLE-BUT-HONEST",
        "J13": "UNAVAILABLE-BUT-HONEST", "J14": "WORKS-WITH-DEFECTS",
        "J15": "UNAVAILABLE-BUT-HONEST", "J16": "UNAVAILABLE-BUT-HONEST",
        "J17": "MISSING", "J18": "UNAVAILABLE-BUT-HONEST",
    }
    for s in jspec.SPECS:
        assert s.expected_verdict == run4[s.jid], \
            f"{s.jid}: expected_verdict {s.expected_verdict} != run-4 {run4[s.jid]}"


# ---------------------------------------------------------------------------
# 2. verdict engine
def _step(pass_name="primary", n=1, action="click", **kw):
    return jrn.StepRecord(jid="J01", pass_name=pass_name, n=n, slug=f"s{n}",
                          action=action, executed=True, **kw)


def _assertion(passed):
    return jrn.AssertionRecord("region_change", "rect", "val", passed,
                               "pil-region")


def _jr(kind, steps, aborted=None):
    return jrn.JourneyResult(jid="J01", title="t", kind=kind,
                             expected_verdict="WORKS", expected_note="",
                             steps=steps, aborted=aborted)


def _spec(kind):
    return type("S", (), {"kind": kind})()


def test_verdict_not_run():
    v, r = jrn.compute_verdict(_spec("flow"), _jr("flow", [], aborted="sandbox died"))
    assert v == "NOT-RUN" and "sandbox died" in r
    v, _ = jrn.compute_verdict(_spec("flow"), _jr("flow", []))
    assert v == "NOT-RUN"


def test_verdict_flow_works():
    jr = _jr("flow", [_step(assertions=[_assertion(True)])])
    assert jrn.compute_verdict(_spec("flow"), jr)[0] == "WORKS"


def test_verdict_flow_defects():
    jr = _jr("flow", [_step(assertions=[_assertion(True), _assertion(False)])])
    v, r = jrn.compute_verdict(_spec("flow"), jr)
    assert v == "WORKS-WITH-DEFECTS" and "1 failed assertion" in r


def test_verdict_flow_silent_no_op():
    jr = _jr("flow", [_step(effect=True, assertions=[_assertion(False)])])
    assert jrn.compute_verdict(_spec("flow"), jr)[0] == "SILENT-NO-OP"


def test_verdict_flow_misleading():
    jr = _jr("flow", [
        _step(n=1, effect=True, assertions=[_assertion(False)]),
        _step(n=2, feedback=True, assertions=[_assertion(True)]),
    ])
    assert jrn.compute_verdict(_spec("flow"), jr)[0] == "MISLEADING"


def test_verdict_gate_honest():
    jr = _jr("gate", [_step(gate=True, entry=True,
                            assertions=[_assertion(True)])])
    assert jrn.compute_verdict(_spec("gate"), jr)[0] == "UNAVAILABLE-BUT-HONEST"


def test_verdict_gate_silent_no_op():
    jr = _jr("gate", [_step(gate=True, entry=True,
                            assertions=[_assertion(False)])])
    assert jrn.compute_verdict(_spec("gate"), jr)[0] == "SILENT-NO-OP"


def test_verdict_gate_misleading_no_boundary():
    jr = _jr("gate", [
        _step(n=1, entry=True, assertions=[_assertion(True)]),
        _step(n=2, gate=True, assertions=[_assertion(False)]),
    ])
    assert jrn.compute_verdict(_spec("gate"), jr)[0] == "MISLEADING"


def _pal_step(pass_name, n, passed, entry=True):
    st = _step(pass_name=pass_name, n=n, action="palette_probe", entry=entry)
    st.assertions = [jrn.AssertionRecord("palette_entries", "x", "y", passed,
                                         "pil-region")]
    return st


def test_verdict_gate_missing_via_search():
    jr = _jr("gate", [_pal_step("primary", 1, False)])
    assert jrn.compute_verdict(_spec("gate"), jr)[0] == "MISSING"


def test_verdict_discovery_hidden():
    jr = _jr("discovery", [
        _pal_step("primary", 1, False),
        _pal_step("palette", 2, True),
    ])
    assert jrn.compute_verdict(_spec("discovery"), jr)[0] == "HIDDEN"


def test_verdict_discovery_missing():
    jr = _jr("discovery", [_pal_step("primary", 1, False),
                           _pal_step("palette", 2, False)])
    assert jrn.compute_verdict(_spec("discovery"), jr)[0] == "MISSING"


def test_verdict_discovery_silent_no_op_click():
    jr = _jr("discovery", [_step(entry=True, assertions=[_assertion(False)])])
    assert jrn.compute_verdict(_spec("discovery"), jr)[0] == "SILENT-NO-OP"


def test_verdict_discovery_click_responded_works():
    jr = _jr("discovery", [_step(entry=True, feedback=True,
                                 assertions=[_assertion(True)])])
    assert jrn.compute_verdict(_spec("discovery"), jr)[0] == "WORKS"


def test_verdict_discovery_search_unknown_not_run():
    jr = _jr("discovery", [_pal_step("primary", 1, None)])
    v, r = jrn.compute_verdict(_spec("discovery"), jr)
    assert v == "NOT-RUN" and "Pillow" in r


# ---------------------------------------------------------------------------
# 3. evidence naming determinism
def test_evidence_naming():
    import re as _re
    pat = _re.compile(r"^j\d{2}-step\d{2}-[a-z0-9-]+\.png$")
    for s in jspec.SPECS:
        seq1 = list(jrn.iter_journey_steps(s))
        seq2 = list(jrn.iter_journey_steps(s))
        assert seq1 == seq2, f"{s.jid}: plan not deterministic"
        names = [f for _, _, _, f in seq1]
        assert len(names) == len(set(names)), f"{s.jid}: duplicate evidence names"
        for f in names:
            assert pat.match(f), f"{s.jid}: bad evidence name {f}"
        # cold-start shots bookend each pass; numbering is continuous
        passes = [p for p, _, _, _ in seq1]
        assert passes[0] == "primary" and "palette" in passes
        ns = [n for _, n, _, _ in seq1]
        assert ns == sorted(ns) and ns[0] == 0 and ns[-1] == len(ns) - 1
        # primary steps come before palette steps
        assert passes == sorted(passes, key=lambda p: 0 if p == "primary" else 1)


# ---------------------------------------------------------------------------
# 4. matrix / ledger golden strings
def _golden_rr():
    spec = jspec.SPEC_BY_ID["J05"]
    jr = jrn.JourneyResult(
        jid="J05", title=spec.title, kind="discovery",
        expected_verdict="SILENT-NO-OP",
        expected_note="run-4: L-009 Projects '+' dead affordance",
        steps=[
            jrn.StepRecord(jid="J05", pass_name="primary", n=0,
                           slug="coldstart-primary", action="reset",
                           executed=True,
                           screenshot="j05-step00-coldstart-primary.png"),
            jrn.StepRecord(jid="J05", pass_name="primary", n=1,
                           slug="projects-plus", action="click",
                           executed=True, entry=True,
                           screenshot="j05-step01-projects-plus.png",
                           assertions=[jrn.AssertionRecord(
                               "region_change", "(290, 95, 995, 775)",
                               "x->y", False, "pil-region")]),
        ])
    return jrn.RunRecord(
        run_id="t-run", battery="baseline", commit="",
        actual_commit="abcdef1234567890", sandbox_id="sbx1",
        started="2026-09-21T10:00:00Z", finished="2026-09-21T10:05:00Z",
        e00=None,
        journeys=[(spec, jr, "SILENT-NO-OP",
                   "mapped control produced no visible response")],
        excluded=["J06"])


GOLDEN_MATRIX = """\
<!-- lab002:run:t-run:begin -->
### Run t-run — LAB-002 battery `baseline` (2026-09-21, E2B sandbox `sbx1`, Flauz @ `abcdef123456`, journeys 1/2)

Scripted by `scripts/e2b/journeys.py` (LAB-002). Verdicts are computed from per-step assertions (window title, calibrated palette differential, pixel-region probes). Wording-class defects are NOT pixel-assertable — review the per-step PNGs and `actions.jsonl` under `docs/linux-gui/evidence/t-run/`. Expected verdicts cite the Lead's run-4 ground truth; mismatches are flagged, not hidden.

| Journey | Verdict | Entries exercised (primary / palette) | Evidence |
| --- | --- | --- | --- |
| J05 Add an environment / project | **SILENT-NO-OP** (expected SILENT-NO-OP) | primary: Projects '+' affordance; palette: 'project' search | `j05-step00-coldstart-primary.png`, `j05-step01-projects-plus.png` |

Journeys excluded from this run (filter): J06 — not part of this sweep.

Evidence root: `docs/linux-gui/evidence/t-run/` — per-step PNGs (`jNN-stepNN-<slug>.png`), `actions.jsonl` (every input event + assertion result), `verdicts.json`, `state.json`.
<!-- lab002:run:t-run:end -->
"""

GOLDEN_LEDGER = """\
| B-t-run-J05 | 2026-09-21 | J05 Add an environment / project (battery t-run) | UNCLASSIFIED | LAB-002 scripted battery verdict SILENT-NO-OP for J05 Add an environment / project: mapped control produced no visible response. Failed assertions: 1. Matches run-4 ground truth (run-4: L-009 Projects '+' dead affordance). Class UNCLASSIFIED — Lead to classify (P0/P1/P2/P3/BOUNDED). | docs/linux-gui/evidence/t-run/ (actions.jsonl, j05-*.png) | — (Lead assigns) | — | OPEN (scripted finding) |"""


def test_matrix_emission_golden():
    got = jrn.render_matrix_section(_golden_rr())
    assert got == GOLDEN_MATRIX, \
        "matrix section mismatch:\n--- got ---\n" + got + "\n--- want ---\n" + GOLDEN_MATRIX


def test_ledger_emission_golden():
    rows = jrn.render_ledger_rows(_golden_rr())
    assert rows == [GOLDEN_LEDGER], f"ledger rows mismatch: {rows}"


def test_ledger_skips_non_defect_shapes():
    rr = _golden_rr()
    spec, jr, _, reason = rr.journeys[0]
    for v in ("WORKS", "UNAVAILABLE-BUT-HONEST", "NOT-RUN"):
        rr.journeys = [(spec, jr, v, reason)]
        assert jrn.render_ledger_rows(rr) == [], f"{v} must not emit a ledger row"


def test_matrix_apply_idempotent():
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "docs/linux-gui").mkdir(parents=True)
        mpath = root / jrn.MATRIX_REL
        mpath.write_text("# matrix\n\n## Run log\n")
        for _ in range(3):
            jrn.apply_matrix_section(root, GOLDEN_MATRIX, "t-run")
        text = mpath.read_text()
        assert text.count("lab002:run:t-run:begin") == 1
        assert text.count("### Run t-run") == 1
        assert text.startswith("# matrix")
        assert "## Run log" in text


def test_ledger_apply_idempotent():
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        lpath = root / jrn.LEDGER_REL
        lpath.parent.mkdir(parents=True)
        header = ("# ledger\n\n| ID | Date | Journey | Class | Summary | "
                  "Evidence | Work order | Fixed @ | Re-run verdict |\n"
                  "| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n"
                  "| L-001 | 2026-09-21 | E-00 | P1 | existing row | x | y | z | OPEN |")
        lpath.write_text(header)
        for _ in range(3):
            jrn.apply_ledger_rows(root, [GOLDEN_LEDGER], "t-run")
        text = lpath.read_text()
        assert text.count("| B-t-run-J05") == 1
        assert "| L-001" in text          # existing rows untouched
        assert text.rstrip("\n").endswith(GOLDEN_LEDGER)


# ---------------------------------------------------------------------------
# 5. dry-run (subprocess: proves no e2b import, no files, exit 0)
def test_dry_run_subprocess():
    code = ("import sys; sys.path.insert(0, %r); import journeys; "
            "rc = journeys.main(['--dry-run']); "
            "print('RC', rc); "
            "bad = [m for m in sys.modules if m.startswith('e2b') or "
            "m == 'flauz_e2b']; "
            "print('IMPORTED', bad); sys.exit(0 if rc == 0 and not bad else 1)"
            % str(HERE))
    r = subprocess.run([sys.executable, "-c", code], capture_output=True,
                       text=True, cwd=tempfile.gettempdir())
    assert r.returncode == 0, f"dry-run failed:\n{r.stdout}\n{r.stderr}"
    assert "IMPORTED []" in r.stdout
    for jid in (f"J{i:02d}" for i in range(1, 19)):
        assert jid in r.stdout, f"dry-run missing {jid}"
    assert "Phase E-00" in r.stdout
    assert "cold-start reset" in r.stdout
    assert "OK: 18 journeys planned" in r.stdout


def test_dry_run_subset_and_errors():
    r = subprocess.run([sys.executable, str(HERE / "journeys.py"), "--dry-run",
                        "--journeys", "j1,J5"], capture_output=True, text=True)
    assert r.returncode == 0
    assert "journeys: 2 selected" in r.stdout
    r = subprocess.run([sys.executable, str(HERE / "journeys.py"), "--dry-run",
                        "--journeys", "J99"], capture_output=True, text=True)
    assert r.returncode == 2 and "unknown journey id" in r.stdout


def test_cli_help_documents_every_flag():
    for flag in ("--battery", "--journeys", "--dry-run", "--run-id",
                 "--commit", "--baseline", "--repo-root"):
        assert flag in jrn.build_argparser().format_help(), f"{flag} missing from --help"
    r = subprocess.run([sys.executable, str(HERE / "journeys.py"), "--help"],
                       capture_output=True, text=True)
    assert r.returncode == 0 and "--battery" in r.stdout


# ---------------------------------------------------------------------------
# 6. full battery against the FakeSandbox (mock of the rc.14 surface)
# Verdicts the scripted battery computes on the mocked surface. J-04/J-14
# compute WORKS where the Lead judged WORKS-WITH-DEFECTS: wording-class
# defects (L-008/L-011) and auth-gated state preservation are not
# pixel-assertable — the mismatch flag + evidence carry that honestly.
MOCK_EXPECTED = {
    "E00": "WORKS",
    "J01": "WORKS", "J02": "UNAVAILABLE-BUT-HONEST",
    "J03": "WORKS-WITH-DEFECTS", "J04": "WORKS",
    "J05": "SILENT-NO-OP", "J06": "UNAVAILABLE-BUT-HONEST",
    "J07": "UNAVAILABLE-BUT-HONEST", "J08": "UNAVAILABLE-BUT-HONEST",
    "J09": "UNAVAILABLE-BUT-HONEST", "J10": "UNAVAILABLE-BUT-HONEST",
    "J11": "WORKS", "J12": "UNAVAILABLE-BUT-HONEST",
    "J13": "UNAVAILABLE-BUT-HONEST", "J14": "WORKS",
    "J15": "UNAVAILABLE-BUT-HONEST", "J16": "UNAVAILABLE-BUT-HONEST",
    "J17": "MISSING", "J18": "UNAVAILABLE-BUT-HONEST",
}


def _mkrepo(td: str) -> Path:
    root = Path(td)
    (root / "docs/linux-gui").mkdir(parents=True, exist_ok=True)
    (root / jrn.MATRIX_REL).write_text(
        "# Linux GUI — User Journey Matrix (J-01 … J-18)\n\n## Run log\n")
    (root / jrn.LEDGER_REL).write_text(
        "# Linux GUI — Regression Ledger\n\n"
        "| ID | Date | Journey | Class | Summary | Evidence | Work order | "
        "Fixed @ | Re-run verdict |\n| --- | --- | --- | --- | --- | --- | "
        "--- | --- | --- |\n")
    return root


def test_full_battery_mock():
    real_sleep = no_sleep()
    try:
        with tempfile.TemporaryDirectory() as td:
            root = _mkrepo(td)
            rr, fd, ctx = run_fake_battery(root, run_id="t-mock")
            # --- verdict table
            got = {"E00": rr.e00.verdict}
            for spec, jr, verdict, _ in rr.journeys:
                got[spec.jid] = verdict
            for jid, want in MOCK_EXPECTED.items():
                assert got.get(jid) == want, \
                    f"{jid}: computed {got.get(jid)}, mock-expect {want}"
            # --- evidence naming on disk
            evdir = root / jrn.EVID_SUBDIR / "t-mock"
            pngs = sorted(p.name for p in evdir.glob("j*-step*.png"))
            assert pngs and all(
                re.fullmatch(r"j\d{2}-step\d{2}-[a-z0-9-]+\.png", n)
                for n in pngs)
            assert (evdir / "e00-step01-launched.png").exists()
            assert (evdir / "e00-gui-log.txt").exists()
            assert (evdir / "verdicts.json").exists()
            assert (evdir / "run.json").exists()
            assert (evdir / "state.json").exists()
            # --- actions.jsonl: every line JSON, has ts+event, covers verbs
            lines = (evdir / "actions.jsonl").read_text().strip().split("\n")
            events = [json.loads(ln) for ln in lines]
            assert all("ts" in e and "event" in e for e in events)
            kinds = {e["event"] for e in events}
            for needed in ("press", "write", "left_click", "screenshot",
                           "assertion", "journey_start", "journey_verdict",
                           "pass_start", "run_start", "run_end",
                           "palette_calibration"):
                assert needed in kinds, f"actions.jsonl missing {needed} events"
            # every journey+pass start has a coldstart shot
            starts = [e for e in events if e["event"] == "pass_start"]
            assert len(starts) == 36, f"expected 36 pass starts, got {len(starts)}"
            # --- verdicts.json shape
            vj = json.loads((evdir / "verdicts.json").read_text())
            assert len(vj["journeys"]) == 18
            assert vj["journeys"]["J05"]["verdict"] == "SILENT-NO-OP"
            assert vj["journeys"]["J17"]["verdict"] == "MISSING"
            assert vj["journeys"]["J03"]["verdict"] == "WORKS-WITH-DEFECTS"
            assert vj["journeys"]["J03"]["failed_assertions"] >= 1
            # --- matrix + ledger applied
            mtext = (root / jrn.MATRIX_REL).read_text()
            assert mtext.count("### Run t-mock") == 1
            assert "**SILENT-NO-OP**" in mtext and "**MISSING**" in mtext
            assert "lab002:run:t-mock:begin" in mtext
            ltext = (root / jrn.LEDGER_REL).read_text()
            defect_rows = [ln for ln in ltext.split("\n") if ln.startswith("| B-t-mock-")]
            assert len(defect_rows) == 3, \
                f"expected 3 defect-shaped rows (J03/J05/J17), got {len(defect_rows)}"
            for jid in ("J03", "J05", "J17"):
                assert f"| B-t-mock-{jid} |" in ltext, f"missing ledger row {jid}"
            for jid in ("J01", "J02", "J11"):
                assert f"| B-t-mock-{jid} |" not in ltext, \
                    f"{jid} must not emit a ledger row (not defect-shaped)"
            # --- J-03 defect actually recorded (draft lost on restart)
            j03 = [e for e in events if e.get("journey") == "J03"
                   and e["event"] == "assertion" and e["kind"] == "region_stable"]
            assert j03 and all(a["passed"] is False for a in j03), \
                "J-03 draft-persistence probe must fail on the mocked L-010"
            # --- resume: second run with the same run-id skips completed work
            fd2 = FakeDesktop()
            rr2, _, _ = run_fake_battery(root, run_id="t-mock", desktop=fd2)
            assert fd2.relaunch_count == 0, "resume must not re-run journeys"
            assert fd2.sb.launch_count == 0, "resume must not relaunch the app"
            mtext2 = (root / jrn.MATRIX_REL).read_text()
            assert mtext2.count("### Run t-mock") == 1, \
                "matrix section must be replaced, not duplicated"
            vj2 = json.loads((root / jrn.EVID_SUBDIR / "t-mock" /
                              "verdicts.json").read_text())
            assert vj2["journeys"]["J05"]["verdict"] == "SILENT-NO-OP"
            # --- baseline compare: identical deterministic renders
            (root / jrn.EVID_SUBDIR / "t-base").mkdir(parents=True, exist_ok=True)
            for p in (root / jrn.EVID_SUBDIR / "t-mock").glob("j*-step*.png"):
                (root / jrn.EVID_SUBDIR / "t-base" / p.name).write_bytes(
                    p.read_bytes())
            rep = jrn.compare_baseline(root, "t-mock", "t-base")
            assert rep["totals"]["same"] == rep["totals"]["compared"] > 0
            # perturb one file -> exactly one difference
            some = sorted((root / jrn.EVID_SUBDIR / "t-base").glob("j*-step*.png"))[0]
            some.write_bytes(some.read_bytes() + b"\x00")
            rep2 = jrn.compare_baseline(root, "t-mock", "t-base")
            assert rep2["totals"]["different"] == 1
            assert (root / jrn.EVID_SUBDIR / "t-mock" /
                    "compare-t-base.json").exists()
    finally:
        sys.modules["time"].sleep = real_sleep


def test_palette_calibration_and_differential():
    real_sleep = no_sleep()
    try:
        with tempfile.TemporaryDirectory() as td:
            root = _mkrepo(td)
            rr, fd, _ = run_fake_battery(root, run_id="t-cal",
                                         journeys_arg="J17,J11")
            # calibration must be valid on the fake (query-independent empty state)
            evdir = root / jrn.EVID_SUBDIR / "t-cal"
            events = [json.loads(ln)
                      for ln in (evdir / "actions.jsonl").read_text().splitlines()]
            cal = [e for e in events if e["event"] == "palette_calibration"]
            assert cal and cal[0]["valid"] is True, cal
            # J-17 MISSING comes from the differential (no 'activity' entries)
            vj = json.loads((evdir / "verdicts.json").read_text())
            assert vj["journeys"]["J17"]["verdict"] == "MISSING"
            assert vj["journeys"]["J11"]["verdict"] == "WORKS"
            pal = [e for e in events if e["event"] == "assertion"
                   and e["kind"] == "palette_entries"]
            by_query = {e["detail"]: e["passed"] for e in pal}
            assert by_query.get("query='activity'") is False
            assert by_query.get("query='sidebar'") is True
    finally:
        sys.modules["time"].sleep = real_sleep


def test_no_pil_degradation():
    # region_stable skips, region_change falls back to full-image digest,
    # and search-only discovery degrades to NOT-RUN (never a false MISSING)
    real_sleep = no_sleep()
    orig_pil = jrn._PIL
    try:
        jrn._PIL = False
        d1, m1 = jrn.region_digest(b"aaa", (0, 0, 10, 10))
        d2, m2 = jrn.region_digest(b"bbb", (0, 0, 10, 10))
        assert m1 == m2 == "full-fallback"
        assert d1 != d2
        assert jrn.region_std(b"aaa", (0, 0, 10, 10)) is None
        jr = _jr("discovery", [_pal_step("primary", 1, None)])
        v, r = jrn.compute_verdict(_spec("discovery"), jr)
        assert v == "NOT-RUN" and "Pillow" in r
    finally:
        jrn._PIL = orig_pil
        sys.modules["time"].sleep = real_sleep


# ---------------------------------------------------------------------------
# 7. hygiene
def test_no_credentials_in_new_sources():
    # built dynamically so this file does not contain the literals itself
    forbidden = ("gh" + "p_", "x-access-" + "token", "PAYSWAP_" + "PAT",
                 "E2B_API_" + "KEY=")
    for name in ("journeys.py", "journey_specs.py", "tests.py"):
        src = (HERE / name).read_text()
        for tok in forbidden:
            assert tok not in src, f"{tok} found in {name}"


def test_flauz_e2b_importable_without_secrets():
    # LAB-002 additive guard: the harness module imports on stations without
    # ~/.secrets/env.sh (dry-run/test stations); live use still needs the env.
    import importlib
    mod = importlib.import_module("flauz_e2b")
    assert hasattr(mod, "FlauzDesktop")
    assert mod.CODEX_CLI_PIN == "0.146.0-alpha.3.1"


def test_state_json_resume_semantics():
    # a fresh sandbox id resets env_ok -> E-00 re-provisions on resume
    with tempfile.TemporaryDirectory() as td:
        root = _mkrepo(td)
        ctx = make_ctx(root, "t-state")
        ctx.state.update(env_ok=True, sandbox_id="old-sbx")
        ctx.save_state()
        ctx2 = make_ctx(root, "t-state")
        ctx2.sandbox_id = "new-sbx"     # live_desktop created a fresh sandbox
        assert not (ctx2.state.get("env_ok") and
                    ctx2.state.get("sandbox_id") == ctx2.sandbox_id), \
            "fresh sandbox must force E-00 re-provision"
        ctx.close()
        ctx2.close()


# ---------------------------------------------------------------------------
# runner (stdlib) — plain asserts, pytest-compatible
def main() -> int:
    global PASS, FAIL
    tests = [(n, f) for n, f in sorted(globals().items())
             if n.startswith("test_") and callable(f)]
    for name, fn in tests:
        try:
            fn()
            PASS += 1
            print(f"PASS {name}")
        except Exception as e:  # noqa: BLE001
            FAIL += 1
            import traceback
            print(f"FAIL {name}: {e}")
            traceback.print_exc()
    print(f"\n{PASS} passed, {FAIL} failed, {len(tests)} total")
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
