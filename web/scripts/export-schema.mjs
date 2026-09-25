// WEB-001 — the app-server schema export procedure.
//
// Runs the official codex binary's experimental schema export
// (CODEX_APP_SERVER_SCHEMA_EXPERIMENTAL=1 codex app-server) and captures
// the emitted JSON schema document into src/protocol/schema.json.
//
// The schema snapshot is the frontend's frozen protocol contract (Wave-6
// kernel addendum §3): web/ consumes ONLY types generated from it
// (scripts/generate-protocol-types.mjs); hand-written protocol types are
// forbidden.
//
// Usage:
//   CODEX_RS_CODEX_BIN=/path/to/codex npm run protocol:export
//   npm run protocol:generate
//
// The codex binary is resolved exactly like the Rust side: the
// CODEX_RS_CODEX_BIN environment variable, then the PATH.

import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import process from "node:process";

const MAX_CAPTURE_BYTES = 64 * 1024 * 1024;
const EXPORT_TIMEOUT_MS = 30_000;
const SCHEMA_PATH = resolve(import.meta.dirname, "../src/protocol/schema.json");

function resolveCodexBinary() {
  const configured = process.env.CODEX_RS_CODEX_BIN;
  if (configured && configured.trim() !== "") {
    return { command: configured, args: [] };
  }
  const onWindows = process.platform === "win32";
  return { command: onWindows ? "codex.exe" : "codex", args: [] };
}

function fail(message) {
  console.error(`protocol:export: ${message}`);
  process.exit(1);
}

function looksLikeSchemaDocument(value) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const keys = Object.keys(value);
  const markers = ["methods", "types", "definitions", "$defs", "$schema", "notifications", "requests"];
  return markers.some((marker) => keys.includes(marker));
}

async function main() {
  const { command, args } = resolveCodexBinary();
  const child = spawn(command, [...args, "app-server"], {
    env: { ...process.env, CODEX_APP_SERVER_SCHEMA_EXPERIMENTAL: "1" },
    stdio: ["pipe", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  let overflowed = false;
  child.stdout.on("data", (chunk) => {
    if (stdout.length + chunk.length > MAX_CAPTURE_BYTES) {
      overflowed = true;
      child.kill();
      return;
    }
    stdout += chunk.toString("utf8");
  });
  child.stderr.on("data", (chunk) => {
    if (stderr.length < 16 * 1024) {
      stderr += chunk.toString("utf8");
    }
  });
  // The export prints the schema document on stdout; close stdin so an
  // interactive app-server exits instead of waiting for requests.
  child.stdin.end();
  const exited = new Promise((resolveExit) => {
    child.once("exit", (code, signal) => resolveExit({ code, signal }));
  });
  const timer = new Promise((resolveTimeout) => {
    const handle = setTimeout(() => resolveTimeout("timeout"), EXPORT_TIMEOUT_MS);
    child.once("exit", () => clearTimeout(handle));
  });
  const outcome = await Promise.race([exited, timer]);
  if (outcome === "timeout") {
    child.kill();
  }
  if (overflowed) {
    fail(`the schema export exceeded the ${MAX_CAPTURE_BYTES}-byte capture bound`);
  }
  const text = stdout.trim();
  if (text === "") {
    fail(
      `the schema export produced no stdout (codex binary: ${command}; stderr: ${stderr.trim().slice(0, 400) || "empty"})`,
    );
  }
  let document;
  try {
    document = JSON.parse(text);
  } catch (error) {
    fail(`the schema export was not valid JSON: ${error.message}`);
  }
  if (!looksLikeSchemaDocument(document)) {
    fail(
      "the schema export did not look like a schema document (no methods/types/definitions markers); " +
        "the generator may need a normalizer extension for this export shape",
    );
  }
  await mkdir(dirname(SCHEMA_PATH), { recursive: true });
  await writeFile(SCHEMA_PATH, `${JSON.stringify(document, null, 2)}\n`, "utf8");
  console.log(`protocol:export: captured the app-server schema export to src/protocol/schema.json`);
}

main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
