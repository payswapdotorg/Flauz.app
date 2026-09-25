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
// Usage: node lab/journeys.mjs [--gateway "<command>"] [--out <dir>] [--keep-state]

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
const GATEWAY_COMMAND = flagValue("--gateway", "");
const OUT_ROOT = flagValue("--out", EVIDENCE_ROOT_DEFAULT);
const PORT = Number(flagValue("--port", "8791"));
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
    const [command, ...rest] = GATEWAY_COMMAND.split(" ");
    gateway = spawn(command, [...rest.split(" ").filter(Boolean), ...(usingMock ? [] : [])], {
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
    lab_surfaces: ["web.shell", "web.connection", "web.sessions", "web.composer", "web.approvals"],
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
  name: "Discover a capability gap (named error + unlock path, web foundation variant)",
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

    // The environments drawer names the unlock path.
    entry = step("open the session's Environments surface for the unlock path");
    await page.click('[data-testid="session-row"]');
    await page.click('[data-testid="environments-button"]');
    const drawerText = await page.textContent('[data-testid="environments-drawer"]').catch(() => "");
    const unlockNamed = drawerText.includes("Adding environments") || drawerText.includes("attach");
    assert(entry, "a4", "capability.unlock_path_named", unlockNamed ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "02-environments-unlock-path"));
    await page.click('[data-testid="environments-button"]');
  },
};

const j05 = {
  id: "j-05-add-environment",
  journeyId: "J-05",
  name: "Add another environment/site (honest foundation empty state)",
  productJourney: "Task → Environments → Add environment/site — the surface is discoverable with a truthful not-yet state (WEB-002 owns the flows).",
  async drive({ page, record, step, assert, capture }) {
    let entry = step("open the Environments drawer from the session surface");
    await page.click('[data-testid="environments-button"]');
    const drawer = (await page.locator('[data-testid="environments-drawer"]').count()) > 0;
    assert(entry, "a1", "environments.surface", drawer ? "visible" : "absent", "visible");
    const text = await page.textContent('[data-testid="environments-drawer"]');
    const emptyStateHonest = text.includes("No environments attached");
    assert(entry, "a2", "environments.empty_state", emptyStateHonest ? "honest" : "missing", "honest");
    const nextStep = text.includes("agents can act");
    assert(entry, "a3", "environments.next_step", nextStep ? "named" : "unnamed", "named");
    entry.frames.push(await capture(entry, "01-environments-empty"));

    // Keyboard path: Ctrl+/ opens the shortcut sheet; Escape closes.
    entry = step("verify the keyboard path (Ctrl+/ shortcuts sheet, Escape restore)");
    await page.keyboard.press("Control+/");
    const sheetVisible = (await page.locator(".flauz-overlay").count()) > 0;
    assert(entry, "a4", "keyboard.shortcuts_sheet", sheetVisible ? "visible" : "absent", "visible");
    entry.frames.push(await capture(entry, "02-keyboard-sheet"));
    await page.keyboard.press("Escape");
    const sheetClosed = (await page.locator(".flauz-overlay").count()) === 0;
    assert(entry, "a5", "keyboard.escape_closes", sheetClosed ? "closed" : "open", "closed");
  },
};

// ---------------------------------------------------------------------------
// The run.

async function main() {
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
  for (const journey of [j01, j02, j03, j04, j05]) {
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
