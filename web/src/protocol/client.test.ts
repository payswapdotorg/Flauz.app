// @vitest-environment node
// The gateway session client tests (WEB-001): the handshake, the typed
// request/response round-trip over a real WebSocket server, server
// request routing (approvals are never auto-resolved), and the named
// failure path.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { AddressInfo } from "node:net";
import { WebSocketServer } from "ws";
import { GatewaySession } from "../protocol/client";
import { parseGatewayControlMessage } from "../gateway/protocol";

interface ServerFrameLog {
  method?: string;
  id?: unknown;
  params?: unknown;
  result?: unknown;
  error?: unknown;
}

let server: WebSocketServer;
let port = 0;
const received: ServerFrameLog[] = [];

beforeAll(async () => {
  server = new WebSocketServer({ port: 0 });
  await new Promise<void>((resolve) => {
    server.once("listening", resolve);
  });
  const address = server.address();
  if (typeof address === "string" || address === null) {
    throw new Error("the test WebSocket server did not bind a port");
  }
  port = (address as AddressInfo).port;
  server.on("connection", (socket) => {
    let claimed = false;
    socket.on("message", (data) => {
      const value = JSON.parse(data.toString()) as Record<string, unknown>;
      received.push(value);
      if (claimed) {
        if (typeof value["method"] === "string" && value["method"] === "thread/list") {
          socket.send(
            JSON.stringify({
              id: value["id"],
              result: {
                data: [{ id: "thread-1", preview: "test", sessionId: "s", cwd: "/", createdAt: 1, updatedAt: 2 }],
                nextCursor: null,
              },
            }),
          );
          return;
        }
        if (typeof value["method"] === "string") {
          // Unknown methods are rejected with a named error.
          socket.send(
            JSON.stringify({
              id: value["id"],
              error: { code: -32601, message: `method not found: ${String(value["method"])}` },
            }),
          );
          return;
        }
        // A response to a server-initiated request: acknowledge it.
        socket.send(
          JSON.stringify({ type: "gateway.error", error: { code: -1, message: "unused" } }),
        );
        return;
      }
      const control = parseGatewayControlMessage(value);
      if (control?.type === "session.claim") {
        claimed = true;
        socket.send(
          JSON.stringify({ type: "session.claimed", sessionId: "gwsess_test_1", protocolVersion: 1 }),
        );
        socket.send(JSON.stringify({ type: "gateway.state", state: "connected", reason: "attached" }));
        // A server-initiated approval request.
        socket.send(
          JSON.stringify({
            id: "srv-1",
            method: "item/commandExecution/requestApproval",
            params: { command: "cargo test" },
          }),
        );
      } else {
        socket.send(
          JSON.stringify({
            type: "session.denied",
            code: "handshake_required",
            message: "claim a session before sending app-server frames",
          }),
        );
        socket.close(1008, "gateway: handshake_required");
      }
    });
  });
});

afterAll(() => {
  server.close();
});

function url(): string {
  return `ws://127.0.0.1:${port}/ws`;
}

describe("the gateway session client", () => {
  it("refuses to complete when the handshake is denied by name", async () => {
    // A raw client that sends a JSON-RPC frame BEFORE claiming.
    const raw = new WebSocket(url());
    const denial = await new Promise<Record<string, unknown>>((resolve) => {
      raw.addEventListener("message", (event) => {
        resolve(JSON.parse(String(event.data)) as Record<string, unknown>);
      });
      raw.addEventListener("open", () => {
        raw.send(JSON.stringify({ method: "thread/list", id: 1 }));
      });
    });
    expect(denial["type"]).toBe("session.denied");
    expect(denial["code"]).toBe("handshake_required");
    raw.close();
  });

  it("claims the session, round-trips a typed request, and routes server requests", async () => {
    const session = await GatewaySession.connect(url());
    expect(session.sessionId).toBe("gwsess_test_1");

    const serverRequest = await new Promise<{ id: unknown; method: string; params: unknown }>((resolve) => {
      session.onServerRequest((request) => resolve(request));
    });
    expect(serverRequest.method).toBe("item/commandExecution/requestApproval");
    expect((serverRequest.params as { command?: string }).command).toBe("cargo test");

    const result = await session.request("thread/list", {
      limit: 20,
      sortKey: "recency_at",
      sortDirection: "desc",
      useStateDbOnly: true,
    });
    expect(result.data.length).toBe(1);
    expect(result.data[0]?.id).toBe("thread-1");

    // The approval decision routes back with the server's id (never
    // auto-resolved — the caller decides).
    session.respondServerRequest(serverRequest.id as number | string, { decision: "accept" });
    await new Promise<void>((resolve) => {
      const started = received.length;
      const check = () => {
        const tail = received.slice(started);
        if (tail.some((frame) => frame.id === "srv-1" && frame.result !== undefined)) {
          resolve();
        } else {
          setTimeout(check, 10);
        }
      };
      check();
    });
    session.close();
  });

  it("rejects typed requests with the named error message", async () => {
    const session = await GatewaySession.connect(url());
    await expect(session.request("account/read", { refreshToken: false })).rejects.toThrow(
      /method not found/,
    );
    session.close();
  });

  it("surfaces a named close to registered listeners", async () => {
    const session = await GatewaySession.connect(url());
    const closed = new Promise<{ code: number; reason: string }>((resolve) => {
      session.onClose((code, reason) => resolve({ code, reason }));
    });
    const client = (server.clients as Set<{ close: (code: number, reason: string) => void }>);
    for (const peer of client) {
      peer.close(1001, "server going away");
    }
    const event = await closed;
    expect(event.code).toBe(1001);
    expect(event.reason).toContain("going away");
  });
});
