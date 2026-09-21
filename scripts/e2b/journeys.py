#!/usr/bin/env python3
"""LAB-002 — E2B journey battery runner for the Flauz Linux-GUI lane.

Reproduces the full J-01..J-18 cold-start sweep (docs/PRODUCT-UX-JOURNEYS.md)
on a fresh or existing E2B desktop, on top of the proven harness
``scripts/e2b/flauz_e2b.py`` (sandbox lifecycle, provisioning incl. the L-001
pipewire side-install and the automatic L-002 gpui visual patch, GUI launch —
extended, not rewritten).

Per journey (cold-start protocol, PRODUCT-UX-JOURNEYS §8): a primary pass
from the app's normal UI (relaunch reset, no palette first), then a palette
pass through the command palette as the search fallback. Every step captures
a screenshot (``<run-id>/jNN-stepNN-<slug>.png``) and every input event +
assertion result is appended to ``actions.jsonl``. Verdicts use the lane
vocabulary (WORKS / WORKS-WITH-DEFECTS / UNAVAILABLE-BUT-HONEST / MISSING /
HIDDEN / SILENT-NO-OP / MISLEADING / NOT-RUN) and are emitted
machine-readably (``verdicts.json``). A run section is appended to
docs/linux-gui/LINUX-GUI-USER-JOURNEY-MATRIX.md (idempotent per run-id) and
defect-shaped findings become UNCLASSIFIED rows in
docs/linux-gui/LINUX-GUI-REGRESSION-LEDGER.md (classification stays
Lead-owned). Signed-out auth boundaries are honest-unavailable outcomes, not
defects — they never emit ledger rows.

Resumability (E2B plan limits: 1 h create-timeout cap, ~40 min idle pause):
run state lives in ``docs/linux-gui/evidence/<run-id>/state.json`` — re-run
with the same ``--run-id`` to reconnect (or recreate) the sandbox and skip
journeys already completed.

Usage:
    python3 scripts/e2b/journeys.py --dry-run
    python3 scripts/e2b/journeys.py --battery baseline
    python3 scripts/e2b/journeys.py --journeys J01,J05 --battery baseline
    python3 scripts/e2b/journeys.py --run-id <id> --battery baseline
    python3 scripts/e2b/journeys.py --battery baseline --baseline <prev-id>

Credentials are read from the Lead station environment by ``flauz_e2b``
(``E2B_API_KEY``); nothing is ever committed, logged or placed in evidence.
Dry-run performs no network calls and never imports the e2b SDK.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import sys
import time
import traceback
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

sys.path.insert(0, str(Path(__file__).resolve().parent))
import journey_specs as jspec  # noqa: E402  (script dir on path)

# ---------------------------------------------------------------------------
# constants
VERDICTS = jspec.VERDICT_VOCAB
DEFECT_VERDICTS = {"WORKS-WITH-DEFECTS", "MISSING", "HIDDEN", "SILENT-NO-OP",
                   "MISLEADING"}
BATTERIES = {"baseline": "full J-01..J-18 cold-start sweep + E-00 environment"}

EVID_SUBDIR = "docs/linux-gui/evidence"
MATRIX_REL = "docs/linux-gui/LINUX-GUI-USER-JOURNEY-MATRIX.md"
LEDGER_REL = "docs/linux-gui/LINUX-GUI-REGRESSION-LEDGER.md"

R_MAIN = jspec.R_MAIN
R_PALETTE = jspec.R_PALETTE
PROMO_PRIMARY = jspec.UI["promo_modal_primary"]
PAL_COMBO = jspec.PALIN

SETTLE = {"click": 0.9, "double_click": 0.6, "right_click": 0.6, "type": 0.5,
          "key": 0.5, "scroll": 0.5, "drag": 0.6, "shot": 0.2, "wait": 0.0,
          "relaunch": 1.0, "palette_probe": 0.7}
JOURNEY_TIMEOUT_S = 900          # per journey, both passes
CONSECUTIVE_ABORT_LIMIT = 3      # stop the run after N consecutive failures
SANDBOX_TIMEOUT_S = 3600         # E2B plan hard cap (1 h) — refreshed per journey

try:  # optional pixel probes (work order: PIL when importable, else degrade)
    from PIL import Image, ImageStat  # type: ignore
    _PIL = True
except ImportError:  # pragma: no cover - depends on station
    Image = None  # type: ignore
    ImageStat = None  # type: ignore
    _PIL = False


def utcnow() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# pixel probe helpers (stdlib + optional PIL)
def _crop_box(rect):
    """(x, y, w, h) -> PIL crop box (x0, y0, x1, y1)."""
    x, y, w, h = rect
    return (x, y, x + w, y + h)


def region_digest(png: bytes, rect) -> tuple:
    """Deterministic digest of a screen region. Returns (digest, method)."""
    if _PIL:
        img = Image.open(io.BytesIO(png)).convert("L")
        crop = img.crop(_crop_box(rect))
        return hashlib.sha256(crop.tobytes()).hexdigest(), "pil-region"
    return hashlib.sha256(png).hexdigest(), "full-fallback"


def region_std(png: bytes, rect) -> Optional[float]:
    """Std-dev of a region's pixels (content presence); None without PIL."""
    if not _PIL:
        return None
    img = Image.open(io.BytesIO(png)).convert("L")
    return float(ImageStat.Stat(img.crop(_crop_box(rect))).stddev[0])


# ---------------------------------------------------------------------------
# records
@dataclass
class AssertionRecord:
    kind: str
    expected: str
    actual: str
    passed: Optional[bool]   # None = skipped (method unavailable)
    method: str
    detail: str = ""


@dataclass
class StepRecord:
    jid: str
    pass_name: str
    n: int
    slug: str
    action: str
    note: str = ""
    executed: bool = False
    error: Optional[str] = None
    screenshot: Optional[str] = None
    assertions: list = field(default_factory=list)
    # semantic flags mirrored from the spec step (verdict aggregation)
    entry: bool = False
    gate: bool = False
    effect: bool = False
    feedback: bool = False

    def any_passed(self) -> bool:
        return any(a.passed for a in self.assertions)


