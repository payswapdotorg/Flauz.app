// WEB-001 — the web parity-lab journey driver (J-01..J-05, web variants).
//
// Runs the REAL web build (web/dist) in a REAL headless browser against
// a gateway and captures journey evidence under the parity-lab evidence
// schema: screenshots + action log + truthful-state assertions per
// journey, written to docs/research/evidence/web/.
//
// Two transports:
//   (default)  the mock gateway (lab/mock-gateway.mjs) — the worker
//              sandbox lacks the Rust toolchain, so the Node mock
//              stands in for the Rust gateway with the SAME protocol
//              contract; the captures evidence the REAL web build.
//   --gateway <cmd>  spawn the real Rust gateway (the Lead's gate
//              station): e.g.
//              node lab/journeys.mjs --gateway "target/debug/flauz-web-gateway --web-root dist"
//
// FV-002 (Wave 7) — the formal-pass manifest mode (ADDITIVE: without
// --manifest the driver's behavior is unchanged):
//   --manifest <path>  run the FV web scenes from the manifest
//              (web/lab/fv-manifest.json — the FV-CATALOG §3 rows:
//              journey id, gateway mode=real, assertions, ×3 runs, the
//              fv-gate evidence root). The manifest REQUIRES the real
//              gateway (mock captures are NOT formal evidence, the
//              Wave-7 addendum §1) and resolves its port from the
//              manifest's gateway block. FV-E00 (the gateway security
//              slice) runs as a manifest scene: /healthz, the
//              //etc/passwd traversal refusal, the relative-only SPA
//              fallback.
//   --scene <id>  filter to one manifest scene (e.g. fv-e01)
//   --runs <n>   override the per-scene run count (the ×3 convention)
//
// Usage: node lab/journeys.mjs [--gateway "<command>"] [--out <dir>] [--keep-state]
//       node lab/journeys.mjs --manifest web/lab/fv-manifest.json [--scene <id>] [--runs <n>]

import { spawn } from "node:child_process";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import process from "node:process";
import { chromium } from "@playwright/test";

const HERE = fileURLToPath(new URL(".", import.meta.url));
const WEB_ROOT = await import("node:path").then((path) => path.resolve(HERE, ".."));
const EVIDENCE_ROOT_DEFAULT = await import("node:path").then((path) =>
  path.resolve(WEB_ROOT, "../docs/research/evidence/web"),
);

const args = process.argv.slice(2);
function flagValue(name, fallback) {
  const index = args.indexOf(name);
  return index !== -1 && args[index + 1] !== undefined ? args[index + 1] : fallback;
}
// FV-002 (additive flags): the manifest mode. loadManifest() hard-fails
// with a named message when the manifest is missing or invalid; without
// --manifest these resolve exactly as before (the default behavior is
// unchanged).
const MANIFEST_PATH = flagValue("--manifest", "");
const SCENE_FILTER = flagValue("--scene", "");
const RUNS_OVERRIDE = flagValue("--runs", "");

function loadManifest(path) {
  const resolved = import("node:path").then((p) => p.resolve(WEB_ROOT, path));
  return resolved.then((absolute) =>
    import("node:fs/promises")
      .then((fs) => fs.readFile(absolute, "utf8"))
      .catch(() => {
        throw new Error(`the FV manifest is missing or unreadable: ${absolute} (author it per web/lab/fv-manifest.json — the FV-002 delivery)`);
      })
      .then((text) => {
        try {
          return JSON.parse(text);
        } catch (error) {
          throw new Error(`the FV manifest is not valid JSON: ${absolute} (${error.message})`);
        }
      }),
  );
}

const manifest = MANIFEST_PATH !== "" ? await loadManifest(MANIFEST_PATH) : null;

function resolveGatewayCommand() {
  const explicit = flagValue("--gateway", "");
  if (explicit !== "") {
    return explicit;
  }
  if (manifest !== null) {
    const gateway = manifest.gateway ?? {};
    if (gateway.mode !== "real") {
      throw new Error(`the FV manifest requires gateway mode=real (mock-gateway captures are NOT formal evidence — the Wave-7 addendum §1); got mode ${JSON.stringify(gateway.mode)}`);
    }
    if (typeof gateway.command !== "string" || gateway.command.trim() === "") {
      throw new Error("the FV manifest's gateway.command is empty — fix web/lab/fv-manifest.json (the real Rust gateway command, e.g. target/debug/flauz-web-gateway --web-root dist)");
    }
    return gateway.command;
  }
  return "";
}

function resolvePort() {
  const explicit = flagValue("--port", "");
  if (explicit !== "") {
    return Number(explicit);
  }
  if (manifest !== null && Number.isFinite(Number(manifest.gateway?.port))) {
    return Number(manifest.gateway.port);
  }
  return 8791;
}

function resolveOutRoot() {
  const explicit = flagValue("--out", "");
  if (explicit !== "") {
    return explicit;
  }
  if (manifest !== null && typeof manifest.evidence_root === "string" && manifest.evidence_root !== "") {
    return import("node:path").then((p) => p.resolve(WEB_ROOT, "..", manifest.evidence_root));
  }
  return EVIDENCE_ROOT_DEFAULT;
}

const GATEWAY_COMMAND = resolveGatewayCommand();
const OUT_ROOT = await resolveOutRoot();
const PORT = resolvePort();
const BASE_URL = `http://127.0.0.1:${PORT}`;

const stateDir = `${HERE}/.run`;
const stateFile = `${stateDir}/lab-state.json`;

// ---------------------------------------------------------------------------
// Gateway lifecycle.

let gateway = null;
let usingMock = false;

async function startGateway() {
  if (GATEWAY_COMMAND === "") {
    usingMock = true;
    gateway = spawn(process.execPath, ["lab/mock-gateway.mjs", "--port", String(PORT), "--root", "dist", "--state", stateFile], {
      cwd: WEB_ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    });
  } else {
    usingMock = false;
    // FV-002 harness fix (disclosed in the FV-002 completion report): the
    // real-gateway spawn path previously called rest.split(" ") on the
    // already-split token array (`rest` is an array — `[command, ...rest]`
    // of GATEWAY_COMMAND.split(" ")), which throws
    // "rest.split is not a function" on EVERY --gateway run, so the
    // real-transport flag (and the FV manifest lane) could never start.
    // The fix passes the token array itself — identical intent, harness
    // code only (no journey behavior change, no product source).
    const [command, ...rest] = GATEWAY_COMMAND.split(" ");
    gateway = spawn(command, rest.filter(Boolean), {
      cwd: WEB_ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    });
  }
  gateway.stdout.on("data", (chunk) => process.stdout.write(`[gateway] ${chunk}`));
  gateway.stderr.on("data", (chunk) => process.stderr.write(`[gateway:err] ${chunk}`));
  await waitForHealth(30);
}

