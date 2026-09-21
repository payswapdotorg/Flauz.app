#!/usr/bin/env python3
"""LAB-002 — declarative journey specs for the Flauz E2B journey battery.

One spec per journey J-01..J-18 from docs/PRODUCT-UX-JOURNEYS.md, written
against the rc.14 Codex-parity surface (Workspace/chat composer/terminal/
browser/Settings/palette/file search — see docs/parity-matrix.md + F1
closure). Executed by scripts/e2b/journeys.py.

Cold-start protocol (PRODUCT-UX-JOURNEYS §8, LINUX-GUI-USER-JOURNEY-MATRIX
header): every journey runs as TWO passes —
  * primary pass  — starts from the app's normal UI (relaunch reset, NO
    command palette first) and drives the primary visible entry;
  * palette pass  — resets again from the normal UI, then reaches the same
    journey goal through the command palette (search fallback).
`journeys.py` enforces this structurally: the reset happens per pass and no
step in `steps` (primary) may use the palette.

Expected verdicts record the Lead's run-4 ground truth (signed-out, patched
build, docs/linux-gui/LINUX-GUI-USER-JOURNEY-MATRIX.md "Run 4"). They are
cross-checked, never enforced: a mismatch is written to the matrix for Lead
review. Signed-out auth boundaries are HONEST-UNAVAILABLE outcomes, not
defects.

Coordinates are pinned to the UI map below, recorded from run-3/run-4
pixel-verified E2B evidence. Run-4 lesson: unverified/VLM coordinates drift
15-60 px on dense rows — every interactive step therefore also carries a
pixel-region probe so drift surfaces as a failed assertion in actions.jsonl
(evidence for the Lead) instead of a silent miss. Tune the UI map in ONE
place if the layout shifts.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Tuple

Rect = Tuple[int, int, int, int]  # x, y, w, h inside the 1920x1080 screen

# ---------------------------------------------------------------------------
# UI map — E2B desktop 1920x1080, app window ~ (4,85)-(1289,878).
# Row y-centres verified by pixel-row scans of run3-j01-* / run4-* evidence.
UI: dict = {
    "screen": (1920, 1080),
    "window": (4, 85, 1289, 878),
    "sidebar_x": 60,                 # click column inside sidebar rows
    "rows": {                        # sidebar row y-centres
        "new_chat": 116,
        "repository": 152,
        "pull_requests": 186,
        "plugins": 224,
        "workflows": 259,
        "projects_header": 314,      # "+" affordance at x=252 (L-009)
        "projects_plus": (252, 314),
        "chats_header": 349,
        "terminal": 745,
        "browser": 775,
        "settings": 822,
    },
    "composer_input": (700, 740),    # composer text area
    "composer_pill_model": (855, 822),   # model selector pill (run-3)
    "promo_modal_primary": (778, 560),   # L-012: only dismiss path (also switches model)
    "settings_back": (95, 112),      # "Back to app" (run-3 VLM+pixel)
    "settings_category": (430, 260), # category row (best estimate — probe-guarded)
    "regions": {
        "main": (290, 95, 995, 775),      # main content area
        "composer": (300, 699, 985, 170), # composer box
        "center": (450, 250, 400, 400),   # centred dialogs / sign-in gate
        "palette_list": (410, 335, 480, 380),  # palette entries (below search input)
        "sidebar": (10, 95, 275, 780),    # sidebar column
    },
}

R_MAIN = UI["regions"]["main"]
R_COMPOSER = UI["regions"]["composer"]
R_CENTER = UI["regions"]["center"]
R_PALETTE = UI["regions"]["palette_list"]
R_SIDEBAR = UI["regions"]["sidebar"]


# ---------------------------------------------------------------------------
# Step / spec model
@dataclass(frozen=True)
class Step:
    """One journey step: an interaction plus optional assertion hooks.

    Assertions (all soft by default — a failure is recorded as a defect and
    the journey continues; ``on_fail="abort"`` marks hard failures):
      * ``expect_title``      — focused window title contains (case-insensitive)
      * ``expect_windows``    — some application-window title contains
      * ``windows_nonempty``  — at least one application window exists
      * ``region_change``     — rect differs from the previous step's shot
      * ``region_stable``     — rect identical to the previous step's shot
      * ``region_content``    — (rect, min_std) rect pixel std-dev >= min_std
      * ``palette_probe`` (action) — differential entries assertion (see
        journeys.py: calibrated against a nonsense-query baseline)
    """
    action: str                    # click|double_click|right_click|type|key|
                                   # scroll|drag|shot|wait|relaunch|palette_probe
    slug: str = ""
    x: int = -1
    y: int = -1
    x2: int = -1
    y2: int = -1
    text: str = ""
    key: object = None             # str or tuple[str, ...] (combo)
    direction: str = "down"
    amount: int = 3
    seconds: float = 0.6
    note: str = ""
    # semantic flags (verdict computation)
    entry: bool = False            # entry-discovery step (click = mapped control,
                                   # palette_probe = search entry)
    gate: bool = False             # honest-boundary observation step
    effect: bool = False           # asserts the journey's real state effect
    feedback: bool = False         # expected to produce visible UI feedback
    # assertion hooks
    expect_title: str = ""
    expect_windows: str = ""
    windows_nonempty: bool = False
    region_change: Optional[Rect] = None
    region_stable: Optional[Rect] = None
    region_content: Optional[Tuple[Rect, float]] = None
    on_fail: str = "defect"        # "defect" | "abort"

    def __post_init__(self) -> None:
        if not self.slug:
            object.__setattr__(self, "slug", _default_slug(self))


@dataclass(frozen=True)
class JourneySpec:
    jid: str                       # "J01" .. "J18"
    title: str
    kind: str                      # "flow" | "discovery" | "gate"
    steps: Tuple[Step, ...]        # primary pass (normal UI, no palette)
    palette_steps: Tuple[Step, ...]  # palette pass (search fallback)
    expected_verdict: str          # run-4 ground truth (cross-checked, not enforced)
    expected_note: str = ""
    entries: str = ""              # entries exercised (for the matrix row)
    docs_ref: str = ""


ACTIONS = {"click", "double_click", "right_click", "type", "key", "scroll",
           "drag", "shot", "wait", "relaunch", "palette_probe"}

VERDICT_VOCAB = {"WORKS", "WORKS-WITH-DEFECTS", "UNAVAILABLE-BUT-HONEST",
                 "MISSING", "HIDDEN", "SILENT-NO-OP", "MISLEADING", "NOT-RUN"}

KINDS = {"flow", "discovery", "gate"}

PALIN = ["ctrl", "shift", "p"]     # command palette combo (run-3 verified)


def _default_slug(step: Step) -> str:
    a = step.action
    if a in ("click", "double_click", "right_click"):
        return f"{a}-{step.x}-{step.y}"
    if a == "key":
        k = step.key if isinstance(step.key, str) else "-".join(step.key or [])
        return f"key-{k}"
    if a == "palette_probe":
        return f"palette-{step.text.replace(' ', '-')}"
    return a


def _sid(name: str) -> int:
    return UI["rows"][name]


def validate_spec(spec: JourneySpec) -> list:
    """Structural validation. Returns a list of error strings (empty = ok)."""
    errs = []
    if spec.jid[0] != "J" or not spec.jid[1:].isdigit():
        errs.append(f"{spec.jid}: bad id")
    if spec.kind not in KINDS:
        errs.append(f"{spec.jid}: bad kind {spec.kind!r}")
    if spec.expected_verdict not in VERDICT_VOCAB:
        errs.append(f"{spec.jid}: bad expected_verdict {spec.expected_verdict!r}")
    if not spec.steps or not spec.palette_steps:
        errs.append(f"{spec.jid}: both passes must have steps (cold-start protocol)")
    sw, sh = UI["screen"]
    seen_slugs = set()
    for label, seq in (("primary", spec.steps), ("palette", spec.palette_steps)):
        for i, st in enumerate(seq):
            where = f"{spec.jid}.{label}[{i}]"
            if st.action not in ACTIONS:
                errs.append(f"{where}: unknown action {st.action!r}")
            if st.on_fail not in ("defect", "abort"):
                errs.append(f"{where}: bad on_fail {st.on_fail!r}")
            for cname in ("region_change", "region_stable"):
                r = getattr(st, cname)
                if r and (len(r) != 4 or r[0] < 0 or r[1] < 0 or
                          r[0] + r[2] > sw or r[1] + r[3] > sh):
                    errs.append(f"{where}: {cname} rect {r} outside screen")
            if st.region_content:
                r = st.region_content[0]
                if r[0] < 0 or r[1] < 0 or r[0] + r[2] > sw or r[1] + r[3] > sh:
                    errs.append(f"{where}: region_content rect {r} outside screen")
            if st.action in ("click", "double_click", "right_click"):
                if not (0 <= st.x < sw and 0 <= st.y < sh):
                    errs.append(f"{where}: click ({st.x},{st.y}) outside screen")
            if st.action == "palette_probe" and not st.text:
                errs.append(f"{where}: palette_probe needs a query")
            if st.key is not None and not isinstance(st.key, (str, tuple, list)):
                errs.append(f"{where}: key must be str or tuple")
            if st.slug in seen_slugs:
                errs.append(f"{where}: duplicate slug {st.slug!r}")
            seen_slugs.add(st.slug)
            # cold-start protocol: primary pass never uses the palette
            if label == "primary":
                if st.action == "palette_probe":
                    errs.append(f"{where}: palette action in primary pass "
                                "(cold-start protocol: no palette first)")
                if isinstance(st.key, (tuple, list)) and \
                        [k.lower() for k in st.key] == PALIN:
                    errs.append(f"{where}: palette combo in primary pass")
    return errs


# ---------------------------------------------------------------------------
# The 18 journey specs (rc.14 surface, signed-out).
#
# Expected verdicts = Lead's run-4 ground truth (2026-09-21, sandbox
# izqinqfuxeylk815bn5sf, patched build): J-01 WORKS; J-02 auth-gated honest;
# J-03 WORKS-WITH-DEFECTS (L-010 draft lost); J-04 WORKS-WITH-DEFECTS
# (L-008/L-011 wording classes); J-05 SILENT-NO-OP (L-009); J-06..J-10,
# J-12, J-13, J-15, J-16, J-18 auth-gated honest; J-11 WORKS; J-14
# WORKS-WITH-DEFECTS; J-17 MISSING (no Activity surface signed-out).

SPECS = (
    # --- J-01 Start a project -------------------------------------------
    # Expected: WORKS (run-3: composer keyboard input verified; run-4: full
    # shell renders, honest signed-out gate with 4 auth paths; L-012 promo
    # modal noted but does not break the journey).
    JourneySpec(
        jid="J01", title="Start a project (cold start)", kind="flow",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-01",
        expected_verdict="WORKS",
        expected_note="run-4: composer input + honest signed-out gate verified",
        entries="primary: promo modal, composer, send; palette: 'new chat'",
        steps=(
            Step("click", x=UI["promo_modal_primary"][0], y=UI["promo_modal_primary"][1],
                 slug="dismiss-promo-primary",
                 note="L-012: promo modal's only dismiss affordance; also switches "
                      "the composer model (honest action, recorded for J-14)"),
            Step("click", x=UI["composer_input"][0], y=UI["composer_input"][1],
                 slug="focus-composer", entry=True,
                 note="task composer = primary entry for starting work"),
            Step("type", text="J-01 battery probe: plan a small weekend hike",
                 slug="type-objective", effect=True, region_change=R_COMPOSER,
                 note="objective in natural language (no software terminology)"),
            Step("key", key="Return", slug="send-objective", gate=True, feedback=True,
                 region_change=R_MAIN,
                 note="send attempt -> honest signed-out gate expected "
                      "(Sign in with ChatGPT / device code / API key / Bedrock)"),
            Step("key", key="Escape", slug="close-gate",
                 note="dismiss gate/overlay if it responds to Escape"),
        ),
        palette_steps=(
            Step("palette_probe", text="new chat", slug="palette-newchat", entry=True,
                 note="search fallback for starting a project (J-11 inventory: "
                      "'New chat Ctrl+N' is a Suggested palette entry)"),
            Step("key", key="Return", slug="palette-newchat-enter", gate=True,
                 feedback=True, region_change=R_MAIN,
                 note="execute highlighted entry -> new-chat view / honest gate"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-02 Understand what the agent knows (Context) -----------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: no Context surface reachable
    # signed-out; hosting surfaces show the honest sign-in gate). Future-
    # platform journey — probing honestly IS the result.
    JourneySpec(
        jid="J02", title="Understand what the agent knows (Context)", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-02",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; no Context surface signed-out",
        entries="primary: task-start gate; palette: 'context' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="a task must exist before Context is inspectable; signed-out "
                      "this shows the honest 'Sign in to get started' gate (run-4)"),
            Step("shot", slug="gate-evidence",
                 note="capture gate state for Lead review (auth-path wording)"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="context", slug="palette-context",
                 note="search fallback for a Context entry (expected: none "
                      "signed-out — recorded honestly)"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-03 Recover/continue a long-running task -----------------------
    # Expected: WORKS-WITH-DEFECTS (run-4: sidebar state persists across
    # restart but the typed composer draft is LOST with no resume prompt —
    # L-010. In-task resume needs auth, operator-gated).
    JourneySpec(
        jid="J03", title="Recover a task after restart", kind="flow",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-03",
        expected_verdict="WORKS-WITH-DEFECTS",
        expected_note="run-4: L-010 draft lost on restart; sidebar persists",
        entries="primary: composer draft + restart; palette: 'search chats'",
        steps=(
            Step("click", x=UI["composer_input"][0], y=UI["composer_input"][1],
                 slug="focus-composer", entry=True),
            Step("type", text="J-03 draft persistence probe", slug="type-draft",
                 effect=True, region_change=R_COMPOSER,
                 note="unsent work that should survive an interruption"),
            Step("relaunch", slug="restart-app", effect=True,
                 region_stable=R_COMPOSER,
                 note="L-010 expected: draft LOST on restart (run-4) — this "
                      "probe_stable is expected to FAIL and register the defect"),
            Step("shot", slug="after-restart-evidence",
                 note="composer state after restart for Lead review"),
        ),
        palette_steps=(
            Step("palette_probe", text="search chats", slug="palette-searchchats",
                 note="recovery search path (J-11 inventory: 'Search chats Ctrl+G')"),
            Step("key", key="Return", slug="palette-searchchats-enter", feedback=True,
                 region_change=R_MAIN,
                 note="chat search opens; empty state expected signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-04 Discover a capability gap ----------------------------------
    # Expected: WORKS-WITH-DEFECTS (run-4: Repository honest empty state,
    # Pull requests honest dependency error, Plugins render, Workflows raw
    # protocol error L-008, Terminal/Browser honest toasts L-006-corrected,
    # toasts pinned L-011). NOTE: wording-class defects (L-008/L-011) are
    # NOT pixel-assertable — probes verify response presence only; the
    # scripted verdict may compute WORKS where the Lead judged W-W-D. The
    # mismatch flag + PNG evidence carry that honestly to the Lead.
    JourneySpec(
        jid="J04", title="Discover a capability gap (sidebar sweep)", kind="flow",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-04",
        expected_verdict="WORKS-WITH-DEFECTS",
        expected_note="run-4: L-008 raw -32600 error + L-011 pinned toasts "
                      "(wording classes — Lead reviews evidence)",
        entries="primary: all sidebar tools; palette: 'workflows'",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("repository"), slug="open-repository",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="honest empty state 'No Git repository detected' (run-4)"),
            Step("click", x=UI["sidebar_x"], y=_sid("pull_requests"), slug="open-pullrequests",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="honest dependency error 'GitHub CLI setup required' + the "
                      "pinned gh toast (L-011) — evidence for Lead review"),
            Step("click", x=UI["sidebar_x"], y=_sid("plugins"), slug="open-plugins",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="marketplace renders (run-4)"),
            Step("click", x=UI["sidebar_x"], y=_sid("workflows"), slug="open-workflows",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="L-008: 'Durable instances' raw -32600 error banner — "
                      "wording not pixel-assertable, evidence captured"),
            Step("click", x=UI["sidebar_x"], y=_sid("terminal"), slug="open-terminal",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="honest toast 'Select a task before opening a terminal.' "
                      "(L-006 corrected run-4)"),
            Step("click", x=UI["sidebar_x"], y=_sid("browser"), slug="open-browser",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="honest toast 'Open a chat before opening the Browser.'"),
            Step("click", x=UI["sidebar_x"], y=_sid("settings"), slug="open-settings",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="Settings fully navigable (run-3 verified)"),
            Step("click", x=UI["settings_back"][0], y=UI["settings_back"][1],
                 slug="back-to-app", feedback=True, region_change=R_MAIN,
                 note="'Back to app' returns to the normal UI"),
        ),
        palette_steps=(
            Step("palette_probe", text="workflows", slug="palette-workflows",
                 note="search fallback for the capability surface"),
            Step("key", key="Return", slug="palette-workflows-enter", feedback=True,
                 region_change=R_MAIN),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-05 Add an environment / project -------------------------------
    # Expected: SILENT-NO-OP (run-4: the sidebar 'Projects +' affordance
    # does nothing visible signed-out — L-009; environment-adding surfaces
    # live in Settings and render). The '+' control is mapped (exists) but
    # produces no dialog, no form, no toast.
    JourneySpec(
        jid="J05", title="Add an environment / project", kind="discovery",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-05",
        expected_verdict="SILENT-NO-OP",
        expected_note="run-4: L-009 Projects '+' dead affordance",
        entries="primary: Projects '+' affordance; palette: 'project' search",
        steps=(
            Step("click", x=UI["rows"]["projects_plus"][0],
                 y=UI["rows"]["projects_plus"][1], slug="projects-plus", entry=True,
                 feedback=True, region_change=R_MAIN,
                 note="L-009: visible control, no visible response — the exact "
                      "SILENT-NO-OP class the lane forbids"),
            Step("click", x=UI["rows"]["projects_plus"][0],
                 y=UI["rows"]["projects_plus"][1], slug="projects-plus-retry",
                 note="single vs double verification (run-4 tested both)"),
            Step("shot", slug="silent-state-evidence",
                 note="no dialog / inline form / toast — evidence for the ledger"),
        ),
        palette_steps=(
            Step("palette_probe", text="project", slug="palette-project",
                 note="search fallback (may match project picker commands — "
                      "recorded; primary dead affordance keeps priority)"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-06 Coordinate browser + terminal + sandbox --------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: cross-surface coordination
    # needs a live task; Terminal/Browser honest task-required toasts).
    JourneySpec(
        jid="J06", title="Coordinate browser + terminal + sandbox", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-06",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; honest task-required toasts",
        entries="primary: Terminal + Browser tools; palette: 'terminal'",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("terminal"), slug="open-terminal",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="honest toast 'Select a task before opening a terminal.'"),
            Step("click", x=UI["sidebar_x"], y=_sid("browser"), slug="open-browser",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="honest toast 'Open a chat before opening the Browser.'"),
            Step("shot", slug="toast-evidence"),
            Step("key", key="Escape", slug="dismiss"),
        ),
        palette_steps=(
            Step("palette_probe", text="terminal", slug="palette-terminal"),
            Step("key", key="Return", slug="palette-terminal-enter", gate=True,
                 feedback=True, region_change=R_MAIN),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-07 Parallelize work (Agents) ----------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: parallel agents need a live
    # model-backed task; no Agents surface signed-out).
    JourneySpec(
        jid="J07", title="Parallelize work (Agents)", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-07",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; no Agents surface signed-out",
        entries="primary: task-start gate; palette: 'agent' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="agents participate in tasks; signed-out shows the honest gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="agent", slug="palette-agent",
                 note="expected: no Agents entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-08 Human takeover / handoff -----------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: takeover needs a running task).
    JourneySpec(
        jid="J08", title="Human takeover / handoff", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-08",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; takeover requires a running task",
        entries="primary: task-start gate; palette: 'take over' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="takeover state is task-scoped; signed-out shows the gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="take over", slug="palette-takeover",
                 note="expected: no takeover entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-09 Understand what happened (Evidence) ------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: evidence states need a task).
    JourneySpec(
        jid="J09", title="Understand what happened (Evidence)", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-09",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; evidence is task-scoped",
        entries="primary: task-start gate; palette: 'evidence' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="evidence views are task-scoped; signed-out shows the gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="evidence", slug="palette-evidence",
                 note="expected: no Evidence entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-10 Save a successful task as a Procedure ----------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: needs a successful task;
    # primary copy 'Save as a reusable workflow' is post-success contextual).
    JourneySpec(
        jid="J10", title="Save a successful task (reusable workflow)", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-10",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; save-as-workflow is post-success",
        entries="primary: task-start gate; palette: 'reusable' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="'Save as a reusable workflow' appears post-success; "
                      "signed-out shows the gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="reusable", slug="palette-reusable",
                 note="expected: no save-as-workflow entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-11 Discover saved work later ----------------------------------
    # Expected: WORKS (run-4: >=2 discovery paths verified — persistent
    # sidebar sections, command palette with working search, keyboard
    # shortcuts, first-run promo modal).
    JourneySpec(
        jid="J11", title="Discover saved work later", kind="discovery",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-11",
        expected_verdict="WORKS",
        expected_note="run-4: sidebar + palette + shortcuts verified",
        entries="primary: persistent sidebar + Workflows surface; palette: "
                "'sidebar' search + Ctrl+G",
        steps=(
            Step("shot", slug="sidebar-persistent",
                 region_content=(R_SIDEBAR, 8.0),
                 note="discovery path 1: persistent sidebar with sections "
                      "(saved-work surfaces stay visible)"),
            Step("click", x=UI["sidebar_x"], y=_sid("workflows"), slug="open-workflows",
                 entry=True, feedback=True, region_change=R_MAIN,
                 note="a saved-work surface opens from persistent navigation"),
        ),
        palette_steps=(
            Step("palette_probe", text="sidebar", slug="palette-sidebar",
                 note="discovery path 2: palette search 'sidebar' -> 'Toggle "
                      "sidebar (Ctrl+B)' (run-4 verified). NOT executed — "
                      "toggling would collapse the sidebar (L-007 hazard)"),
            Step("key", key="Escape", slug="palette-escape"),
            Step("key", key=("ctrl", "g"), slug="ctrl-g-search", feedback=True,
                 region_change=R_MAIN,
                 note="discovery path 3: keyboard shortcut 'Search chats Ctrl+G' "
                      "(J-11 inventory) — empty state expected signed-out"),
            Step("key", key="Escape", slug="close-search"),
        ),
    ),

    # --- J-12 Reuse / improve a Procedure --------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: running workflows needs auth).
    JourneySpec(
        jid="J12", title="Reuse / improve a workflow", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-12",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; Workflows surface renders but needs auth",
        entries="primary: task-start gate; palette: 'workflow' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="running a workflow needs a live task; signed-out gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="workflow", slug="palette-workflow",
                 note="Workflows commands may be discoverable (run-4 surface "
                      "exists) — recorded"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-13 Collaborate on one task ------------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: collaboration needs a task).
    JourneySpec(
        jid="J13", title="Collaborate on one task", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-13",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; no Share/Invite surface signed-out",
        entries="primary: task-start gate; palette: 'share' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="Share/Invite is task-scoped; signed-out gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="share", slug="palette-share",
                 note="expected: no share/invite entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-14 Switch model without losing work ---------------------------
    # Expected: WORKS-WITH-DEFECTS (run-4: discovery verified via the promo
    # modal switching the composer model; model/effort/mode selectors
    # present; state preservation across a real task switch needs auth).
    # The scripted probe verifies picker opening + draft retention only —
    # shallower than the Lead's run-4 judgment; mismatch is recorded.
    JourneySpec(
        jid="J14", title="Switch model without losing work", kind="flow",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-14",
        expected_verdict="WORKS-WITH-DEFECTS",
        expected_note="run-4: selectors verified; state preservation auth-gated",
        entries="primary: composer model selector; palette: 'model' search",
        steps=(
            Step("click", x=UI["composer_input"][0], y=UI["composer_input"][1],
                 slug="focus-composer", entry=True),
            Step("type", text="J-14 model switch probe", slug="type-draft",
                 effect=True, region_change=R_COMPOSER,
                 note="draft present before the switch (continuity probe)"),
            Step("click", x=UI["composer_pill_model"][0], y=UI["composer_pill_model"][1],
                 slug="open-model-picker", entry=True, feedback=True,
                 region_change=R_MAIN,
                 note="composer model selector (run-3: model/effort/mode "
                      "selectors present in the composer)"),
            Step("key", key="Escape", slug="close-picker"),
            Step("shot", slug="draft-retained-check", region_stable=R_COMPOSER,
                 note="draft retained across picker open/close (local continuity)"),
        ),
        palette_steps=(
            Step("palette_probe", text="model", slug="palette-model",
                 note="search fallback for model switching"),
            Step("key", key="Return", slug="palette-model-enter", feedback=True,
                 region_change=R_MAIN,
                 note="palette model command opens the picker"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-15 Switch execution environment -------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: no environment exists to
    # switch signed-out; Settings surfaces for environments render).
    JourneySpec(
        jid="J15", title="Switch execution environment", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-15",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; Settings environment surfaces render",
        entries="primary: Settings surfaces; palette: 'environment' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("settings"), slug="open-settings",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="environment surfaces live in Settings (Plugins/MCP/Browser/"
                      "Computer use/Connections — run-4); signed-out state is the "
                      "honest boundary"),
            Step("click", x=UI["settings_category"][0], y=UI["settings_category"][1],
                 slug="settings-category", feedback=True, region_change=R_MAIN,
                 note="category row (best-estimate coordinate, probe-guarded — "
                      "tune UI map if it drifts)"),
            Step("click", x=UI["settings_back"][0], y=UI["settings_back"][1],
                 slug="back-to-app", feedback=True, region_change=R_MAIN),
        ),
        palette_steps=(
            Step("palette_probe", text="environment", slug="palette-environment",
                 note="expected: no environment-switch entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-16 Inspect resource conflicts ---------------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: conflicts need live resources).
    JourneySpec(
        jid="J16", title="Inspect resource conflicts", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-16",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: auth-gated; conflict views are task-scoped",
        entries="primary: task-start gate; palette: 'conflict' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("new_chat"), slug="click-new-chat",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="conflict views are task-scoped; signed-out gate"),
            Step("shot", slug="gate-evidence"),
            Step("key", key="Escape", slug="dismiss-gate"),
        ),
        palette_steps=(
            Step("palette_probe", text="conflict", slug="palette-conflict",
                 note="expected: no conflict entries signed-out"),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),

    # --- J-17 Review activity that needs the human -----------------------
    # Expected: MISSING (run-4: no Activity surface found in sidebar or
    # palette inventory signed-out — either chat/task-scoped or not yet
    # implemented; needs signed-in verification before final classification).
    # NOTE: absence of a specific row is not pixel-assertable without OCR —
    # the MISSING verdict rests on the calibrated palette differential
    # (no 'activity' entries) + sidebar inventory screenshots for the Lead.
    JourneySpec(
        jid="J17", title="Review activity that needs the human (Activity)",
        kind="discovery",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-17",
        expected_verdict="MISSING",
        expected_note="run-4: no Activity surface signed-out (sidebar + palette)",
        entries="primary: sidebar inventory; palette: 'activity' search + Ctrl+G",
        steps=(
            Step("shot", slug="sidebar-inventory", region_content=(R_SIDEBAR, 8.0),
                 note="primary path: a user looks at the persistent navigation — "
                      "Lead reviews for an Activity row (absence of a specific "
                      "row is not pixel-assertable without OCR)"),
        ),
        palette_steps=(
            Step("palette_probe", text="activity", slug="palette-activity",
                 entry=True,
                 note="search fallback: expected no palette entries (run-4 "
                      "palette inventory has no Activity entry)"),
            Step("key", key="Escape", slug="palette-escape"),
            Step("key", key=("ctrl", "g"), slug="ctrl-g-search", feedback=True,
                 region_change=R_MAIN,
                 note="chat search surface — Activity not present there either"),
            Step("key", key="Escape", slug="close-search"),
        ),
    ),

    # --- J-18 Discover automation opportunities --------------------------
    # Expected: UNAVAILABLE-BUT-HONEST (run-4: Workflows surface exists —
    # Teach a workflow / Published versions render, Durable instances shows
    # the raw L-008 error; using it needs auth; no silent auto-scheduling).
    JourneySpec(
        jid="J18", title="Discover automation opportunities", kind="gate",
        docs_ref="PRODUCT-UX-JOURNEYS §3 J-18",
        expected_verdict="UNAVAILABLE-BUT-HONEST",
        expected_note="run-4: Workflows surface renders, needs auth; L-008 noted",
        entries="primary: Workflows surface; palette: 'workflow' search",
        steps=(
            Step("click", x=UI["sidebar_x"], y=_sid("workflows"), slug="open-workflows",
                 entry=True, gate=True, feedback=True, region_change=R_MAIN,
                 note="Workflows view: Teach a workflow / Published versions "
                      "render; signed-out 'Durable instances' shows the raw "
                      "L-008 error — wording for Lead review"),
            Step("shot", slug="workflows-signedout",
                 note="evidence of the honest signed-out automation surface"),
        ),
        palette_steps=(
            Step("palette_probe", text="workflow", slug="palette-workflow",
                 note="workflow commands discoverable (run-4 surface exists)"),
            Step("key", key="Return", slug="palette-workflow-enter", feedback=True,
                 region_change=R_MAIN),
            Step("key", key="Escape", slug="palette-escape"),
        ),
    ),
)

SPEC_BY_ID = {s.jid: s for s in SPECS}


def load_specs() -> list:
    return list(SPECS)


def validate_all() -> list:
    errs = []
    ids = [s.jid for s in SPECS]
    if ids != [f"J{i:02d}" for i in range(1, 19)]:
        errs.append(f"spec ids must be J01..J18 in order, got {ids}")
    if len(set(ids)) != len(ids):
        errs.append("duplicate journey ids")
    for s in SPECS:
        errs.extend(validate_spec(s))
    return errs
