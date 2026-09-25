// WEB-002 — the web parity-lab mock gateway.
//
// A faithful Node implementation of the flauz-web-gateway WebSocket
// contract (session.claim handshake, denial laws, gateway.state events,
// transparent JSON-RPC bridging) hosting an in-process fake app-server
// that implements the frozen protocol slice (the schema snapshot's
// methods — NO methods the snapshot does not carry, so the lab evidences
// the named gaps honestly) with the app-server wire dialect
// ({method,id,params} / {id,result|error}, no jsonrpc field).
//
// The WORKER SANDBOX LACKS THE RUST TOOLCHAIN (honestly declared in the
// WEB-001 report): this mock stands in for the Rust gateway so the
// journey lab can capture real browser evidence of the REAL web build.
// The Lead's gate station runs the same lab against the real Rust
// gateway (lab/journeys.mjs --gateway). The protocol contract is
// identical (see crates/flauz-web-gateway/src/protocol.rs and
// GET /gateway-protocol.json).
//
// Usage: node lab/mock-gateway.mjs --port 8791 --root ../dist --state lab-state.json

import { createServer } from "node:http";
import { readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";
import process from "node:process";
import { WebSocketServer } from "ws";

const args = process.argv.slice(2);
function flagValue(name, fallback) {
  const index = args.indexOf(name);
  return index !== -1 && args[index + 1] !== undefined ? args[index + 1] : fallback;
}

const PORT = Number(flagValue("--port", "8791"));
const ROOT = resolve(flagValue("--root", "dist"));
const STATE_FILE = flagValue("--state", "");

// ---------------------------------------------------------------------------
// The fake app-server state (persisted across kill/restart for J-03).

const nowSeconds = () => Math.floor(Date.now() / 1000);

function defaultState() {
  return {
    account: null, // signed out initially
    logins: {},
    threads: [],
    turns: {},
    // The per-thread execution choices the client sent (model/effort/
    // cwd — the reflection the capability surfaces assert against).
    threadExec: {},
  };
}

let state = defaultState();

// The running (pending) long turns, in memory only — never persisted.
const pendingTurns = new Map();

async function loadState() {
  if (STATE_FILE === "" || !existsSync(STATE_FILE)) {
    return;
  }
  try {
    const parsed = JSON.parse(await readFile(STATE_FILE, "utf8"));
    state = { ...defaultState(), ...parsed };
    console.log(`[mock-gateway] loaded state: ${state.threads.length} thread(s)`);
  } catch (error) {
    console.warn(`[mock-gateway] state load failed: ${error.message}`);
  }
}

let saveScheduled = false;
function persistState() {
  if (STATE_FILE === "" || saveScheduled) {
    return;
  }
  saveScheduled = true;
  setTimeout(() => {
    saveScheduled = false;
    writeFile(STATE_FILE, JSON.stringify(state), "utf8").catch(() => {});
  }, 30);
}

// ---------------------------------------------------------------------------
// The fake app-server surface (the WEB-001 protocol slice).

function threadSummary(thread) {
  return {
    id: thread.id,
    sessionId: `sess-${thread.id}`,
    forkedFromId: null,
    parentThreadId: null,
    preview: thread.preview,
    name: thread.name,
    cwd: thread.cwd ?? "/tmp/flauz-lab",
    createdAt: thread.createdAt,
    updatedAt: thread.updatedAt,
    recencyAt: thread.updatedAt,
    status: {},
    gitInfo: null,
    turns: [],
  };
}

/** Reflects the execution choices the client rode on its turns (the
 * protocol's own model/effort reporting on thread/resume). */
function reportedExec(threadId) {
  const exec = state.threadExec[threadId] ?? {};
  return {
    model: typeof exec.model === "string" ? exec.model : "gpt-5.6-sol",
    reasoningEffort: typeof exec.effort === "string" ? exec.effort : "high",
  };
}

function turnsPage(threadId) {
  return {
    data: (state.turns[threadId] ?? []).map((entry) => ({ id: entry.id, item: entry })),
    nextCursor: null,
    backwardsCursor: null,
  };
}

function handleAppServerRequest(socket, request) {
  const { method, id, params } = request;
  const respond = (result) => socket.send(JSON.stringify({ id, result }));
  const respondError = (code, message) => socket.send(JSON.stringify({ id, error: { code, message } }));
  const notify = (method, params) => socket.send(JSON.stringify({ method, params }));
  switch (method) {
    case "account/read":
      respond({
        account: state.account,
        requiresOpenaiAuth: state.account === null,
      });
      return;
    case "getAuthStatus":
      respond({
        authMethod: state.account === null ? null : "chatgpt",
        accountId: state.account === null ? null : "acct_lab",
        requiresOpenaiAuth: state.account === null,
      });
      return;
    case "account/login/start": {
      if (params?.type === "chatgpt") {
        const loginId = `login-${Date.now().toString(36)}`;
        state.logins[loginId] = { startedAt: nowSeconds() };
        respond({
          type: "chatgpt",
          loginId,
          authUrl: `http://127.0.0.1:${PORT}/lab-auth/${loginId}`,
        });
        return;
      }
      if (params?.type === "apiKey") {
        // The key is used and immediately discarded; it is never logged
        // or stored (the credential law).
        if (typeof params.apiKey !== "string" || params.apiKey.trim() === "") {
          respondError(-32002, "the API key was rejected by the agent runtime (empty)");
          return;
        }
        state.account = { type: "apiKey" };
        persistState();
        notify("account/login/completed", {});
        respond({ type: "apiKey" });
        return;
      }
      respondError(-32602, `unsupported login type: ${String(params?.type)}`);
      return;
    }
    case "account/logout":
      state.account = null;
      persistState();
      respond({});
      return;
    case "thread/list":
      respond({
        data: state.threads.map(threadSummary),
        nextCursor: null,
        backwardsCursor: null,
      });
      return;
    case "thread/search":
      respond({
        data: state.threads
          .filter((thread) => thread.preview.toLowerCase().includes(String(params?.searchTerm ?? "").toLowerCase()))
          .map((thread) => ({ thread: threadSummary(thread), snippet: thread.preview.slice(0, 80) })),
        nextCursor: null,
      });
      return;
    case "thread/read":
    case "thread/resume": {
      const thread = state.threads.find((candidate) => candidate.id === params?.threadId);
      if (thread === undefined) {
        respondError(-32001, `session not found: ${String(params?.threadId)}`);
        return;
      }
      thread.updatedAt = nowSeconds();
      const exec = reportedExec(thread.id);
      respond({
        thread: threadSummary(thread),
        initialTurnsPage: turnsPage(thread.id),
        model: exec.model,
        reasoningEffort: exec.reasoningEffort,
        serviceTier: null,
      });
      return;
    }
    case "thread/turns/list":
      respond(turnsPage(String(params?.threadId)));
      return;
    case "turn/start": {
      const text = Array.isArray(params?.input)
        ? params.input
            .map((item) =>
              item?.type === "text" ? item.text : item?.type === "skill" ? `@${item.name}` : `[${item?.type}]`,
            )
            .join(" ")
        : "";
      // A named capability-gap failure when the objective asks for a
      // capability the runtime cannot provide (J-04): the failure is a
      // protocol-shaped JSON-RPC error naming cause + recovery.
      if (text.toLowerCase().includes("gpu")) {
        respondError(
          -32003,
          "turn failed: the task needs a GPU environment, but no environment with `gpu` is attached to this task (capability gap: gpu). Recovery: attach an environment that provides `gpu`, or restate the objective to avoid it.",
        );
        return;
      }
      let thread = state.threads.find((candidate) => candidate.id === params?.threadId);
      if (thread === undefined) {
        thread = {
          id: String(params?.threadId),
          name: text.length > 48 ? `${text.slice(0, 48)}…` : text,
          preview: text,
          createdAt: nowSeconds(),
          updatedAt: nowSeconds(),
        };
        state.threads.unshift(thread);
        state.turns[thread.id] = [];
      }
      thread.updatedAt = nowSeconds();
      // The execution choices the client rode on this turn (the
      // model/effort/cwd reflection the capability surfaces assert
      // against — the protocol's own fields, never invented ones).
      const previousExec = state.threadExec[thread.id] ?? {};
      const requestedCwd = typeof params?.cwd === "string" && params.cwd.trim() !== "" ? params.cwd.trim() : null;
      state.threadExec[thread.id] = {
        model:
          typeof params?.model === "string" && params.model.trim() !== ""
            ? params.model.trim()
            : (previousExec.model ?? null),
        effort:
          typeof params?.effort === "string" && params.effort.trim() !== ""
            ? params.effort.trim()
            : (previousExec.effort ?? null),
        cwd: requestedCwd ?? (previousExec.cwd ?? null),
      };
      if (requestedCwd !== null) {
        thread.cwd = requestedCwd;
      }
      const turnId = `turn-${Date.now().toString(36)}`;
      notify("turn/started", { threadId: thread.id, turn: { id: turnId, status: "running" } });
      // A long-running turn: completes only after several seconds, and
      // honours turn/interrupt with a NAMED cancelled terminal state
      // (the cancellation-propagation honesty law).
      if (text.toLowerCase().includes("long")) {
        const timer = setTimeout(() => {
          pendingTurns.delete(thread.id);
          notify("turn/completed", { threadId: thread.id, turn: { id: turnId, status: "completed" } });
        }, 4000);
        pendingTurns.set(thread.id, { timer, turnId, threadId: thread.id });
        persistState();
        respond({ threadId: thread.id, turn: { id: turnId, status: "running" } });
        return;
      }
      // The agent needs a decision mid-turn (the approval path).
      if (text.toLowerCase().includes("install")) {
        socket.send(
          JSON.stringify({
            id: `srv-${turnId}`,
            method: "item/commandExecution/requestApproval",
            params: { threadId: thread.id, turnId, command: "npm install --no-audit" },
          }),
        );
      }
      notify("item/agentMessage/delta", { threadId: thread.id, turnId, itemId: `${turnId}-m`, delta: "Planning " });
      notify("item/agentMessage/delta", { threadId: thread.id, turnId, itemId: `${turnId}-m`, delta: "the layout…" });
      const message = `Done: ${text.charAt(0).toUpperCase()}${text.slice(1)} — here is the plan.`;
      notify("item/completed", {
        threadId: thread.id,
        turnId,
        item: { type: "agentMessage", text: message },
      });
      state.turns[thread.id].push({ id: `${turnId}-m`, type: "agentMessage", text: message });
      // A command the runtime executed (an install turn reports the
      // command it ran — the artifacts surface's command item).
      if (text.toLowerCase().includes("install")) {
        const commandItem = { id: `${turnId}-c`, type: "commandExecution", command: "npm install --no-audit" };
        notify("item/completed", { threadId: thread.id, turnId, item: { type: commandItem.type, command: commandItem.command } });
        state.turns[thread.id].push(commandItem);
      }
      // A file the runtime changed (a planning/calendar turn reports the
      // file change — the artifacts surface's file-change item).
      if (text.toLowerCase().includes("calendar") || text.toLowerCase().includes("report")) {
        const fileItem = { id: `${turnId}-f`, type: "fileChange", text: "garden-plan.md — 14 lines changed" };
        notify("item/completed", { threadId: thread.id, turnId, item: { type: fileItem.type, text: fileItem.text } });
        state.turns[thread.id].push(fileItem);
      }
      persistState();
      notify("turn/completed", { threadId: thread.id, turn: { id: turnId, status: "completed" } });
      respond({ threadId: thread.id, turn: { id: turnId, status: "completed" } });
      return;
    }
    case "turn/interrupt": {
      const pending = pendingTurns.get(String(params?.threadId));
      if (pending !== undefined) {
        clearTimeout(pending.timer);
        pendingTurns.delete(String(params?.threadId));
        // The named cancelled terminal state — cancellation is never a
        // silent disappearance and never a fake failure.
        notify("turn/completed", {
          threadId: pending.threadId,
          turn: { id: pending.turnId, status: "cancelled", error: { message: "stopped by you from the web client" } },
        });
      }
      respond({});
      return;
    }
    default:
      respondError(-32601, `method not found: ${method}`);
      return;
  }
}

// ---------------------------------------------------------------------------
// The gateway protocol layer (the Rust gateway's contract).

const sessions = new Set();

function handleConnection(socket) {
  let claimed = false;
  sessions.add(socket);
  socket.on("message", (data) => {
    let value;
    try {
      value = JSON.parse(data.toString());
    } catch {
      socket.send(
        JSON.stringify({ type: "gateway.error", error: { code: -32000, message: "the frame was not valid JSON" } }),
      );
      return;
    }
    if (claimed) {
      if (typeof value.method === "string") {
        if (value.id !== undefined) {
          handleAppServerRequest(socket, value);
        } else {
          // A client notification: accepted and ignored by the fake.
        }
        return;
      }
      if (value.id !== undefined && (value.result !== undefined || value.error !== undefined)) {
        // A response to a server-initiated request (an approval
        // decision): consumed honestly.
        return;
      }
      socket.send(
        JSON.stringify({
          type: "gateway.error",
          error: { code: -32000, message: "the frame is neither a gateway control message nor an app-server JSON-RPC frame" },
        }),
      );
      return;
    }
    if (value?.type === "session.claim") {
      claimed = true;
      socket.send(
        JSON.stringify({ type: "session.claimed", sessionId: `gwsess_${Math.random().toString(16).slice(2, 18)}`, protocolVersion: 1 }),
      );
      socket.send(
        JSON.stringify({ type: "gateway.state", state: "connected", reason: "the supervised app-server is attached and initialized" }),
      );
      return;
    }
    // THE auth handshake refusal law.
    socket.send(
      JSON.stringify({
        type: "session.denied",
        code: "handshake_required",
        message: "claim a session before sending app-server frames",
      }),
    );
    socket.close(1008, "gateway: handshake_required");
  });
  socket.on("close", () => {
    sessions.delete(socket);
  });
}

// ---------------------------------------------------------------------------
// The combined server: static hosting (SPA fallback) + /ws upgrade + the
// lab fake-auth completion page.

const CONTENT_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".map": "application/json; charset=utf-8",
};