async function waitForHealth(timeoutSeconds) {
  const deadline = Date.now() + timeoutSeconds * 1000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${BASE_URL}/healthz`);
      if (response.ok) {
        return;
      }
    } catch {
      // Not up yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error(`the gateway did not become healthy within ${timeoutSeconds}s`);
}

function killGateway() {
  if (gateway !== null) {
    gateway.kill("SIGKILL");
    gateway = null;
  }
}

// ---------------------------------------------------------------------------
// The evidence record writer (parity-lab evidence schema vocabulary).

const runId = `webrun_${Date.now().toString(36)}`;
const run = {
  v: 1,
  kind: "flauz.web-lab.run",
  run_id: runId,
  adapter: {
    provider_kind: "flauz-web-lab",
    family: "local",
    locality: "local",
    lab_surfaces: [
      "web.shell",
      "web.connection",
      "web.sessions",
      "web.composer",
      "web.approvals",
      "web.environments",
      "web.models",
      "web.skills",
      "web.collab",
      "web.artifacts",
      "web.a11y",
    ],
    transport: GATEWAY_COMMAND === "" ? "mock-gateway (Node; the worker sandbox lacks the Rust toolchain — the Lead reruns with the real gateway via --gateway)" : "flauz-web-gateway (real)",
    browser: "",
  },
  started_at: new Date().toISOString(),
  journeys: [],
};

async function capture(page, directory, name) {
  await mkdir(directory, { recursive: true });
  const file = `frames/${name}.png`;
  await page.screenshot({ path: `${directory}/${file}`, fullPage: false });
  return file;
}

function verdict(actual, expected) {
  return actual === expected ? "pass" : "fail";
}

async function recordJourney(page, definition) {
  const directory = `${OUT_ROOT}/${definition.id}`;
  await rm(directory, { recursive: true, force: true });
  await mkdir(`${directory}/frames`, { recursive: true });
  const record = {
    journey_id: definition.journeyId,
    journey_name: definition.name,
    product_journey: definition.productJourney,
    steps: [],
    outcome: "pass",
  };
  run.journeys.push(record);
  const step = (action) => {
    const entry = { step_id: `s${record.steps.length + 1}`, action, frames: [], assertions: [] };
    record.steps.push(entry);
    return entry;
  };
  const assert = (entry, id, stateKey, observation, expected) => {
    const result = verdict(observation, expected);
    if (result === "fail") {
      record.outcome = "fail";
    }
    entry.assertions.push({ id, kind: "state", state_key: stateKey, observation, expected, verdict: result });
  };
  await definition.drive({ page, record, step, assert, capture: (entry, name) => capture(page, directory, name) });
  record.finished_at = new Date().toISOString();
  await writeFile(`${directory}/run.json`, `${JSON.stringify(record, null, 2)}\n`, "utf8");
  return record;
}

// ---------------------------------------------------------------------------
// Shared browser helpers.

async function bannerState(page) {
  return page.getAttribute('[data-testid="connection-banner"]', "data-state");
}

async function bannerReason(page) {
  const reason = await page.textContent('[data-testid="connection-reason"]').catch(() => null);
  return reason === null ? "" : reason.trim();
}

async function waitForBanner(page, state, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if ((await bannerState(page)) === state) {
      return true;
    }
    await page.waitForTimeout(150);
  }
  return false;
}

// ---------------------------------------------------------------------------
// The journeys (J-01..J-05, web foundation variants).

const j01 = {
  id: "j-01-start-project",
  journeyId: "J-01",
  name: "Start any project (web foundation variant)",
  productJourney: "Workspace → New Project/Task → describe objective → start → live session with success state; palette fallback discoverable.",
  async drive({ page, record, step, assert, capture }) {
    // Primary visible entry + first-run empty state.
    let entry = step("open the served web app and sign in");
    await page.goto(`${BASE_URL}/`, { waitUntil: "domcontentloaded" });
    const signedOut = (await page.textContent("body")).includes("Sign in to Flauz");
    if (signedOut) {
      // The ChatGPT sign-in flow (the app-server auth API through the bridge).
      const popup = page.waitForEvent("popup", { timeout: 10000 });
      await page.click('[data-testid="signin-chatgpt"]');
      const authPage = await popup;
      await authPage.waitForLoadState("domcontentloaded");
      await authPage.close();
    }
    const reachedConnected = await waitForBanner(page, "connected");
    assert(entry, "a1", "connection.state", reachedConnected ? "connected" : (await bannerState(page)), "connected");
    entry.frames.push(await capture(entry, "01-signed-in"));

    // The primary "Start a new task" entry is visible.
    entry = step("see the workspace home with the primary Start-a-new-task entry");
    const emptyVisible = (await page.locator('[data-testid="workspace-empty"]').count()) > 0;
    assert(entry, "a2", "workspace.empty_state", emptyVisible ? "visible" : "absent", "visible");
    const primaryEntry = (await page.locator('[data-testid="start-new-task"]').count()) > 0;
    assert(entry, "a3", "workspace.primary_entry", primaryEntry ? "visible" : "absent", "visible");
    entry.frames.push(await capture(entry, "02-workspace-empty"));

    // Palette fallback discovery (Ctrl+K).
    entry = step("open the command palette with Ctrl+K and find the new-task command");
    await page.keyboard.press("Control+k");
    await page.fill('[data-testid="palette-input"]', "task");
    const paletteHasNewTask = (await page.locator(".flauz-palette-item").count()) > 0;
    assert(entry, "a4", "palette.fallback", paletteHasNewTask ? "reachable" : "unreachable", "reachable");
    entry.frames.push(await capture(entry, "03-palette"));
    await page.keyboard.press("Escape");

    // Describe the objective and start.
    entry = step("describe the objective in plain language and start the task");
    await page.click('[data-testid="start-new-task"]');
    await page.fill("#newtask-objective", "Plan the community garden layout for next spring");
    await page.click('[data-testid="newtask-start"]');
    await page.waitForFunction(
      () => (document.querySelector('[data-testid="session-connection"]')?.textContent ?? "").includes("Connected"),
      null,
      { timeout: 10000 },
    );
    const live = await page
      .locator('[data-testid="session-connection"]')
      .textContent()
      .catch(() => "");
    assert(entry, "a5", "session.connected_state", live.includes("Connected") ? "connected" : live, "connected");
    entry.frames.push(await capture(entry, "04-session-live"));

    // Success/next-step state: the timeline completed entry + next-step hint.
    entry = step("see the success state with the next-step affordance");
    await page.waitForTimeout(600);
    const completed = (await page.locator(".flauz-timeline-entry").count()) > 0;
    assert(entry, "a6", "session.success_state", completed ? "visible" : "absent", "visible");
    const nextStep = (await page.textContent("body")).includes("next step");
    assert(entry, "a7", "session.next_step_affordance", nextStep ? "present" : "absent", "present");
    entry.frames.push(await capture(entry, "05-session-success"));
  },
};

const j02 = {
  id: "j-02-understand-context",
  journeyId: "J-02",
  name: "Understand what the agent knows (web foundation variant)",
  productJourney: "Task → Context indicator → Context Inspector → current objective and activity, honestly scoped.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the session's Context inspector from the task surface");
    await page.click('[data-testid="context-button"]');
    const drawer = (await page.locator('[data-testid="context-drawer"]').count()) > 0;
    assert(entry, "a1", "context.inspector", drawer ? "visible" : "absent", "visible");
    const objective = await page.textContent('[data-testid="context-drawer"]').catch(() => "");
    assert(
      entry,
      "a2",
      "context.objective_present",
      objective.includes("garden") ? "present" : "absent",
      "present",
    );
    entry.frames.push(await capture(entry, "01-context-inspector"));
    entry = step("the context surface names its honest foundation scope");
    const honest = (await page.textContent('[data-testid="context-drawer"]')).includes("capability surfaces");
    assert(entry, "a3", "context.honest_scope", honest ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-context-honest-scope"));
    await page.click('[data-testid="context-button"]');
  },
};

const j03 = {
  id: "j-03-recover-reconnect",
  journeyId: "J-03",
  name: "Recover/continue after disconnection (kill → named state → restart → recover)",
  productJourney: "Connection drops → named disconnection state → gateway restarts → reconnect → state recovers.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("return to the workspace with a live connection");
    await page.click("text=Back to workspace");
    const connectedFirst = await waitForBanner(page, "connected");
    assert(entry, "a1", "connection.state", connectedFirst ? "connected" : (await bannerState(page)), "connected");
    entry.frames.push(await capture(entry, "01-connected"));

    // Kill the gateway.
    entry = step("kill the gateway process");
    killGateway();
    const sawReconnecting = await waitForBanner(page, "reconnecting", 15000);
    const reason = await bannerReason(page);
    assert(entry, "a2", "connection.state", sawReconnecting ? "reconnecting" : (await bannerState(page)), "reconnecting");
    assert(entry, "a3", "connection.reason_named", reason === "" ? "unnamed" : "named", "named");
    entry.frames.push(await capture(entry, "02-named-disconnection"));

    // Restart the gateway; the browser reconnects and the state recovers.
    entry = step("restart the gateway and watch the browser reconnect");
    await startGateway();
    const recovered = await waitForBanner(page, "connected", 30000);
    assert(entry, "a4", "connection.state", recovered ? "connected" : (await bannerState(page)), "connected");
    entry.frames.push(await capture(entry, "03-reconnected"));

    entry = step("verify the workspace state recovered (sessions list restored)");
    await page.waitForTimeout(800);
    const sessionRow = (await page.locator('[data-testid="session-row"]').count()) > 0;
    assert(entry, "a5", "workspace.sessions_recovered", sessionRow ? "restored" : "absent", "restored");
    entry.frames.push(await capture(entry, "04-recovered-workspace"));
  },
};

const j04 = {
  id: "j-04-capability-gap",
  journeyId: "J-04",
  name: "Discover a capability gap (named error + unlock path, web capability variant)",
  productJourney: "Agent needs a capability → task-visible named explanation → recovery action; never a silent no-op.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("start a task that needs a missing capability (gpu)");
    await page.click('[data-testid="start-new-task"]');
    await page.fill("#newtask-objective", "Fine-tune the report model on a gpu");
    await page.click('[data-testid="newtask-start"]');
    await page.waitForTimeout(600);
    const errorText = await page.textContent('[role="alert"]').catch(() => "");
    const namedCause = errorText.includes("capability gap") && errorText.includes("gpu");
    assert(entry, "a1", "capability.error_names_cause", namedCause ? "named" : "unnamed", "named");
    const namesRecovery = errorText.includes("Recovery") || errorText.includes("restate");
    assert(entry, "a2", "capability.error_names_recovery", namesRecovery ? "named" : "unnamed", "named");
    assert(entry, "a3", "capability.silent_noop", errorText === "" ? "silent" : "visible", "visible");
    entry.frames.push(await capture(entry, "01-named-capability-gap"));
    await page.click("text=Cancel");

    // The environments surface names the unlock path (J-04 law).
    entry = step("open the session's Environments surface for the unlock path");
    await page.click('[data-testid="session-row"]');
    await page.click('[data-testid="environments-button"]');
    const drawerText = await page.textContent('[data-testid="environments-drawer"]').catch(() => "");
    const unlockNamed = drawerText.includes("How it unlocks");
    assert(entry, "a4", "capability.unlock_path_named", unlockNamed ? "named" : "unnamed", "named");
    const gapIsProtocolDerived = drawerText.includes("environment control methods");
    assert(entry, "a5", "capability.gap_protocol_derived", gapIsProtocolDerived ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-environments-unlock-path"));

    // The skills surface names the locked-skill unlock path (J-04 law).
    entry = step("open the Skills surface: the locked-skill unlock path");
    await page.click('[data-testid="skills-button"]');
    const skillsText = await page.textContent('[data-testid="skills-panel"]').catch(() => "");
    const skillUnlock = skillsText.includes("Reference an installed skill by name");
    assert(entry, "a6", "capability.skill_unlock_path_named", skillUnlock ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "03-skills-unlock-path"));
    await page.click('[data-testid="skills-button"]');
  },
};

const j05 = {
  id: "j-05-add-environment",
  journeyId: "J-05",
  name: "Add another environment/site (named gap + the protocol's own control)",
  productJourney: "Task → Environments → the surface is discoverable, the named gap is honest, and the per-turn working-directory control rides the protocol.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Environments drawer from the session surface");
    await page.click('[data-testid="environments-button"]');
    const drawer = (await page.locator('[data-testid="environments-drawer"]').count()) > 0;
    assert(entry, "a1", "environments.surface", drawer ? "visible" : "absent", "visible");
    const text = await page.textContent('[data-testid="environments-drawer"]');
    const reportedCwd = (await page.textContent('[data-testid="environments-reported-cwd"]').catch(() => "")) ?? "";
    assert(entry, "a2", "environments.reported_cwd", reportedCwd.includes("/tmp") ? "reported" : "missing", "reported");
    const gapHonest = text.includes("carries no environment control methods");
    assert(entry, "a3", "environments.named_gap", gapHonest ? "honest" : "missing", "honest");
    const hasControl = (await page.locator('[data-testid="environments-cwd-input"]').count()) > 0;
    assert(entry, "a4", "environments.cwd_control", hasControl ? "present" : "absent", "present");
    entry.frames.push(await capture(entry, "01-environments-surface"));

    // Keyboard path: Ctrl+/ opens the shortcut sheet; Escape closes.
    entry = step("verify the keyboard path (Ctrl+/ shortcuts sheet, Escape restore)");
    await page.keyboard.press("Control+/");
    const sheetVisible = (await page.locator(".flauz-overlay").count()) > 0;
    assert(entry, "a5", "keyboard.shortcuts_sheet", sheetVisible ? "visible" : "absent", "visible");
    entry.frames.push(await capture(entry, "02-keyboard-sheet"));
    await page.keyboard.press("Escape");
    const sheetClosed = (await page.locator(".flauz-overlay").count()) === 0;
    assert(entry, "a6", "keyboard.escape_closes", sheetClosed ? "closed" : "open", "closed");
  },
};

// ---------------------------------------------------------------------------
// The WEB-002 capability journeys (J-06..J-18 web variants + the
// domain-neutral scenario + the a11y/responsive pass).

const j06 = {
  id: "j-06-cross-environment",
  journeyId: "J-06",
  name: "Coordinate browser + terminal + sandbox (web variant: the honest topology)",
  productJourney: "Task → Environments → the runtime's execution context is visible, the produced artifacts are linked to the task, and the multi-environment gap is named.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("start a task that produces a reported artifact");
    await page.click("text=Back to workspace");
    await page.click('[data-testid="start-new-task"]');
    await page.fill("#newtask-objective", "Check the planting calendar report for the south beds");
    await page.click('[data-testid="newtask-start"]');
    await page.waitForTimeout(700);
    const produced = (await page.locator(".flauz-timeline-entry").count()) > 0;
    assert(entry, "a1", "task.turn_ran", produced ? "ran" : "absent", "ran");
    entry.frames.push(await capture(entry, "01-task-started"));

    entry = step("open the Environments surface: the execution context and the honest topology");
    await page.click('[data-testid="environments-button"]');
    const envText = await page.textContent('[data-testid="environments-drawer"]');
    const contextVisible = envText.includes("/tmp/flauz-lab");
    assert(entry, "a2", "environments.execution_context", contextVisible ? "visible" : "absent", "visible");
    const topologyGapNamed = envText.includes("remote environments");
    assert(entry, "a3", "environments.topology_gap_named", topologyGapNamed ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-environments-topology"));

    entry = step("open the Artifacts surface: the produced artifact is linked to the task");
    await page.click('[data-testid="artifacts-button"]');
    await page.waitForTimeout(500);
    const fileChange = await page.locator('[data-testid="artifact-item"][data-item-type="fileChange"]').count();
    assert(entry, "a4", "artifacts.file_change_reported", fileChange > 0 ? "reported" : "absent", "reported");
    const agentMessage = await page.locator('[data-testid="artifact-item"][data-item-type="agentMessage"]').count();
    assert(entry, "a5", "artifacts.agent_message_reported", agentMessage > 0 ? "reported" : "absent", "reported");
    entry.frames.push(await capture(entry, "03-artifacts-linked"));
    await page.click('[data-testid="artifacts-button"]');
  },
};

const j08 = {
  id: "j-08-takeover-approval-cancellation",
  journeyId: "J-08",
  name: "Human takeover / handoff + cancellation (named records, never auto-resolved)",
  productJourney: "Task → the agent requests a decision → a NAMED approval record arrives (aria-live) → the user decides → the decision stays a named record; a watched turn cancelled by the user says so.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("start a task that needs a decision mid-turn (install)");
    await page.click("text=Back to workspace");
    await page.click('[data-testid="start-new-task"]');
    await page.fill("#newtask-objective", "Install the dependencies for the garden planner");
    await page.click('[data-testid="newtask-start"]');
    await page.waitForTimeout(700);
    const card = (await page.locator('[data-testid="approval-card"]').count()) > 0;
    assert(entry, "a1", "approval.record_arrives", card ? "arrived" : "absent", "arrived");
    entry.frames.push(await capture(entry, "01-approval-arrives"));

    entry = step("the record never auto-resolves (§6 conflict-honesty)");
    await page.waitForTimeout(900);
    const stillOpen = (await page.locator('[data-testid="approval-card"]').count()) > 0;
    assert(entry, "a2", "approval.never_auto_resolved", stillOpen ? "open" : "vanished", "open");
    const liveRegion = await page.getAttribute('[data-testid="approval-queue"]', "aria-live");
    assert(entry, "a3", "approval.announced", liveRegion === "polite" ? "polite" : String(liveRegion), "polite");

    entry = step("the user decides; the decision stays a named record");
    await page.click('[data-testid="approval-approve"]');
    await page.waitForTimeout(300);
    const decided = (await page.locator('[data-testid="approval-record"]').count()) > 0;
    assert(entry, "a4", "approval.decision_recorded", decided ? "named_record" : "vanished", "named_record");
    const outcome = await page.getAttribute('[data-testid="approval-record"]', "data-outcome");
    assert(entry, "a5", "approval.outcome_attributed", outcome ?? "none", "approved");
    entry.frames.push(await capture(entry, "02-decision-record"));

    entry = step("cancel a watched turn: the cancelled state is named (propagation honesty)");
    await page.fill("#session-composer", "Run the long survey of the south beds");
    await page.keyboard.press("Control+Enter");
    await page.waitForTimeout(600);
    const interruptVisible = (await page.locator('[data-testid="interrupt-button"]').count()) > 0;
    assert(entry, "a6", "turn.stop_control", interruptVisible ? "visible" : "absent", "visible");
    await page.click('[data-testid="interrupt-button"]');
    await page.waitForTimeout(500);
    const cancelled = await page.locator('.flauz-timeline-entry[data-kind="turn_cancelled"]').count();
    assert(entry, "a7", "turn.cancelled_named", cancelled > 0 ? "named" : "missing", "named");
    const cancelledText = await page.textContent('.flauz-timeline-entry[data-kind="turn_cancelled"]').catch(() => "");
    assert(entry, "a8", "turn.cancelled_names_reason", cancelledText.includes("Cancelled") ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "03-cancelled-named"));
  },
};

const j13 = {
  id: "j-13-collaborate",
  journeyId: "J-13",
  name: "Collaborate on one task (web variant: the named gap + shared decisions)",
  productJourney: "Task → Collaborators → the membership/presence gap is named with the collaboration contract, shared decisions are counted, the privacy law is stated.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Collaborators surface from the task rail");
    await page.click('[data-testid="collaborators-button"]');
    const panel = (await page.locator('[data-testid="collaborators-panel"]').count()) > 0;
    assert(entry, "a1", "collab.surface", panel ? "visible" : "absent", "visible");
    entry.frames.push(await capture(entry, "01-collaborators"));

    entry = step("the membership/presence gap is named with the recovery path");
    const text = await page.textContent('[data-testid="collaborators-panel"]');
    const gapNamed = text.includes("no membership or presence methods");
    assert(entry, "a2", "collab.gap_named", gapNamed ? "named" : "unnamed", "named");
    const rolesNamed = text.includes("owner, admin, contributor and viewer");
    assert(entry, "a3", "collab.contract_shapes_named", rolesNamed ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-collab-gap"));

    entry = step("shared decisions and the privacy law are stated");
    const decisions = await page.textContent('[data-testid="collab-approval-count"]');
    const decisionsHonest = decisions.includes("decisions") || decisions.includes("No decisions");
    assert(entry, "a4", "collab.shared_decisions_truthful", decisionsHonest ? "truthful" : "missing", "truthful");
    const privacy = (await page.textContent('[data-testid="collab-privacy-note"]')).includes("private context");
    assert(entry, "a5", "collab.privacy_law_stated", privacy ? "stated" : "missing", "stated");
    entry.frames.push(await capture(entry, "03-shared-decisions"));

    // The palette fallback discovery path.
    entry = step("the palette locates the surface (search fallback)");
    await page.keyboard.press("Control+k");
    await page.fill('[data-testid="palette-input"]', "collaborators");
    const paletteHas = (await page.locator(".flauz-palette-item").count()) > 0;
    assert(entry, "a6", "collab.palette_fallback", paletteHas ? "reachable" : "unreachable", "reachable");
    entry.frames.push(await capture(entry, "04-palette-fallback"));
    await page.keyboard.press("Escape");
    await page.click('[data-testid="collaborators-button"]');
  },
};

const j14 = {
  id: "j-14-switch-model",
  journeyId: "J-14",
  name: "Switch model without losing work (the choice rides the turn's own fields)",
  productJourney: "Task → Model → choose another model/effort → the next turn rides it → the same task continues with the reported model updated.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Model surface: the current model is reported");
    await page.click('[data-testid="model-button"]');
    const currentModel = await page.textContent('[data-testid="model-current-model"]').catch(() => "");
    assert(entry, "a1", "model.current_reported", currentModel.includes("gpt-5.6-sol") ? "reported" : currentModel, "reported");
    entry.frames.push(await capture(entry, "01-model-current"));

    entry = step("choose the model and effort for the next turn");
    await page.fill('[data-testid="model-input"]', "gpt-5.6-mini");
    await page.fill('[data-testid="model-effort-input"]', "medium");
    await page.click('[data-testid="model-apply"]');
    const applied = await page.textContent('[data-testid="model-applied"]');
    assert(entry, "a2", "model.choice_applied", applied.includes("gpt-5.6-mini") ? "applied" : "missing", "applied");
    assert(entry, "a3", "model.effort_applied", applied.includes("medium") ? "applied" : "missing", "applied");
    entry.frames.push(await capture(entry, "02-model-chosen"));

    entry = step("send the next turn — the same task continues");
    const sessionUrl = page.url();
    await page.fill("#session-composer", "Continue with the medium-effort planting plan");
    await page.keyboard.press("Control+Enter");
    await page.waitForTimeout(900);
    assert(entry, "a4", "model.same_task_continues", page.url() === sessionUrl ? "same_task" : "navigated", "same_task");
    entry.frames.push(await capture(entry, "03-turn-rode-choice"));

    entry = step("the runtime now reports the chosen model (recovery/reload path)");
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1200);
    await page.click('[data-testid="model-button"]');
    const newModel = await page.textContent('[data-testid="model-current-model"]').catch(() => "");
    assert(entry, "a5", "model.switch_reflected", newModel.includes("gpt-5.6-mini") ? "reflected" : newModel, "reflected");
    const sameAfterReload = page.url() === sessionUrl;
    assert(entry, "a6", "model.task_continuity", sameAfterReload ? "same_task" : "lost", "same_task");
    entry.frames.push(await capture(entry, "04-model-switched-same-task"));
    await page.click('[data-testid="model-button"]');
  },
};

const j15 = {
  id: "j-15-switch-environment",
  journeyId: "J-15",
  name: "Switch execution environment without losing work (web variant: cwd + the named gap)",
  productJourney: "Task → Environment → the compatible-environment switch is a named gap with its recovery path; the working-directory control rides the protocol and is reflected.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Environments surface: the remote-switch gap is named");
    await page.click('[data-testid="environments-button"]');
    const gapText = await page.textContent('[data-testid="environments-drawer"]');
    const switchGap = gapText.includes("Listing, selecting and connecting remote environments");
    assert(entry, "a1", "environment.switch_gap_named", switchGap ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "01-environment-gap"));

    entry = step("set the working directory for the next turn (the protocol's own control)");
    await page.fill('[data-testid="environments-cwd-input"]', "/tmp/garden-south");
    await page.click('[data-testid="environments-cwd-apply"]');
    const applied = await page.textContent('[data-testid="environments-cwd-applied"]');
    assert(entry, "a2", "environment.cwd_applied", applied.includes("/tmp/garden-south") ? "applied" : "missing", "applied");
    entry.frames.push(await capture(entry, "02-cwd-applied"));

    entry = step("send the next turn — the same task continues");
    const sessionUrl = page.url();
    await page.fill("#session-composer", "Continue planning the south beds");
    await page.keyboard.press("Control+Enter");
    await page.waitForTimeout(900);
    assert(entry, "a3", "environment.same_task_continues", page.url() === sessionUrl ? "same_task" : "navigated", "same_task");

    entry = step("the runtime reports the new working directory (recovery/reload path)");
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1200);
    await page.click('[data-testid="environments-button"]');
    const reported = await page.textContent('[data-testid="environments-reported-cwd"]').catch(() => "");
    assert(entry, "a4", "environment.cwd_reflected", reported.includes("/tmp/garden-south") ? "reflected" : reported, "reflected");
    entry.frames.push(await capture(entry, "03-cwd-reflected"));
    await page.click('[data-testid="environments-button"]');
  },
};

const j09art = {
  id: "j-09-artifacts-items",
  journeyId: "J-09",
  name: "Understand what happened (web variant: the artifacts/items record)",
  productJourney: "Task → Artifacts → the items the runtime reports, labeled by their own type — agent messages, commands, file changes — with the documents/sheets gap named.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Artifacts surface from the task rail");
    await page.click('[data-testid="artifacts-button"]');
    await page.waitForTimeout(600);
    const panel = (await page.locator('[data-testid="artifacts-panel"]').count()) > 0;
    assert(entry, "a1", "artifacts.surface", panel ? "visible" : "absent", "visible");
    const items = await page.locator('[data-testid="artifact-item"]').count();
    assert(entry, "a2", "artifacts.items_listed", items > 0 ? "listed" : "empty", "listed");
    entry.frames.push(await capture(entry, "01-artifacts-items"));

    entry = step("the items are labeled by their own protocol type");
    const kinds = await page.locator(".flauz-item-kind").allTextContents();
    const labeled = kinds.some((kind) => kind.length > 0);
    assert(entry, "a3", "artifacts.kind_labels", labeled ? "labeled" : "raw", "labeled");
    const types = new Set(await page.locator('[data-testid="artifact-item"]').evaluateAll((nodes) => nodes.map((node) => node.getAttribute("data-item-type"))));
    assert(entry, "a4", "artifacts.types_carried", types.has("agentMessage") ? "carried" : "missing", "carried");

    entry = step("refreshing re-reads the honest record");
    await page.click('[data-testid="artifacts-refresh"]');
    await page.waitForTimeout(500);
    const stillListed = (await page.locator('[data-testid="artifact-item"]').count()) > 0;
    assert(entry, "a5", "artifacts.refresh", stillListed ? "works" : "broken", "works");

    entry = step("the documents/sheets gap is named with its recovery path");
    const gapText = await page.textContent('[data-testid="artifacts-panel"]');
    const gapNamed = gapText.includes("no artifact, document or sheet methods");
    assert(entry, "a6", "artifacts.gap_named", gapNamed ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-artifacts-gap"));
    await page.click('[data-testid="artifacts-button"]');
  },
};

const jNeutral = {
  id: "j-domain-neutral-research",
  journeyId: "DOMAIN-NEUTRAL",
  name: "The domain-neutral research scenario (a non-code workflow across the capability surfaces)",
  productJourney: "A garden-planning research task (no code): pick the model at compose time, reference a skill, read the produced items, see the collaboration gap named — the shared acceptance law.",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("compose a non-code research objective and pick the model at compose time");
    await page.click("text=Back to workspace");
    await page.click('[data-testid="start-new-task"]');
    await page.fill("#newtask-objective", "Research seasonal planting calendars for the community garden");
    await page.click('[data-testid="newtask-model-change"]');
    await page.fill('[data-testid="newtask-model-input"]', "gpt-5.6-mini");
    await page.click('[data-testid="newtask-model-apply"]');
    const summary = await page.textContent('[data-testid="newtask-model-summary"]');
    assert(entry, "a1", "neutral.compose_model_choice", summary.includes("gpt-5.6-mini") ? "chosen" : "missing", "chosen");
    entry.frames.push(await capture(entry, "01-compose-with-model"));
    await page.click('[data-testid="newtask-start"]');
    await page.waitForTimeout(900);
    entry.frames.push(await capture(entry, "02-research-task"));

    entry = step("reference a skill for the research (the protocol's own input variant)");
    await page.click('[data-testid="skills-button"]');
    await page.fill('[data-testid="skill-name-input"]', "seasonal-planting");
    await page.fill('[data-testid="skill-path-input"]', "/skills/seasonal-planting.md");
    await page.fill('[data-testid="skill-note-input"]', "use it for the spring beds");
    await page.click('[data-testid="skill-use-button"]');
    await page.waitForTimeout(900);
    const timeline = await page.textContent(".flauz-timeline");
    assert(entry, "a2", "neutral.skill_referenced", timeline.includes("seasonal-planting") ? "referenced" : "missing", "referenced");
    entry.frames.push(await capture(entry, "03-skill-referenced"));

    entry = step("the runtime reports the research items (non-code artifacts)");
    await page.click('[data-testid="artifacts-button"]');
    await page.waitForTimeout(600);
    const items = await page.locator('[data-testid="artifact-item"]').count();
    assert(entry, "a3", "neutral.items_reported", items > 0 ? "reported" : "empty", "reported");
    entry.frames.push(await capture(entry, "04-items"));

    entry = step("the model and collaboration surfaces stay honest on the research task");
    await page.click('[data-testid="model-button"]');
    const modelText = await page.textContent('[data-testid="model-panel"]');
    assert(entry, "a4", "neutral.model_reported", modelText.includes("gpt-5.6-mini") ? "reported" : "missing", "reported");
    await page.click('[data-testid="collaborators-button"]');
    const collabText = await page.textContent('[data-testid="collaborators-panel"]');
    assert(entry, "a5", "neutral.collab_gap_named", collabText.includes("no membership or presence methods") ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "05-honest-surfaces"));
    await page.click('[data-testid="collaborators-button"]');
  },
};

const jA11y = {
  id: "j-a11y-responsive",
  journeyId: "A11Y",
  name: "The accessibility/responsive pass (keyboard-only drives, aria assertions, mobile + desktop captures)",
  productJourney: "Every capability surface is keyboard-complete (tab/enter/escape with focus restoration), aria-truthful, and responsive from mobile to desktop width.",
  async drive({ page, record, step, assert, capture }) {
    // The no-session palette path names where to go (no silent no-op).
    let entry = step("the palette names the task-scoped surfaces when no task is open");
    await page.click("text=Back to workspace");
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1200);
    await page.keyboard.press("Control+k");
    await page.fill('[data-testid="palette-input"]', "model");
    await page.keyboard.press("Enter");
    await page.waitForTimeout(400);
    const notice = await page.textContent('[data-testid="notice-strip"]').catch(() => "");
    assert(entry, "a1", "a11y.palette_no_session_named", notice.includes("Open a task first") ? "named" : "silent", "named");
    entry.frames.push(await capture(entry, "01-palette-no-session"));
    if (notice !== "") {
      await page.click('[data-testid="notice-strip"] button');
    }

    entry = step("keyboard-only: tab to the task rail and open every capability surface with Enter");
    await page.click('[data-testid="session-row"]');
    await page.waitForTimeout(800);
    const rail = page.locator('[data-testid="task-rail"]');
    assert(entry, "a2", "a11y.rail_group_label", (await rail.getAttribute("role")) === "group" ? "labeled" : "unlabeled", "labeled");
    const railButtons = [
      "context-button",
      "environments-button",
      "model-button",
      "skills-button",
      "collaborators-button",
      "artifacts-button",
    ];
    for (const testId of railButtons) {
      const button = page.locator(`[data-testid="${testId}"]`);
      await button.focus();
      const focusedBefore = await page.evaluate(() => document.activeElement?.getAttribute("data-testid"));
      await page.keyboard.press("Enter");
      await page.waitForTimeout(250);
      const pressed = await button.getAttribute("aria-pressed");
      assert(entry, `a3-${testId}`, `a11y.${testId}.opens_by_keyboard`, pressed === "true" ? "opens" : "closed", "opens");
      await page.keyboard.press("Escape");
      await page.waitForTimeout(250);
      const closed = (await button.getAttribute("aria-pressed")) === "false";
      const focusRestored = await page.evaluate(() => document.activeElement?.getAttribute("data-testid"));
      assert(entry, `a4-${testId}`, `a11y.${testId}.escape_closes`, closed ? "closed" : "open", "closed");
      assert(
        entry,
        `a5-${testId}`,
        `a11y.${testId}.focus_restored`,
        focusRestored === focusedBefore ? "restored" : "lost",
        "restored",
      );
    }
    entry.frames.push(await capture(entry, "02-keyboard-rail"));

    entry = step("aria truth: gap cards are notes, rail buttons carry pressed state");
    const gapRole = await page
      .locator('[data-testid="environments-button"]')
      .click()
      .then(() => page.getAttribute('[data-testid="gap-environments"]', "role"));
    assert(entry, "a6", "a11y.gap_role_note", gapRole === "note" ? "note" : String(gapRole), "note");
    await page.keyboard.press("Escape");

    entry = step("responsive: the surfaces render at mobile width (375px)");
    await page.setViewportSize({ width: 375, height: 812 });
    await page.waitForTimeout(300);
    for (const [testId, panelId] of [
      ["environments-button", "environments-drawer"],
      ["model-button", "model-panel"],
      ["skills-button", "skills-panel"],
      ["collaborators-button", "collaborators-panel"],
      ["artifacts-button", "artifacts-panel"],
    ]) {
      await page.click(`[data-testid="${testId}"]`);
      await page.waitForTimeout(250);
      const box = await page.locator(`[data-testid="${panelId}"]`).boundingBox();
      assert(
        entry,
        `a7-${panelId}`,
        `a11y.${panelId}.mobile_width`,
        box !== null && box.width > 300 ? "full_width" : "narrow",
        "full_width",
      );
      entry.frames.push(await capture(entry, `03-mobile-${panelId}`));
      await page.click(`[data-testid="${testId}"]`);
    }
    const navBox = await page.locator(".flauz-nav").boundingBox();
    assert(entry, "a8", "a11y.mobile_nav_visible", navBox !== null && navBox.height > 0 ? "visible" : "hidden", "visible");
    entry.frames.push(await capture(entry, "04-mobile-shell"));

    entry = step("responsive: back at desktop width (1280px)");
    await page.setViewportSize({ width: 1280, height: 860 });
    await page.waitForTimeout(300);
    await page.click('[data-testid="model-button"]');
    await page.waitForTimeout(250);
    const desktopBox = await page.locator('[data-testid="model-panel"]').boundingBox();
    assert(
      entry,
      "a9",
      "a11y.desktop_side_panel",
      desktopBox !== null && (desktopBox?.width ?? 0) < 400 ? "side_panel" : "unexpected",
      "side_panel",
    );
    entry.frames.push(await capture(entry, "05-desktop-model-panel"));
    await page.click('[data-testid="model-button"]');
  },
};

// ---------------------------------------------------------------------------
// FV-002 (Wave 7) — the FV-E00 gateway security scene + the manifest
// run mode (ADDITIVE: reachable only through --manifest; the default
// journey list above is unchanged).
//
// CALIBRATION (2026-09-25, verified against the pinned base
// 7f660c00407a5741eee975b2274570a200ff576b):
//   - the gateway's default bind is 127.0.0.1:8610 (flauz-web-gateway
//     main.rs:18 "Bind address (default 127.0.0.1:8610, localhost-only)";
//     a non-loopback bind requires --session-token-file, main.rs:19)
//   - /healthz answers 200 with the health payload { v, kind, status:
//     "ok", bind, uptimeMs, sessions } (server.rs:137 + static_files.rs
//     health_payload :182-191 — liveness and session count only, no
//     credentials)
//   - "//etc/passwd" is a REFUSAL (404, never file content): the
//     protocol-relative form must never be treated as an in-app route
//     (static_files.rs safe_relative_path :102-131 — the 2026-09-25
//     Lead gate fix: it used to SPA-serve the shell with 200)
//   - the SPA fallback serves ONLY relative in-app routes (the
//     extension-less unknown path serves index.html; missing assets
//     404; traversal/absolute/backslash/NUL forms all refuse)

const je00 = {
  id: "fv-e00-gateway-security",
  journeyId: "GATEWAY-SECURITY",
  name: "The gateway security slice (W6 I-11/I-12: loopback bind, healthz, traversal refusal, relative-only SPA fallback)",
  productJourney: "The gateway boots with the default localhost bind; /healthz is ok; //etc/passwd is refused with 404 (never file content); the SPA fallback serves only relative in-app routes.",
  async drive({ record, step, assert }) {
    let entry = step("the gateway answers /healthz (ok, with its bind reported)");
    const health = await fetch(`${BASE_URL}/healthz`)
      .then((response) => response.json().catch(() => null))
      .catch(() => null);
    assert(entry, "a1", "gateway.healthz_ok", health !== null && health.status === "ok" ? "ok" : "unreachable", "ok");

    entry = step("the bind is the documented localhost default (or the run's documented override)");
    const bind = typeof health?.bind === "string" ? health.bind : "";
    const loopback = bind.startsWith("127.0.0.1:") || bind.startsWith("[::1]:") || bind.startsWith("localhost:");
    assert(entry, "a2", "gateway.bind_loopback", bind === "" ? "unreported" : (loopback ? "loopback" : bind), "loopback");

    entry = step("the //etc/passwd traversal probe is refused (404, never file content)");
    const traversal = await fetch(`${BASE_URL}//etc/passwd`).catch(() => null);
    const traversalStatus = traversal === null ? "unreachable" : String(traversal.status);
    let traversalBody = "";
    if (traversal !== null) {
      traversalBody = await traversal.text().catch(() => "");
    }
    assert(entry, "a3", "gateway.traversal_refused", traversalStatus === "404" ? "refused_404" : traversalStatus, "refused_404");
    const leakedRoot = traversalBody.includes("root:") ? "file_content" : "no_file_content";
    assert(entry, "a3b", "gateway.traversal_no_file_content", leakedRoot, "no_file_content");

    entry = step("a relative in-app route serves the SPA shell (the fallback is relative-only)");
    const shell = await fetch(`${BASE_URL}/tasks`).catch(() => null);
    const shellStatus = shell === null ? "unreachable" : String(shell.status);
    let shellIsApp = false;
    if (shell !== null && shell.ok) {
      const body = await shell.text().catch(() => "");
      shellIsApp = body.includes("<!DOCTYPE html") || body.includes("<html");
    }
    assert(entry, "a4", "gateway.spa_fallback_relative_only", shellStatus === "200" && shellIsApp ? "shell_served" : shellStatus, "shell_served");
  },
};