@dataclass
class JourneyResult:
    jid: str
    title: str
    kind: str
    expected_verdict: str
    expected_note: str
    steps: list = field(default_factory=list)
    aborted: Optional[str] = None
    verdict: Optional[str] = None
    reason: str = ""
    failed_override: Optional[int] = None  # resumed runs: count from state.json

    # ---- aggregates used by the verdict engine
    @property
    def failures(self) -> list:
        if self.failed_override is not None:
            return [None] * self.failed_override  # length-only (resumed)
        out = []
        for s in self.steps:
            out.extend(a for a in s.assertions if a.passed is False)
        return out

    def _flagged(self, name: str) -> list:
        return [s for s in self.steps if s.executed and getattr(s, name)]

    @property
    def entry_click(self) -> bool:
        return any(s.action in ("click", "double_click", "right_click")
                   for s in self._flagged("entry"))

    @property
    def entry_click_responded(self) -> bool:
        return any(s.any_passed() for s in self._flagged("entry")
                   if s.action in ("click", "double_click", "right_click"))

    @property
    def gate_seen(self) -> bool:
        return any(s.any_passed() for s in self._flagged("gate"))

    @property
    def effect_seen(self) -> bool:
        return any(s.any_passed() for s in self._flagged("effect"))

    @property
    def effect_expected(self) -> bool:
        return bool(self._flagged("effect"))

    @property
    def feedback_seen(self) -> bool:
        return any(s.any_passed() for s in self.steps
                   if s.executed and (s.feedback or s.gate))

    def search_entries(self, pass_name: str) -> Optional[bool]:
        """Palette differential result for a pass: True/False/None (unknown)."""
        vals = [a for s in self.steps if s.pass_name == pass_name
                for a in s.assertions if a.kind == "palette_entries"]
        if not vals:
            return None
        if any(a.passed is True for a in vals):
            return True
        if any(a.passed is False for a in vals):
            return False
        return None  # all skipped (calibration invalid / no PIL)

    def evidence_files(self) -> list:
        return [s.screenshot for s in self.steps if s.screenshot]


# ---------------------------------------------------------------------------
# verdict engine (deterministic; unit-tested against run-4 ground truth)
def compute_verdict(spec, jr: JourneyResult) -> tuple:
    """Returns (verdict, reason). Ordered rules — see tests.py for coverage."""
    if jr.aborted:
        return "NOT-RUN", jr.aborted
    if not jr.steps:
        return "NOT-RUN", "no steps executed"

    if spec.kind == "gate":
        if jr.gate_seen:
            return "UNAVAILABLE-BUT-HONEST", \
                "honest boundary observed (gate assertion passed; evidence captured)"
        if jr.entry_click:
            if not jr.entry_click_responded:
                return "SILENT-NO-OP", "mapped control produced no visible response"
            return "MISLEADING", "control responded without the expected honest boundary"
        sp, spa = jr.search_entries("primary"), jr.search_entries("palette")
        if sp is False or spa is False:
            return "MISSING", "no discoverable entry for this capability"
        return "NOT-RUN", "palette differential unavailable (install Pillow on " \
                          "the Lead station for full fidelity)"

    if spec.kind == "discovery":
        if jr.entry_click:
            if not jr.entry_click_responded:
                return "SILENT-NO-OP", "mapped control produced no visible response"
            return _post_entry(jr)
        sp, spa = jr.search_entries("primary"), jr.search_entries("palette")
        if sp is None and spa is None:
            return "NOT-RUN", "palette differential unavailable (install Pillow " \
                              "on the Lead station for full fidelity)"
        if sp is True:
            return _post_entry(jr)
        if spa is True:
            return "HIDDEN", "entry discoverable only through the command palette"
        return "MISSING", "no discoverable entry (sidebar inventory + palette search)"

    # flow
    return _post_entry(jr)


def _post_entry(jr: JourneyResult) -> tuple:
    if jr.effect_expected and not jr.effect_seen:
        if jr.feedback_seen:
            return "MISLEADING", \
                "UI feedback observed but the expected state effect did not occur"
        return "SILENT-NO-OP", "flow executed with no visible effect"
    if jr.failures:
        return "WORKS-WITH-DEFECTS", \
            f"{len(jr.failures)} failed assertion(s) — see actions.jsonl"
    return "WORKS", "all steps and assertions passed"


# ---------------------------------------------------------------------------
# evidence naming (deterministic — single source for dry-run and executor)
def evidence_name(jid: str, n: int, slug: str) -> str:
    return f"j{jid[1:].lower()}-step{n:02d}-{slug}.png"


def iter_journey_steps(spec):
    """Yield (pass_name, n, step_or_None, evidence_filename) in execution order.

    ``step_or_None`` marks the per-pass cold-start reset shot (step 0 for the
    primary pass; the palette pass continues the numbering — deterministic
    evidence naming across both passes).
    """
    n = 0
    yield "primary", n, None, evidence_name(spec.jid, n, "coldstart-primary")
    for st in spec.steps:
        n += 1
        yield "primary", n, st, evidence_name(spec.jid, n, st.slug)
    n += 1
    yield "palette", n, None, evidence_name(spec.jid, n, "coldstart-palette")
    for st in spec.palette_steps:
        n += 1
        yield "palette", n, st, evidence_name(spec.jid, n, st.slug)


# ---------------------------------------------------------------------------
# plan / dry-run
@dataclass
class Plan:
    battery: str
    run_id: str
    specs: list
    excluded: list
    commit: str


def default_run_id(battery: str) -> str:
    return f"{battery}-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"


def filter_specs(specs, journeys_arg: Optional[str]):
    if not journeys_arg:
        return list(specs), []
    want = set()
    for tok in journeys_arg.split(","):
        t = tok.strip().upper()
        if not t:
            continue
        if t.startswith("J") and t[1:].isdigit():
            want.add(f"J{int(t[1:]):02d}")
        elif t.isdigit():
            want.add(f"J{int(t):02d}")
        else:
            want.add(t)
    known = {s.jid for s in specs}
    unknown = want - known
    if unknown:
        raise ValueError(f"unknown journey id(s): {', '.join(sorted(unknown))}")
    return ([s for s in specs if s.jid in want],
            [s.jid for s in specs if s.jid not in want])


def build_plan(args, specs=None) -> Plan:
    specs = specs if specs is not None else jspec.load_specs()
    selected, excluded = filter_specs(specs, args.journeys)
    return Plan(battery=args.battery,
                run_id=args.run_id or default_run_id(args.battery),
                specs=selected, excluded=excluded, commit=args.commit)


def render_plan(plan: Plan) -> str:
    lines = []
    lines.append(f"LAB-002 journey battery — plan (battery: {plan.battery})")
    lines.append(f"run id: {plan.run_id}")
    lines.append(f"journeys: {len(plan.specs)} selected"
                 + (f", {len(plan.excluded)} excluded ({','.join(plan.excluded)})"
                    if plan.excluded else ""))
    lines.append(f"commit pin: "
                 f"{plan.commit or '(default: origin/main HEAD at provision time)'}")
    lines.append("pixel probes: "
                 + ("PIL available (region crops + calibrated palette differential)"
                    if _PIL else "PIL MISSING — region probes degrade (see --help)"))
    lines.append("")
    lines.append("Phase E-00 (environment setup — not a J-number):")
    lines.append("  provision(commit) → launch_gui → settle → screenshot")
    lines.append("  assertions: windows_nonempty (hard — blocks the battery),")
    lines.append("              window title contains 'codex' (soft),")
    lines.append("              main-area pixel std >= 6 (render check; soft)")
    lines.append("  evidence: e00-step01-launched.png, e00-gui-log.txt")
    lines.append("")
    for spec in plan.specs:
        lines.append(f"{spec.jid} {spec.title} [{spec.kind}] "
                     f"expected={spec.expected_verdict} — {spec.expected_note}")
        for pass_name, n, st, fname in iter_journey_steps(spec):
            if st is None:
                lines.append(f"  {pass_name:7s} step{n:02d} (cold-start reset: "
                             f"relaunch → Escape×2 → promo-primary → shot) → {fname}")
                continue
            parts = [f"  {pass_name:7s} step{n:02d} {describe_action(st)}"]
            asserts = describe_assertions(st)
            if asserts:
                parts.append(f"assert: {asserts}")
            flags = describe_flags(st)
            if flags:
                parts.append(f"[{flags}]")
            if st.note:
                parts.append(f"— {st.note}")
            lines.append(" ".join(parts) + f" → {fname}")
        lines.append("")
    return "\n".join(lines)