const server = createServer(async (request, response) => {
  const url = new URL(request.url ?? "/", `http://127.0.0.1:${PORT}`);
  if (url.pathname === "/healthz") {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ status: "ok", sessions: sessions.size, transport: "mock-gateway" }));
    return;
  }
  if (url.pathname === "/gateway-protocol.json") {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        v: 1,
        kind: "flauz.web-gateway.protocol",
        transport: "mock (see crates/flauz-web-gateway for the real descriptor)",
      }),
    );
    return;
  }
  const auth = /^\/lab-auth\/(.+)$/.exec(url.pathname);
  if (auth?.[1]) {
    const loginId = auth[1];
    if (state.logins[loginId] !== undefined) {
      state.account = { type: "chatgpt", email: "researcher@example.test", planType: "pro" };
      persistState();
      for (const socket of sessions) {
        socket.send(JSON.stringify({ method: "account/login/completed", params: {} }));
      }
    }
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end(
      `<!doctype html><meta charset="utf-8"><title>Flauz lab auth</title><body style="font-family:sans-serif;padding:40px"><h1>Flauz lab authorization complete</h1><p>You can return to the Flauz tab.</p></body>`,
    );
    return;
  }
  // Static hosting with SPA fallback (mirrors the Rust static layer).
  const pathname = url.pathname === "/" ? "/index.html" : url.pathname;
  const candidate = normalize(join(ROOT, `.${pathname}`));
  if (!candidate.startsWith(ROOT)) {
    response.writeHead(404, { "content-type": "text/plain" });
    response.end("not found");
    return;
  }
  const serveFile = async (path) => {
    const body = await readFile(path);
    response.writeHead(200, {
      "content-type": CONTENT_TYPES[extname(path)] ?? "application/octet-stream",
      "cache-control": "no-store",
    });
    response.end(body);
  };
  try {
    await serveFile(candidate);
    return;
  } catch {
    if (!extname(pathname)) {
      try {
        await serveFile(join(ROOT, "index.html"));
        return;
      } catch {
        // fall through to 404
      }
    }
    response.writeHead(404, { "content-type": "text/plain" });
    response.end("not found");
  }
});

const wss = new WebSocketServer({ noServer: true });
server.on("upgrade", (request, socket, head) => {
  const url = new URL(request.url ?? "/", `http://127.0.0.1:${PORT}`);
  if (url.pathname !== "/ws") {
    socket.destroy();
    return;
  }
  wss.handleUpgrade(request, socket, head, (ws) => {
    handleConnection(ws);
  });
});

await loadState();
server.listen(PORT, "127.0.0.1", () => {
  console.log(`[mock-gateway] listening on http://127.0.0.1:${PORT} (root ${ROOT}, transport mock-gateway)`);
});