// The FV manifest scene runner: every requested scene runs `runs` times
// (the w6 ×3 convention — the scene passes only when every run passes);
// each run lands under <evidence_root>/<scene-id>/run-<n>/ with the
// parity-lab record schema.

// Manifest-path only: after a restart kill, wait for the gateway port to
// actually free (a SIGKILL'd listener can leave the bind briefly held;
// the tight per-run restart would otherwise race into EADDRINUSE).
async function waitForPortFree(timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      await fetch(`${BASE_URL}/healthz`, { signal: AbortSignal.timeout(250) });
    } catch {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error(`the gateway port did not free within ${timeoutMs}ms after the restart kill — a stale gateway is still answering ${BASE_URL}/healthz`);
}

async function runManifestMode() {
  if (GATEWAY_COMMAND === "") {
    throw new Error("the FV manifest mode requires the real gateway (mode=real) — no --gateway command resolved (see web/lab/fv-manifest.json gateway.command)");
  }
  // Hard-fail with NAMED messages when the lane's prerequisites are
  // missing (the Wave-7 addendum §7 law — never a raw spawn error, never
  // silent): the real gateway binary, the real web build (web/dist).
  const path = await import("node:path");
  const fs = await import("node:fs/promises");
  const [gatewayCommand] = GATEWAY_COMMAND.split(" ");
  const gatewayAbsolute = path.resolve(WEB_ROOT, gatewayCommand);
  await fs.access(gatewayAbsolute).catch(() => {
    throw new Error(`the real gateway binary is missing: ${gatewayAbsolute} — build it first (cargo build -p flauz-web-gateway; the Lead station has the toolchain — addendum §8), then rerun the FV manifest lane`);
  });
  const webRoot = manifest.gateway?.web_root;
  if (typeof webRoot === "string" && webRoot !== "") {
    const distAbsolute = path.resolve(WEB_ROOT, "..", webRoot);
    await fs.access(distAbsolute).catch(() => {
      throw new Error(`the real web build is missing: ${distAbsolute} — build it first (npm run build in web/), then rerun the FV manifest lane`);
    });
  }
  const driverJourneys = new Map([
    ["fv-e00-gateway-security", je00],
    ["j-01-start-project", j01],
    ["j-02-understand-context", j02],
    ["j-03-recover-reconnect", j03],
    ["j-04-capability-gap", j04],
    ["j-05-add-environment", j05],
    ["j-06-cross-environment", j06],
    ["j-08-takeover-approval-cancellation", j08],
    ["j-09-artifacts-items", j09art],
    ["j-13-collaborate", j13],
    ["j-14-switch-model", j14],
    ["j-15-switch-environment", j15],
    ["j-domain-neutral-research", jNeutral],
    ["j-a11y-responsive", jA11y],
  ]);
  const scenes = Array.isArray(manifest.scenes) ? manifest.scenes : [];
  if (scenes.length === 0) {
    throw new Error("the FV manifest declares no scenes — fix web/lab/fv-manifest.json (every catalog web scene FV-E00..FV-E13 must appear)");
  }
  const defaultRuns = Number.isFinite(Number(manifest.runs)) && Number(manifest.runs) > 0 ? Number(manifest.runs) : 1;
  // The suite is SEQUENTIAL (the journeys were authored that way: j-02
  // opens the Context inspector on the session j-01 created, and so on),
  // so a filtered rerun runs the manifest PREFIX through the requested
  // scene — every scene stays runnable, every record stays honest.
  const filterIndex = scenes.findIndex((scene) => scene.id === SCENE_FILTER);
  const requested = SCENE_FILTER === "" ? scenes : (filterIndex === -1 ? [] : scenes.slice(0, filterIndex + 1));
  if (requested.length === 0) {
    throw new Error(`the FV manifest scene filter matched nothing (--scene ${JSON.stringify(SCENE_FILTER)}); known ids: ${scenes.map((scene) => scene.id).join(", ")}`);
  }
  for (const scene of requested) {
    if (!driverJourneys.has(scene.driver_journey)) {
      throw new Error(`the FV manifest scene ${scene.id} names an unknown driver_journey ${JSON.stringify(scene.driver_journey)} — fix web/lab/fv-manifest.json (known: ${[...driverJourneys.keys()].join(", ")})`);
    }
  }
  const runsPerScene = new Map(requested.map((scene) => {
    const runs = RUNS_OVERRIDE !== "" && Number.isFinite(Number(RUNS_OVERRIDE)) && Number(RUNS_OVERRIDE) > 0
      ? Number(RUNS_OVERRIDE)
      : Number.isFinite(Number(scene.runs)) && Number(scene.runs) > 0
        ? Number(scene.runs)
        : defaultRuns;
    return [scene.id, runs];
  }));
  const maxRuns = Math.max(...runsPerScene.values());

  await rm(stateDir, { recursive: true, force: true });
  await mkdir(stateDir, { recursive: true });

  const browser = await chromium.launch({ headless: true });
  run.adapter.browser = `chromium ${browser.version()} (headless)`;
  run.adapter.transport = "flauz-web-gateway (real; FV manifest mode)";

  // The w6 ×3 convention = N green RUNS, each equivalent to a fresh driver
  // invocation of the sequential suite: a freshly started gateway (fresh
  // supervised runtime), a fresh isolated CODEX_HOME (set on the driver's
  // own environment so every gateway spawn in the run — including the
  // journeys' own kill/restart legs — inherits it), and a fresh browser
  // context in which the requested scenes run in manifest order, sharing
  // the run's page exactly as the classic suite does (j-02 opens the
  // Context inspector on the session j-01 created). Scene N's evidence
  // lands under <evidence_root>/<scene-id>/run-<n>/.
  const failures = [];
  const sceneOutcomes = new Map(requested.map((scene) => [scene.id, "pass"]));
  for (let attempt = 1; attempt <= maxRuns; attempt += 1) {
    if (gateway !== null) {
      killGateway();
      await waitForPortFree();
    }
    const runHome = `${stateDir}/codex-home-run-${attempt}`;
    await mkdir(runHome, { recursive: true });
    process.env.CODEX_HOME = runHome;
    await startGateway();
    const runContext = await browser.newContext({ viewport: { width: 1280, height: 860 } });
    const runPage = await runContext.newPage();
    runPage.on("console", (message) => {
      if (message.type() === "error") {
        process.stderr.write(`[browser:err] ${message.text()}\n`);
      }
    });
    try {
      for (const scene of requested) {
        const runs = runsPerScene.get(scene.id) ?? defaultRuns;
        if (attempt > runs) {
          continue;
        }
        const journey = driverJourneys.get(scene.driver_journey);
        // The run-scoped directory reuses recordJourney's writer
        // unchanged: the wrapped definition's id carries
        // scene-id/run-<n>, so the record lands under
        // <evidence_root>/<scene-id>/run-<n>/.
        const runScoped = { ...journey, id: `${scene.id}/run-${attempt}` };
        try {
          const record = await recordJourney(runPage, runScoped);
          process.stdout.write(`fv:   ${scene.id} run ${attempt}/${runs} → ${record.outcome}\n`);
          if (record.outcome !== "pass") {
            sceneOutcomes.set(scene.id, "fail");
          }
        } catch (error) {
          sceneOutcomes.set(scene.id, "fail");
          process.stderr.write(`fv:   ${scene.id} run ${attempt}/${runs} CRASHED: ${error.message}\n`);
        }
      }
    } finally {
      await runContext.close().catch(() => {});
      killGateway();
      await waitForPortFree().catch(() => {});
    }
  }
  for (const scene of requested) {
    process.stdout.write(`fv: scene ${scene.id} → ${sceneOutcomes.get(scene.id)}\n`);
    if (sceneOutcomes.get(scene.id) !== "pass") {
      failures.push(scene.id);
    }
  }

  run.finished_at = new Date().toISOString();
  run.kind = "flauz.web-fv.run";
  run.outcome = failures.length === 0 ? "pass" : "fail";
  run.failed_scenes = failures;
  run.manifest = {
    pinned_base: manifest.pinned_base ?? null,
    gateway_mode: manifest.gateway?.mode ?? null,
    runs: defaultRuns,
    scenes_requested: requested.map((scene) => scene.id),
  };
  await mkdir(OUT_ROOT, { recursive: true });
  await writeFile(`${OUT_ROOT}/RUN.json`, `${JSON.stringify(run, null, 2)}\n`, "utf8");

  await browser.close();
  killGateway();
  process.stdout.write(
    `fv: run ${run.run_id} → ${run.outcome}${failures.length === 0 ? "" : ` (failed: ${failures.join(", ")})`}\n`,
  );
  process.exit(failures.length === 0 ? 0 : 1);
}