def describe_action(st) -> str:
    a = st.action
    if a in ("click", "double_click", "right_click"):
        return f"{a} ({st.x},{st.y})"
    if a == "type":
        return f"type {st.text!r}"
    if a == "key":
        k = st.key if isinstance(st.key, str) else "+".join(st.key or [])
        return f"key {k}"
    if a == "scroll":
        return f"scroll {st.direction} x{st.amount}"
    if a == "drag":
        return f"drag ({st.x},{st.y})->({st.x2},{st.y2})"
    if a == "wait":
        return f"wait {st.seconds}s"
    if a == "palette_probe":
        return f"palette_probe {st.text!r}"
    return a


def describe_assertions(st) -> str:
    out = []
    if st.expect_title:
        out.append(f"title~{st.expect_title!r}")
    if st.expect_windows:
        out.append(f"windows~{st.expect_windows!r}")
    if st.windows_nonempty:
        out.append("windows_nonempty")
    if st.region_change:
        out.append(f"region_change{st.region_change}")
    if st.region_stable:
        out.append(f"region_stable{st.region_stable}")
    if st.region_content:
        out.append(f"region_content{st.region_content[0]}>={st.region_content[1]}")
    if st.action == "palette_probe":
        out.append("palette_entries (calibrated differential)")
    return ", ".join(out)


def describe_flags(st) -> str:
    return "+".join(f for f in ("entry", "gate", "effect", "feedback")
                    if getattr(st, f))


# ---------------------------------------------------------------------------
# action log (actions.jsonl)
class ActionLog:
    def __init__(self, path: Path):
        self.path = path
        self._fh = open(path, "a", encoding="utf-8")

    def event(self, **kw):
        rec = {"ts": utcnow(), **kw}
        self._fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
        self._fh.flush()

    def close(self):
        try:
            self._fh.close()
        except Exception:
            pass


# ---------------------------------------------------------------------------
# run context (evidence dir + resumable state.json)
class RunContext:
    def __init__(self, repo_root: Path, run_id: str, battery: str, commit: str = ""):
        self.repo_root = repo_root
        self.run_id = run_id
        self.battery = battery
        self.commit = commit
        self.evdir = repo_root / EVID_SUBDIR / run_id
        self.evdir.mkdir(parents=True, exist_ok=True)
        self.log = ActionLog(self.evdir / "actions.jsonl")
        self.state_path = self.evdir / "state.json"
        self.state = self._load_state()
        self.started = utcnow()
        self.sandbox_id = self.state.get("sandbox_id", "")
        self.actual_commit = self.state.get("actual_commit", "")

    def _load_state(self) -> dict:
        if self.state_path.exists():
            try:
                return json.loads(self.state_path.read_text())
            except Exception:
                pass
        return {"run_id": self.run_id, "battery": self.battery,
                "journeys": {}, "env_ok": False}

    def save_state(self):
        self.state.update({"run_id": self.run_id, "battery": self.battery,
                           "updated": utcnow()})
        if self.sandbox_id:
            self.state["sandbox_id"] = self.sandbox_id
        if self.actual_commit:
            self.state["actual_commit"] = self.actual_commit
        tmp = self.state_path.with_suffix(".tmp")
        tmp.write_text(json.dumps(self.state, indent=2))
        tmp.replace(self.state_path)

    def journey_state(self, jid: str) -> dict:
        return self.state.setdefault("journeys", {}).setdefault(jid, {})

    def close(self):
        self.log.close()


