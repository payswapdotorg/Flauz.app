// The gateway session client: the authenticated-session WebSocket
// handshake, the typed JSON-RPC bridge, server-request routing, and the
// raw event stream consumed by the connection-state machine.
//
// ZERO protocol types are defined here: app-server shapes come from
// src/protocol/generated.ts (the schema-export pipeline) and gateway
// envelope shapes from src/gateway/protocol.ts.

import {
  type ProtocolMethods,
  type ProtocolNotificationName,
  type ProtocolServerRequestName,
} from "./generated";
import {
  GATEWAY_PROTOCOL_VERSION,
  type GatewayControlMessage,
  type SessionClaimed,
  type SessionDenied,
  parseGatewayControlMessage,
} from "../gateway/protocol";

export type { GatewayControlMessage };

/** A raw frame from the app-server side: a notification or a
 * server-initiated request, or a response to a bridged request. */
export type BridgeEvent =
  | { kind: "notification"; method: ProtocolNotificationName | string; params: unknown }
  | { kind: "serverRequest"; id: number | string; method: ProtocolServerRequestName | string; params: unknown }
  | { kind: "response"; id: number; result?: unknown; error?: { code: number; message: string } };

/** A server request the caller must respond to (approvals, …). */
export interface PendingServerRequest {
  id: number | string;
  method: ProtocolServerRequestName | string;
  params: unknown;
}

const REQUEST_TIMEOUT_MS = 15_000;
const MAX_PENDING_NOTIFICATIONS = 512;

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

/**
 * One authenticated gateway session: claim → bridge.
 *
 * The client is deliberately transport-thin: it claims the session,
 * routes responses to awaiting callers, forwards notifications to a
 * bounded listener, and surfaces server-initiated requests for explicit
 * user decisions (never auto-resolved). Reconnection POLICY lives in
 * the connection-state machine, not here.
 */
export class GatewaySession {
  private socket: WebSocket;
  private nextRequestId = 1;
  private readonly pending = new Map<number, PendingRequest>();
  private readonly notificationListeners = new Set<(event: BridgeEvent) => void>();
  private readonly closeListeners = new Set<(code: number, reason: string) => void>();
  private readonly gatewayListeners = new Set<(message: GatewayControlMessage) => void>();
  /** Control messages seen before the first listener attached (bounded
   * replay so early gateway.state events are never lost). */
  private readonly gatewayMessageBuffer: GatewayControlMessage[] = [];
  private readonly serverRequests: PendingServerRequest[] = [];
  private readonly serverRequestListeners = new Set<(request: PendingServerRequest) => void>();
  private droppedNotifications = 0;
  private closed = false;

  private constructor(socket: WebSocket) {
    this.socket = socket;
  }