// ---------------------------------------------------------------------------
// The run.

async function main() {
  if (manifest !== null) {
    await runManifestMode();
    return;
  }
  await rm(stateDir, { recursive: true, force: true });
  await mkdir(stateDir, { recursive: true });
  await rm(OUT_ROOT, { recursive: true, force: true });
  await startGateway();

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 860 } });
  const page = await context.newPage();
  page.on("console", (message) => {
    if (message.type() === "error") {
      process.stderr.write(`[browser:err] ${message.text()}\n`);
    }
  });
  run.adapter.browser = `chromium ${browser.version()} (headless)`;

  const failures = [];
  for (const journey of [j01, j02, j03, j04, j05, j06, j08, j13, j14, j15, j09art, jNeutral, jA11y]) {
    process.stdout.write(`lab: running ${journey.journeyId} (${journey.name})…\n`);
    try {
      const record = await recordJourney(page, journey);
      process.stdout.write(`lab:   ${journey.journeyId} → ${record.outcome}\n`);
      if (record.outcome !== "pass") {
        failures.push(journey.journeyId);
      }
    } catch (error) {
      failures.push(journey.journeyId);
      process.stderr.write(`lab:   ${journey.journeyId} CRASHED: ${error.message}\n`);
    }
  }

  run.finished_at = new Date().toISOString();
  run.outcome = failures.length === 0 ? "pass" : "fail";
  run.failed_journeys = failures;
  await mkdir(OUT_ROOT, { recursive: true });
  await writeFile(`${OUT_ROOT}/RUN.json`, `${JSON.stringify(run, null, 2)}\n`, "utf8");

  await browser.close();
  killGateway();
  process.stdout.write(
    `lab: run ${run.run_id} → ${run.outcome}${failures.length === 0 ? "" : ` (failed: ${failures.join(", ")})`}\n`,
  );
  process.exit(failures.length === 0 ? 0 : 1);
}

main().catch((error) => {
  killGateway();
  console.error(`lab: fatal: ${error.message}`);
  process.exit(1);
});