# ---------------------------------------------------------------------------
# executor — driver-agnostic (live adapter or the test fake)
class BatteryExecutor:
    def __init__(self, desktop, ctx: RunContext):
        self.fd = desktop            # duck-typed: .sb, .relaunch(), .provision(), .gui_log()
        self.ctx = ctx
        self._last_png: Optional[bytes] = None
        self._palette_calib: Optional[dict] = None

    # -- primitives -------------------------------------------------------
    def _sb(self):
        return self.fd.sb

    def _park(self):
        try:
            self._sb().move_mouse(1600, 40)
        except Exception:
            pass

    def _shot_bytes(self) -> bytes:
        return self._sb().screenshot(format="bytes")

    def _save_png(self, name: str, data: bytes) -> str:
        (self.ctx.evdir / name).write_bytes(data)
        self.ctx.log.event(run_id=self.ctx.run_id, event="screenshot", file=name,
                           sha256=hashlib.sha256(data).hexdigest(), bytes=len(data))
        return name

    def _press(self, key, jid="", n=0, slug=""):
        k = list(key) if isinstance(key, (tuple, list)) else key
        self._sb().press(k)
        self.ctx.log.event(run_id=self.ctx.run_id, journey=jid, step=n, slug=slug,
                           event="press",
                           key=(k if isinstance(k, str) else list(k)))

    def refresh_timeout(self):
        try:
            self._sb().set_timeout(SANDBOX_TIMEOUT_S)
        except Exception:
            pass

    # -- cold-start reset (per pass) ---------------------------------------
    def reset(self, jid: str, pass_name: str, n: int, fname: str) -> StepRecord:
        """Cold start from the app's normal UI: relaunch, calm the surface,
        park the pointer, screenshot. No palette (protocol §8)."""
        rec = StepRecord(jid=jid, pass_name=pass_name, n=n,
                         slug=f"coldstart-{pass_name}", action="reset")
        self.ctx.log.event(run_id=self.ctx.run_id, journey=jid, phase=pass_name,
                           event="pass_start",
                           note="cold-start reset (relaunch, no palette)")
        self.fd.relaunch()
        for _ in range(2):
            self._press("Escape", jid=jid, n=n, slug=rec.slug)
            time.sleep(0.3)
        # L-012: the first-run promo modal's only dismiss affordance is its
        # primary button; the click is a harmless no-op when no modal is up.
        self._sb().move_mouse(PROMO_PRIMARY[0], PROMO_PRIMARY[1])
        self._sb().left_click()
        self.ctx.log.event(run_id=self.ctx.run_id, journey=jid, step=n,
                           slug=rec.slug, event="left_click",
                           x=PROMO_PRIMARY[0], y=PROMO_PRIMARY[1],
                           note="promo-modal primary (L-012 dismiss path; "
                                "no-op when absent)")
        time.sleep(SETTLE["click"])
        self._park()
        time.sleep(0.2)
        png = self._shot_bytes()
        rec.screenshot = self._save_png(fname, png)
        rec.executed = True
        self._last_png = png
        return rec

    # -- palette differential ----------------------------------------------
    def calibrate_palette(self) -> dict:
        """One-time calibration of the entries-differential method.

        Opens the palette with two nonsense terms and one known-present term.
        Valid iff the two nonsense states are pixel-identical AND differ from
        the known-present state — this catches 'no results' states that echo
        the query (which would defeat a plain differential).
        """
        if self._palette_calib is not None:
            return self._palette_calib
        digests = {}
        method = "full-fallback"
        for i, term in enumerate(("zzzz", "qqqq", "settings")):
            self._press(PAL_COMBO)
            time.sleep(0.5)
            self._sb().write(term)
            time.sleep(0.7)
            self._park()
            png = self._shot_bytes()
            self._save_png(f"cal-palette-{i}-{term}.png", png)
            digests[term] = region_digest(png, R_PALETTE)
            method = digests[term][1]
            self._press("Escape")
            time.sleep(0.3)
        dz, dq, ds = digests["zzzz"][0], digests["qqqq"][0], digests["settings"][0]
        if dz == dq and dz != ds:
            cal = {"valid": True, "baseline": dz, "reason":
                   "differential discriminates (empty==empty != known-present)"}
        elif dz != dq:
            cal = {"valid": False, "baseline": None, "reason":
                   "empty palette states differ between nonsense terms (query "
                   "echo or animation) — differential unusable"}
        else:
            cal = {"valid": False, "baseline": None, "reason":
                   "known-present term matches the empty state — palette may "
                   "not render or the term vanished"}
        self._palette_calib = cal
        self.ctx.log.event(run_id=self.ctx.run_id, event="palette_calibration",
                           valid=cal["valid"], reason=cal["reason"], method=method)
        return cal

    def do_palette_probe(self, st, rec: StepRecord) -> AssertionRecord:
        """Open the palette, type the query, capture evidence and evaluate the
        calibrated entries differential. The palette is left OPEN with the
        query typed (the spec may press Return next, or Escape)."""
        cal = self.calibrate_palette()
        self._press(PAL_COMBO)
        time.sleep(0.5)
        self._sb().write(st.text)
        time.sleep(SETTLE["palette_probe"])
        self._park()
        png = self._shot_bytes()
        rec.screenshot = self._save_png(evidence_name(rec.jid, rec.n, st.slug), png)
        self._last_png = png
        rec.executed = True
        digest, method = region_digest(png, R_PALETTE)
        if not cal["valid"]:
            arec = AssertionRecord("palette_entries", "entries != empty baseline",
                                   "unknown", None, method,
                                   f"calibration invalid: {cal['reason']}")
        else:
            passed = digest != cal["baseline"]
            arec = AssertionRecord("palette_entries", "entries != empty baseline",
                                   "present" if passed else "absent", passed,
                                   method, f"query={st.text!r}")
        self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                           phase=rec.pass_name, step=rec.n, slug=st.slug,
                           event="assertion", kind=arec.kind,
                           expected=arec.expected, actual=arec.actual,
                           passed=arec.passed, method=arec.method,
                           detail=arec.detail)
        return arec

    # -- assertion hooks ----------------------------------------------------
    def _eval_assertions(self, st, rec: StepRecord, prev_png: bytes):
        out = []
        png = self._last_png or b""
        if st.expect_title:
            try:
                title = self._sb().get_window_title() or ""
            except Exception as e:
                title = f"(title query failed: {e})"
            out.append(AssertionRecord(
                "title_contains", st.expect_title, str(title)[:120],
                st.expect_title.lower() in str(title).lower(), "window-title"))
        if st.expect_windows or st.windows_nonempty:
            try:
                wins = self._sb().get_application_windows()
                wins_s = str(wins)
            except Exception as e:
                wins, wins_s = [], f"(windows query failed: {e})"
            if st.windows_nonempty:
                out.append(AssertionRecord("windows_nonempty", ">=1 window",
                                           f"{len(wins)} window(s)", bool(wins),
                                           "app-windows"))
            if st.expect_windows:
                out.append(AssertionRecord(
                    "windows_include", st.expect_windows, wins_s[:120],
                    st.expect_windows.lower() in wins_s.lower(), "app-windows"))
        if st.region_change or st.region_stable:
            rect = st.region_change or st.region_stable
            cur_d, method = region_digest(png, rect)
            if not prev_png:
                out.append(AssertionRecord(
                    "region_" + ("change" if st.region_change else "stable"),
                    str(rect), "no previous shot", None, method,
                    "skipped: no baseline shot"))
            else:
                prev_d, _ = region_digest(prev_png, rect)
                if st.region_change:
                    out.append(AssertionRecord(
                        "region_change", str(rect),
                        f"{prev_d[:8]}…->{cur_d[:8]}…", cur_d != prev_d, method))
                elif method == "full-fallback":
                    out.append(AssertionRecord(
                        "region_stable", str(rect), "n/a", None, method,
                        "skipped: locality needs PIL"))
                else:
                    out.append(AssertionRecord(
                        "region_stable", str(rect),
                        f"{prev_d[:8]}…=={cur_d[:8]}…", cur_d == prev_d, method))
        if st.region_content:
            rect, min_std = st.region_content
            std = region_std(png, rect)
            if std is None:
                out.append(AssertionRecord(
                    "region_content", f"std>={min_std}", "n/a", None, "no-PIL",
                    "skipped: content probe needs PIL"))
            else:
                out.append(AssertionRecord(
                    "region_content", f"std>={min_std}", f"std={std:.1f}",
                    std >= min_std, "pil-stddev"))
        for a in out:
            self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                               phase=rec.pass_name, step=rec.n, slug=rec.slug,
                               event="assertion", kind=a.kind, expected=a.expected,
                               actual=a.actual, passed=a.passed, method=a.method,
                               detail=a.detail)
        return out

    # -- step execution -----------------------------------------------------
    def execute_step(self, st, rec: StepRecord) -> Optional[str]:
        """Run one step. Returns an abort reason (or None)."""
        a = st.action
        sb = self._sb()
        try:
            if a == "palette_probe":
                rec.assertions.append(self.do_palette_probe(st, rec))
                rec.executed = True
                return self._finish_step(st, rec, already_shot=True)
            if a in ("click", "double_click", "right_click"):
                sb.move_mouse(st.x, st.y)
                # declarative "click" = the SDK's left_click()
                getattr(sb, "left_click" if a == "click" else a)()
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event=a, x=st.x, y=st.y)
            elif a == "type":
                sb.write(st.text)
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event="write", text=st.text)
            elif a == "key":
                self._press(st.key, jid=rec.jid, n=rec.n, slug=st.slug)
            elif a == "scroll":
                sb.scroll(direction=st.direction, amount=st.amount)
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event="scroll", direction=st.direction,
                                   amount=st.amount)
            elif a == "drag":
                sb.drag(st.x, st.y, st.x2, st.y2)
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event="drag", x1=st.x, y1=st.y, x2=st.x2,
                                   y2=st.y2)
            elif a == "relaunch":
                self.fd.relaunch()
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event="relaunch")
            elif a == "wait":
                self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                                   phase=rec.pass_name, step=rec.n, slug=st.slug,
                                   event="wait", seconds=st.seconds)
            elif a == "shot":
                pass
            else:  # pragma: no cover - validated earlier
                raise ValueError(f"unhandled action {a}")
            time.sleep(SETTLE.get(a, 0.5) if a != "wait" else st.seconds)
            rec.executed = True
            return self._finish_step(st, rec)
        except Exception as e:  # sandbox death / API error
            rec.error = f"{type(e).__name__}: {str(e)[:200]}"
            rec.executed = True
            self.ctx.log.event(run_id=self.ctx.run_id, journey=rec.jid,
                               phase=rec.pass_name, step=rec.n, slug=st.slug,
                               event="step_error", error=rec.error)
            return f"step {rec.n} ({st.slug}) raised: {rec.error}"

    def _finish_step(self, st, rec: StepRecord, already_shot: bool = False
                     ) -> Optional[str]:
        """Post-action screenshot + assertion hooks. Returns abort reason."""
        if already_shot:
            prev = b""  # palette probe shot already captured; probes unused there
        else:
            self._park()
            time.sleep(0.15)
            png = self._shot_bytes()
            rec.screenshot = self._save_png(
                evidence_name(rec.jid, rec.n, st.slug), png)
            prev, self._last_png = self._last_png or b"", png
        rec.assertions.extend(self._eval_assertions(st, rec, prev))
        for a in rec.assertions:
            if a.passed is False and st.on_fail == "abort":
                return (f"step {rec.n} ({st.slug}) hard assertion failed: "
                        f"{a.kind}")
        return None

    # -- journey ------------------------------------------------------------
    def run_journey(self, spec) -> JourneyResult:
        jr = JourneyResult(jid=spec.jid, title=spec.title, kind=spec.kind,
                           expected_verdict=spec.expected_verdict,
                           expected_note=spec.expected_note)
        self.refresh_timeout()
        self.ctx.log.event(run_id=self.ctx.run_id, journey=spec.jid,
                           event="journey_start", title=spec.title, kind=spec.kind)
        t0 = time.time()
        self._last_png = None
        for pass_name, n, st, fname in iter_journey_steps(spec):
            if time.time() - t0 > JOURNEY_TIMEOUT_S:
                jr.aborted = f"journey timeout after {JOURNEY_TIMEOUT_S}s"
                break
            if st is None:
                jr.steps.append(self.reset(spec.jid, pass_name, n, fname))
                continue
            rec = StepRecord(jid=spec.jid, pass_name=pass_name, n=n, slug=st.slug,
                             action=st.action, note=st.note, entry=st.entry,
                             gate=st.gate, effect=st.effect, feedback=st.feedback)
            jr.steps.append(rec)
            abort = self.execute_step(st, rec)
            if abort:
                jr.aborted = abort
                break
        jr.verdict, jr.reason = compute_verdict(spec, jr)
        self.ctx.log.event(run_id=self.ctx.run_id, journey=spec.jid,
                           event="journey_verdict", verdict=jr.verdict,
                           reason=jr.reason, expected=spec.expected_verdict,
                           seconds=round(time.time() - t0, 1))
        return jr

    # -- E-00 environment phase ----------------------------------------------
    def run_e00(self, commit: str) -> JourneyResult:
        """E-00 fresh desktop → provision → build → launch → window appears.
        Not a J-number: the setup phase that gates the battery."""
        jr = JourneyResult(jid="E00",
                           title="environment (provision → build → launch → window)",
                           kind="flow", expected_verdict="WORKS",
                           expected_note="L-001/L-002/L-004 accommodations are "
                                         "built into flauz_e2b provision")
        t0 = time.time()
        self.ctx.log.event(run_id=self.ctx.run_id, journey="E00", event="e00_start",
                           commit=commit or "(main HEAD)")
        try:
            cur = self.fd.provision(commit)
            if cur:
                self.ctx.actual_commit = cur
        except Exception as e:
            jr.aborted = f"provision failed: {type(e).__name__}: {str(e)[:300]}"
            jr.verdict, jr.reason = compute_verdict(_E00_SPEC, jr)
            return jr
        rec = StepRecord(jid="E00", pass_name="primary", n=1, slug="launched",
                         action="relaunch")
        jr.steps.append(rec)
        self.fd.relaunch()
        time.sleep(3.0)
        self._park()
        try:
            png = self._shot_bytes()
        except Exception as e:
            jr.aborted = f"launch screenshot failed: {e}"
            jr.verdict, jr.reason = compute_verdict(_E00_SPEC, jr)
            return jr
        rec.screenshot = self._save_png("e00-step01-launched.png", png)
        rec.executed = True
        self._last_png = png
        rec.assertions.extend(self._eval_assertions(_E00_ASSERT_STEP, rec, b""))
        try:
            (self.ctx.evdir / "e00-gui-log.txt").write_text(
                self.fd.gui_log(60) or "")
        except Exception:
            pass
        # windows_nonempty is the hard gate for the whole battery
        for a in rec.assertions:
            if a.kind == "windows_nonempty" and a.passed is False:
                jr.aborted = "no application window after launch (E-00 failed)"
        jr.verdict, jr.reason = compute_verdict(_E00_SPEC, jr)
        self.ctx.log.event(run_id=self.ctx.run_id, journey="E00",
                           event="journey_verdict", verdict=jr.verdict,
                           reason=jr.reason,
                           seconds=round(time.time() - t0, 1))
        return jr