  /**
   * Opens a WebSocket to the gateway, completes the `session.claim`
   * handshake, and resolves once the gateway acknowledges the claim.
   *
   * @param url the gateway WebSocket URL (ws:// or wss://).
   * @param token an operator-provisioned session token when the gateway
   *   requires one. Tokens are sent once in the handshake and never
   *   logged or persisted.
   * @param interrupt an AbortSignal to cancel the connect attempt.
   */
  static async connect(url: string, token?: string, interrupt?: AbortSignal): Promise<GatewaySession> {
    const socket = await openSocket(url, interrupt);
    const session = new GatewaySession(socket);
    socket.addEventListener("message", (event) => {
      session.handleMessage(event.data);
    });
    socket.addEventListener("close", (event) => {
      session.handleClose(event.code, event.reason);
    });
    socket.addEventListener("error", () => {
      // The close event always follows; nothing to do here.
    });
    const claimMessage = await new Promise<SessionClaimed | SessionDenied>((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error("the gateway did not acknowledge the session claim in time"));
      }, 15_000);
      const onMessage = (message: GatewayControlMessage) => {
        if (message.type === "session.claimed" || message.type === "session.denied") {
          clearTimeout(timer);
          session.gatewayListeners.delete(onMessage);
          resolve(message);
        }
      };
      session.gatewayListeners.add(onMessage);
      session.sendRaw({ type: "session.claim", ...(token === undefined ? {} : { token }) });
    });
    if (claimMessage.type !== "session.claimed") {
      session.close();
      throw new Error(`the gateway refused the session: ${claimMessage.code} — ${claimMessage.message}`);
    }
    if (claimMessage.protocolVersion !== GATEWAY_PROTOCOL_VERSION) {
      session.close();
      throw new Error(
        `the gateway speaks protocol version ${claimMessage.protocolVersion}; this client requires ${GATEWAY_PROTOCOL_VERSION}`,
      );
    }
    return session;
  }

  get sessionId(): string | null {
    return this.claimedSessionId;
  }

  private claimedSessionId: string | null = null;

  /** Issues one typed JSON-RPC request through the bridge. */
  async request<M extends keyof ProtocolMethods>(
    method: M,
    ...params: ProtocolMethods[M]["params"] extends undefined
      ? []
      : [ProtocolMethods[M]["params"]]
  ): Promise<ProtocolMethods[M]["result"]> {
    if (this.closed) {
      throw new Error("the gateway session is closed");
    }
    const id = this.nextRequestId;
    this.nextRequestId += 1;
    const paramsValue = params.length === 0 ? undefined : params[0];
    const frame: Record<string, unknown> = { method, id };
    if (paramsValue !== undefined) {
      frame["params"] = paramsValue;
    }
    const reply = await this.sendRequestFrame(id, frame);
    return reply as ProtocolMethods[M]["result"];
  }

  private sendRequestFrame(id: number, frame: Record<string, unknown>): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`the request (id ${id}) timed out`));
      }, REQUEST_TIMEOUT_MS);
      this.pending.set(id, {
        resolve: (value) => {
          clearTimeout(timer);
          resolve(value);
        },
        reject: (error) => {
          clearTimeout(timer);
          reject(error);
        },
        timer,
      });
      this.sendRaw(frame);
    });
  }

  /** Sends a notification through the bridge. */
  notify(method: string, params?: unknown): void {
    const frame: Record<string, unknown> = { method };
    if (params !== undefined) {
      frame["params"] = params;
    }
    this.sendRaw(frame);
  }

  /** Responds to a server-initiated request with a result. */
  respondServerRequest(id: number | string, result: unknown): void {
    this.sendRaw({ id, result });
  }

  /** Responds to a server-initiated request with a named error. */
  respondServerRequestError(id: number | string, code: number, message: string): void {
    this.sendRaw({ id, error: { code, message } });
  }

  /** Subscribes to bridged notifications (bounded: slow listeners drop
   * the event locally and increment a named counter). */
  onNotification(listener: (event: BridgeEvent) => void): () => void {
    this.notificationListeners.add(listener);
    return () => {
      this.notificationListeners.delete(listener);
    };
  }

  /** Subscribes to gateway control messages (state, shutdown, errors).
   * Messages that arrived before the first subscription are replayed
   * to it (bounded). */
  onGatewayMessage(listener: (message: GatewayControlMessage) => void): () => void {
    this.gatewayListeners.add(listener);
    if (this.gatewayListeners.size === 1) {
      for (const buffered of this.gatewayMessageBuffer.splice(0)) {
        listener(buffered);
      }
    }
    return () => {
      this.gatewayListeners.delete(listener);
    };
  }

  /** Subscribes to server-initiated requests needing a user decision. */
  onServerRequest(listener: (request: PendingServerRequest) => void): () => void {
    this.serverRequestListeners.add(listener);
    for (const queued of this.serverRequests.splice(0)) {
      listener(queued);
    }
    return () => {
      this.serverRequestListeners.delete(listener);
    };
  }

  /** Subscribes to socket close events (named code + reason). */
  onClose(listener: (code: number, reason: string) => void): () => void {
    this.closeListeners.add(listener);
    return () => {
      this.closeListeners.delete(listener);
    };
  }

  /** True when notifications were dropped locally (honest backpressure). */
  getNotificationDropCount(): number {
    return this.droppedNotifications;
  }

  /** Closes the session (the gateway reaps the supervised app-server). */
  close(): void {
    if (!this.closed) {
      this.closed = true;
      try {
        this.socket.close(1000, "client closed the session");
      } catch {
        // The socket was already closing.
      }
    }
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(new Error("the gateway session closed before a response arrived"));
    }
    this.pending.clear();
  }

  private sendRaw(frame: unknown): void {
    if (this.socket.readyState !== WebSocket.OPEN) {
      return;
    }
    this.socket.send(JSON.stringify(frame));
  }

  private handleMessage(data: unknown): void {
    if (typeof data !== "string") {
      return;
    }
    let value: unknown;
    try {
      value = JSON.parse(data);
    } catch {
      return;
    }
    const control = parseGatewayControlMessage(value);
    if (control !== null) {
      if (control.type === "session.claimed") {
        this.claimedSessionId = control.sessionId;
      }
      if (this.gatewayListeners.size === 0 && this.gatewayMessageBuffer.length < 64) {
        this.gatewayMessageBuffer.push(control);
      }
      for (const listener of this.gatewayListeners) {
        listener(control);
      }
      return;
    }
    this.dispatchBridgeFrame(value);
  }

  private dispatchBridgeFrame(value: unknown): void {
    if (value === null || typeof value !== "object" || Array.isArray(value)) {
      return;
    }
    const record = value as Record<string, unknown>;
    if (typeof record["method"] === "string") {
      if (record["id"] !== undefined && (typeof record["id"] === "number" || typeof record["id"] === "string")) {
        const request: PendingServerRequest = {
          id: record["id"],
          method: record["method"],
          params: record["params"] ?? null,
        };
        if (this.serverRequestListeners.size > 0) {
          for (const listener of this.serverRequestListeners) {
            listener(request);
          }
        } else {
          this.serverRequests.push(request);
        }
        this.emitEvent({ kind: "serverRequest", id: request.id, method: request.method, params: request.params });
      } else {
        this.emitEvent({
          kind: "notification",
          method: record["method"],
          params: record["params"] ?? null,
        });
      }
      return;
    }
    if (record["id"] !== undefined && (record["result"] !== undefined || record["error"] !== undefined)) {
      const id = record["id"];
      if (typeof id === "number") {
        const pending = this.pending.get(id);
        if (pending) {
          this.pending.delete(id);
          const error = record["error"];
          if (
            error !== null &&
            typeof error === "object" &&
            typeof (error as Record<string, unknown>)["message"] === "string"
          ) {
            pending.reject(
              new Error(
                `the app-server rejected the request: ${
                  (error as Record<string, unknown>)["message"]
                } (code ${(error as Record<string, unknown>)["code"] ?? "?"})`,
              ),
            );
          } else {
            pending.resolve(record["result"]);
          }
        }
      }
      this.emitEvent({ kind: "response", id: id as number, result: record["result"], error: undefined });
    }
  }

  private emitEvent(event: BridgeEvent): void {
    if (this.notificationListeners.size === 0) {
      if (this.droppedNotifications < MAX_PENDING_NOTIFICATIONS) {
        this.droppedNotifications += 1;
      }
      return;
    }
    for (const listener of this.notificationListeners) {
      listener(event);
    }
  }

  private handleClose(code: number, reason: string): void {
    this.closed = true;
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(new Error(`the gateway connection closed (${code}): ${reason || "no reason given"}`));
    }
    this.pending.clear();
    for (const listener of this.closeListeners) {
      listener(code, reason);
    }
  }
}

function openSocket(url: string, interrupt?: AbortSignal): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    if (interrupt?.aborted) {
      reject(new Error("the connect attempt was cancelled"));
      return;
    }
    const socket = new WebSocket(url);
    const onAbort = () => {
      try {
        socket.close();
      } catch {
        // Already closing.
      }
      reject(new Error("the connect attempt was cancelled"));
    };
    interrupt?.addEventListener("abort", onAbort, { once: true });
    socket.addEventListener("open", () => {
      interrupt?.removeEventListener("abort", onAbort);
      resolve(socket);
    });
    socket.addEventListener("error", () => {
      interrupt?.removeEventListener("abort", onAbort);
      reject(new Error(`could not open the gateway connection at ${url}`));
    });
  });
}