class _E00AssertStep:
    """Assertion-only pseudo-step for the E-00 launch check."""

    def __init__(self):
        self.expect_title = "codex"        # window class com.codexrs.CodexRS
        self.expect_windows = ""
        self.windows_nonempty = True       # hard gate (enforced in run_e00)
        self.region_change = None
        self.region_stable = None
        self.region_content = (R_MAIN, 6.0)  # render check (L-002 signal)
        self.on_fail = "defect"


class _E00Spec:
    jid = "E00"
    kind = "flow"


_E00_ASSERT_STEP = _E00AssertStep()
_E00_SPEC = _E00Spec()


# ---------------------------------------------------------------------------
# run orchestration
@dataclass
class RunRecord:
    run_id: str
    battery: str
    commit: str
    actual_commit: str
    sandbox_id: str
    started: str
    finished: str = ""
    e00: Optional[JourneyResult] = None
    journeys: list = field(default_factory=list)  # (spec, jr, verdict, reason)
    excluded: list = field(default_factory=list)


def execute_battery(desktop, ctx: RunContext, plan: Plan) -> RunRecord:
    """Drive the whole battery. ``desktop`` is a FlauzDesktop-like adapter
    (live) or the test fake."""
    rr = RunRecord(run_id=ctx.run_id, battery=ctx.battery, commit=plan.commit,
                   actual_commit=ctx.actual_commit, sandbox_id=ctx.sandbox_id,
                   started=ctx.started, excluded=list(plan.excluded))
    ex = BatteryExecutor(desktop, ctx)
    ctx.log.event(run_id=ctx.run_id, event="run_start", battery=ctx.battery,
                  commit=plan.commit or "(main HEAD)",
                  resumed=bool(ctx.state.get("env_ok")))

    # E-00 — environment gate (per sandbox; a fresh sandbox re-provisions)
    if ctx.state.get("env_ok") and ctx.state.get("sandbox_id") == ctx.sandbox_id \
            and ctx.sandbox_id:
        e00 = JourneyResult(jid="E00",
                            title="environment (provision → build → launch → window)",
                            kind="flow", expected_verdict="WORKS",
                            expected_note="resumed",
                            verdict="WORKS",
                            reason="already provisioned (resumed sandbox)")
        print("[battery] E-00: skipped (sandbox already provisioned)")
    else:
        e00 = ex.run_e00(plan.commit)
        if e00.aborted:
            rr.e00 = e00
            ctx.save_state()
            _finish(rr, ctx, plan, blocked=e00.aborted)
            return rr
        ctx.state["env_ok"] = True
        ctx.save_state()
    rr.e00 = e00
    print(f"[battery] E-00: {e00.verdict} — {e00.reason}")

    consecutive = 0
    for spec in plan.specs:
        js = ctx.journey_state(spec.jid)
        if js.get("status") == "done":
            jr = JourneyResult(jid=spec.jid, title=spec.title, kind=spec.kind,
                               expected_verdict=spec.expected_verdict,
                               expected_note=spec.expected_note,
                               failed_override=js.get("failed", 0))
            for i, fname in enumerate(js.get("evidence", [])):
                jr.steps.append(StepRecord(
                    jid=spec.jid, pass_name="resumed", n=i,
                    slug=fname[:-4], action="resumed", executed=True,
                    screenshot=fname))
            rr.journeys.append((spec, jr, js.get("verdict", "NOT-RUN"),
                                js.get("reason", "resumed from state.json")))
            print(f"[battery] {spec.jid}: skipped "
                  f"(already {js.get('verdict')})")
            continue
        try:
            jr = ex.run_journey(spec)
            consecutive = 0
        except Exception as e:
            traceback.print_exc()
            jr = JourneyResult(jid=spec.jid, title=spec.title, kind=spec.kind,
                               expected_verdict=spec.expected_verdict,
                               expected_note=spec.expected_note,
                               aborted=f"executor error: {type(e).__name__}: "
                                       f"{str(e)[:200]}")
            consecutive += 1
        verdict, reason = compute_verdict(spec, jr)
        jr.verdict, jr.reason = verdict, reason
        ctx.journey_state(spec.jid).update(
            status="done", verdict=verdict, reason=reason,
            evidence=jr.evidence_files(), failed=len(jr.failures))
        ctx.save_state()
        rr.journeys.append((spec, jr, verdict, reason))
        print(f"[battery] {spec.jid}: {verdict}"
              + ("" if verdict == spec.expected_verdict
                 else f"  (expected {spec.expected_verdict})"))
        if consecutive >= CONSECUTIVE_ABORT_LIMIT:
            print(f"[battery] aborting run: {CONSECUTIVE_ABORT_LIMIT} consecutive "
                  f"executor failures")
            break
    _finish(rr, ctx, plan)
    return rr


def _finish(rr: RunRecord, ctx: RunContext, plan: Plan, blocked: str = None):
    rr.finished = utcnow()
    rr.actual_commit = ctx.actual_commit
    rr.sandbox_id = ctx.sandbox_id
    if blocked:
        for spec in plan.specs:
            if any(j[0].jid == spec.jid for j in rr.journeys):
                continue
            jr = JourneyResult(jid=spec.jid, title=spec.title, kind=spec.kind,
                               expected_verdict=spec.expected_verdict,
                               expected_note=spec.expected_note,
                               aborted=f"blocked by E-00: {blocked}")
            rr.journeys.append((spec, jr, "NOT-RUN", jr.aborted))
    # machine-readable verdict per journey
    verdicts = {
        "run_id": rr.run_id, "battery": rr.battery, "started": rr.started,
        "finished": rr.finished, "commit_requested": rr.commit,
        "commit_actual": rr.actual_commit, "sandbox_id": rr.sandbox_id,
        "e00": _jr_json(rr.e00) if rr.e00 else None,
        "journeys": {},
    }
    for spec, jr, verdict, reason in rr.journeys:
        verdicts["journeys"][spec.jid] = {
            "title": spec.title, "kind": spec.kind, "verdict": verdict,
            "reason": reason, "expected_verdict": spec.expected_verdict,
            "matches_expected": verdict == spec.expected_verdict,
            "expected_note": spec.expected_note, "entries": spec.entries,
            "evidence": jr.evidence_files(),
            "failed_assertions": len(jr.failures), "aborted": jr.aborted,
        }
    (ctx.evdir / "verdicts.json").write_text(json.dumps(verdicts, indent=2))
    (ctx.evdir / "run.json").write_text(json.dumps({
        "run_id": rr.run_id, "battery": rr.battery, "started": rr.started,
        "finished": rr.finished, "commit_requested": rr.commit,
        "commit_actual": rr.actual_commit, "sandbox_id": rr.sandbox_id,
        "excluded": rr.excluded, "pil": _PIL,
    }, indent=2))
    ctx.log.event(run_id=ctx.run_id, event="run_end", finished=rr.finished,
                  journeys=len(rr.journeys))
    # matrix + ledger (idempotent per run-id)
    apply_matrix_section(ctx.repo_root, render_matrix_section(rr), rr.run_id)
    apply_ledger_rows(ctx.repo_root, render_ledger_rows(rr), rr.run_id)
    ctx.save_state()


def _jr_json(jr: JourneyResult) -> dict:
    return {"verdict": jr.verdict, "reason": jr.reason, "aborted": jr.aborted,
            "steps": [{"n": s.n, "slug": s.slug, "action": s.action,
                       "screenshot": s.screenshot,
                       "assertions": [a.__dict__ for a in s.assertions]}
                      for s in jr.steps]}


# ---------------------------------------------------------------------------
# matrix / ledger emission (golden-string tested)
def _verdict_cell(spec, verdict: str) -> str:
    if verdict == spec.expected_verdict:
        return f"**{verdict}** (expected {spec.expected_verdict})"
    return (f"**{verdict}** (expected {spec.expected_verdict} — "
            f"mismatch flagged for Lead review)")


def render_matrix_section(rr: RunRecord) -> str:
    date = rr.started[:10]
    commit = (rr.actual_commit or rr.commit or "main HEAD")[:12]
    ran = len(rr.journeys)
    total = ran + len(rr.excluded)
    L = []
    L.append(f"<!-- lab002:run:{rr.run_id}:begin -->")
    L.append(f"### Run {rr.run_id} — LAB-002 battery `{rr.battery}` "
             f"({date}, E2B sandbox `{rr.sandbox_id or 'n/a'}`, Flauz @ "
             f"`{commit}`, journeys {ran}/{total})")
    L.append("")
    L.append("Scripted by `scripts/e2b/journeys.py` (LAB-002). Verdicts are "
             "computed from per-step assertions (window title, calibrated "
             "palette differential, pixel-region probes). Wording-class defects "
             "are NOT pixel-assertable — review the per-step PNGs and "
             f"`actions.jsonl` under `{EVID_SUBDIR}/{rr.run_id}/`. Expected "
             "verdicts cite the Lead's run-4 ground truth; mismatches are "
             "flagged, not hidden.")
    L.append("")
    L.append("| Journey | Verdict | Entries exercised (primary / palette) | Evidence |")
    L.append("| --- | --- | --- | --- |")
    if rr.e00 is not None:
        ev = rr.e00.evidence_files()[:2]
        L.append(f"| E-00 environment (provision → build → launch → window) | "
                 f"**{rr.e00.verdict}** | provision, launch, window-appears | "
                 f"{', '.join(f'`{e}`' for e in ev) or '—'} |")
    for spec, jr, verdict, reason in rr.journeys:
        evs = jr.evidence_files()
        ev_s = ", ".join(f"`{e}`" for e in evs[:3])
        if len(evs) > 3:
            ev_s += f" +{len(evs) - 3} more"
        entries = spec.entries or "—"
        L.append(f"| {spec.jid} {spec.title} | {_verdict_cell(spec, verdict)} "
                 f"| {entries} | {ev_s or '—'} |")
    if rr.excluded:
        L.append("")
        L.append(f"Journeys excluded from this run (filter): "
                 f"{', '.join(rr.excluded)} — not part of this sweep.")
    L.append("")
    L.append(f"Evidence root: `{EVID_SUBDIR}/{rr.run_id}/` — per-step PNGs "
             f"(`jNN-stepNN-<slug>.png`), `actions.jsonl` (every input event + "
             f"assertion result), `verdicts.json`, `state.json`.")
    L.append(f"<!-- lab002:run:{rr.run_id}:end -->")
    return "\n".join(L) + "\n"


def _ledger_summary(spec, jr, verdict: str, reason: str) -> str:
    base = (f"LAB-002 scripted battery verdict {verdict} for {spec.jid} "
            f"{spec.title}: {reason}.")
    fails = len(jr.failures)
    extra = f" Failed assertions: {fails}." if fails else ""
    if verdict == spec.expected_verdict:
        exp = f" Matches run-4 ground truth ({spec.expected_note})."
    else:
        exp = (f" Expected per run-4 ground truth: {spec.expected_verdict} "
               f"({spec.expected_note}).")
    return (base + extra + exp +
            " Class UNCLASSIFIED — Lead to classify (P0/P1/P2/P3/BOUNDED).")


def render_ledger_rows(rr: RunRecord) -> list:
    date = rr.started[:10]
    rows = []
    for spec, jr, verdict, reason in rr.journeys:
        if verdict not in DEFECT_VERDICTS:
            continue  # honest-unavailable / works / not-run are not defect-shaped
        rid = f"B-{rr.run_id}-{spec.jid}"
        summary = _ledger_summary(spec, jr, verdict, reason)
        ev = f"{EVID_SUBDIR}/{rr.run_id}/ (actions.jsonl, {spec.jid.lower()}-*.png)"
        rows.append(f"| {rid} | {date} | {spec.jid} {spec.title} "
                    f"(battery {rr.run_id}) | UNCLASSIFIED | {summary} | {ev} | "
                    f"— (Lead assigns) | — | OPEN (scripted finding) |")
    return rows


def apply_matrix_section(repo_root: Path, section: str, run_id: str):
    path = repo_root / MATRIX_REL
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("# Linux GUI — User Journey Matrix (J-01 … J-18)\n\n"
                        "## Run log\n")
    text = path.read_text()
    begin = f"<!-- lab002:run:{run_id}:begin -->"
    end = f"<!-- lab002:run:{run_id}:end -->"
    if begin in text and end in text:
        pre = text[:text.index(begin)]
        post = text[text.index(end) + len(end):].lstrip("\n")
        text = pre + section.rstrip("\n") + "\n" + post
        if not text.endswith("\n"):
            text += "\n"
    else:
        text = text.rstrip("\n") + "\n\n" + section
    path.write_text(text)


def apply_ledger_rows(repo_root: Path, rows: list, run_id: str):
    if not rows:
        return
    path = repo_root / LEDGER_REL
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("# Linux GUI — Regression Ledger\n\n"
                        "| ID | Date | Journey | Class | Summary | Evidence | "
                        "Work order | Fixed @ | Re-run verdict |\n"
                        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n")
    text = path.read_text().rstrip("\n")
    # idempotency: drop this run's previously emitted rows
    text = "\n".join(ln for ln in text.split("\n")
                     if not (ln.startswith("| B-") and run_id in ln))
    for r in rows:
        text += "\n" + r
    path.write_text(text + "\n")


# ---------------------------------------------------------------------------
# baseline regression compare (--baseline)
def compare_baseline(repo_root: Path, run_id: str, baseline_id: str) -> dict:
    cur = repo_root / EVID_SUBDIR / run_id
    base = repo_root / EVID_SUBDIR / baseline_id
    report = {"run_id": run_id, "baseline_id": baseline_id,
              "note": "deterministic sha256+size compare of per-step PNGs; "
                      "differences are review signals, not verdicts",
              "journeys": {}}
    files = sorted(p.name for p in cur.glob("j*-step*.png"))
    same = diff = missing = 0
    for name in files:
        jid = name.split("-")[0]
        cbytes = (cur / name).read_bytes()
        bpath = base / name
        if not bpath.exists():
            missing += 1
            report["journeys"].setdefault(jid, {})[name] = {"same": None}
            continue
        bbytes = bpath.read_bytes()
        entry = {"same": cbytes == bbytes,
                 "cur": {"sha256": hashlib.sha256(cbytes).hexdigest()[:16],
                         "bytes": len(cbytes)},
                 "base": {"sha256": hashlib.sha256(bbytes).hexdigest()[:16],
                          "bytes": len(bbytes)}}
        report["journeys"].setdefault(jid, {})[name] = entry
        same += bool(entry["same"])
        diff += (not entry["same"])
    report["totals"] = {"compared": len(files), "same": same, "different": diff,
                        "missing_in_baseline": missing}
    (cur / f"compare-{baseline_id}.json").write_text(json.dumps(report, indent=2))
    return report


# ---------------------------------------------------------------------------
# live session (lazy imports — dry-run/tests never touch the e2b SDK)
class LiveDesktop:
    """Adapter over flauz_e2b.FlauzDesktop for the battery executor."""

    def __init__(self, fd):
        self.fd = fd
        self.sb = fd.sb
        self.sid = fd.sid

    def relaunch(self):
        self.fd.launch_gui()
        time.sleep(4.0)  # window settle (harness practice: sleep 3 + 4)

    def provision(self, commit: str) -> str:
        return self.fd.provision(commit)

    def gui_log(self, tail: int = 30) -> str:
        return self.fd.gui_log(tail)


def live_desktop(ctx: RunContext):
    """Connect to the run's sandbox or create a fresh one. Returns LiveDesktop."""
    import flauz_e2b  # noqa: WPS433 (deliberate lazy import)
    sid = ctx.state.get("sandbox_id")
    if sid:
        try:
            from e2b_desktop import Sandbox  # noqa: WPS433
            sb = Sandbox.connect(sid, timeout=SANDBOX_TIMEOUT_S)
            print(f"[e2b] reconnected: {sid}")
            ctx.sandbox_id = sid
            return LiveDesktop(flauz_e2b.FlauzDesktop(sb, sid))
        except Exception as e:
            print(f"[e2b] reconnect failed ({str(e)[:140]}) — creating a fresh "
                  f"sandbox")
    fd = flauz_e2b.FlauzDesktop.create(timeout_s=SANDBOX_TIMEOUT_S)
    ctx.sandbox_id = fd.sid
    ctx.state["env_ok"] = False  # fresh sandbox: E-00 must re-provision
    return LiveDesktop(fd)


# ---------------------------------------------------------------------------
# CLI
def build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="journeys.py",
        description="LAB-002 — E2B journey battery (J-01..J-18 runner, evidence "
                    "pipeline, matrix population) for the Flauz Linux-GUI lane. "
                    "Runs on the Lead station with E2B_API_KEY in the "
                    "environment; --dry-run works anywhere (no E2B, no network, "
                    "no files).")
    p.add_argument("--battery", default="baseline", choices=sorted(BATTERIES),
                   help="battery to run (default: baseline = the full "
                        "J-01..J-18 cold-start sweep + E-00 environment phase)")
    p.add_argument("--journeys", default="",
                   help="comma-separated journey subset, e.g. J01,J05 "
                        "(default: all)")
    p.add_argument("--dry-run", action="store_true",
                   help="no E2B, no network, no files: emit the planned step "
                        "list with planned evidence names and validate every "
                        "spec (exit 0 iff all specs parse)")
    p.add_argument("--run-id", default="",
                   help="evidence run id (default: <battery>-<UTC timestamp>). "
                        "Re-running with the same id RESUMES: state.json "
                        "reconnects the sandbox (or recreates it) and skips "
                        "completed journeys")
    p.add_argument("--commit", default="",
                   help="pin the provisioned Flauz commit (default: current "
                        "origin/main HEAD at provision time)")
    p.add_argument("--baseline", default="", metavar="RUN_ID",
                   help="regression mode: after the run, compare per-step "
                        "screenshots against a previous run-id (sha256+size; "
                        "report written into the run dir)")
    p.add_argument("--repo-root", default="",
                   help="advanced/testing: repository root (default: the repo "
                        "containing this script)")
    return p


def main(argv=None, stdout=None) -> int:
    stdout = stdout or sys.stdout
    args = build_argparser().parse_args(argv)

    errs = jspec.validate_all()
    if errs:
        stdout.write("SPEC VALIDATION FAILED:\n")
        for e in errs:
            stdout.write(f"  - {e}\n")
        return 1

    if not args.run_id:
        args.run_id = default_run_id(args.battery)
    if not re.fullmatch(r"[A-Za-z0-9._-]+", args.run_id):
        stdout.write(f"error: invalid run id {args.run_id!r}\n")
        return 2
    if args.baseline and args.baseline == args.run_id:
        stdout.write("error: --baseline must reference a different run id\n")
        return 2

    specs = jspec.load_specs()
    try:
        plan = build_plan(args, specs)
    except ValueError as e:
        stdout.write(f"error: {e}\n")
        return 2

    if args.dry_run:
        stdout.write(render_plan(plan) + "\n")
        stdout.write(f"OK: {len(plan.specs)} journeys planned, "
                     f"{sum(len(s.steps) + len(s.palette_steps) for s in plan.specs)} "
                     f"spec steps (+ per-pass cold-start resets), all specs valid.\n")
        return 0

    repo_root = Path(args.repo_root).resolve() if args.repo_root else \
        Path(__file__).resolve().parent.parent.parent
    ctx = RunContext(repo_root=repo_root, run_id=args.run_id,
                     battery=args.battery, commit=args.commit)
    try:
        fd = live_desktop(ctx)
        rr = execute_battery(fd, ctx, plan)
    finally:
        ctx.close()

    # summary
    stdout.write("\n=== LAB-002 battery summary ===\n")
    stdout.write(f"run id: {rr.run_id}  (evidence: {EVID_SUBDIR}/{rr.run_id}/)\n")
    if rr.e00 is not None:
        stdout.write(f"E-00: {rr.e00.verdict} — {rr.e00.reason}\n")
    not_run = 0
    for spec, jr, verdict, reason in rr.journeys:
        flag = "" if verdict == spec.expected_verdict else \
            f"  <- expected {spec.expected_verdict}"
        stdout.write(f"  {spec.jid} {verdict:24s} {reason[:70]}{flag}\n")
        if verdict == "NOT-RUN":
            not_run += 1
    stdout.write(f"matrix: {MATRIX_REL} (run section appended, idempotent)\n")
    stdout.write(f"ledger: {LEDGER_REL} (UNCLASSIFIED rows for defect-shaped "
                 f"verdicts)\n")

    if args.baseline:
        rep = compare_baseline(repo_root, args.run_id, args.baseline)
        t = rep["totals"]
        stdout.write(f"baseline compare vs {args.baseline}: {t['same']} same / "
                     f"{t['different']} different / {t['missing_in_baseline']} "
                     f"missing -> {EVID_SUBDIR}/{args.run_id}/"
                     f"compare-{args.baseline}.json\n")

    return 1 if not_run else 0


if __name__ == "__main__":
    sys.exit(main())
