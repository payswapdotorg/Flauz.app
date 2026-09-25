// The gateway WebSocket control-envelope contract (v1) — the TypeScript
// mirror of `crates/flauz-web-gateway/src/protocol.rs`
// (also served by the gateway at GET /gateway-protocol.json).
//
// NOTE: these are the GATEWAY transport types, NOT app-server protocol
// types. The app-server protocol types live exclusively in
// `src/protocol/generated.ts` (generated from the schema export); the
// gateway envelope is defined by the gateway crate, which this module
// mirrors 1:1.

export const GATEWAY_PROTOCOL_VERSION = 1 as const;

export type GatewayControlMessage =
  | SessionClaim
  | SessionClaimed
  | SessionDenied
  | GatewayState
  | GatewayShutdown
  | GatewayError;

export interface SessionClaim {
  type: "session.claim";
  token?: string;
}

export interface SessionClaimed {
  type: "session.claimed";
  sessionId: string;
  protocolVersion: number;
}

export type SessionDenialCode =
  | "handshake_required"
  | "handshake_already_claimed"
  | "invalid_token"
  | "session_limit_reached"
  | "message_too_large"
  | "malformed_message";

export interface SessionDenied {
  type: "session.denied";
  code: SessionDenialCode;
  message: string;
}

export type GatewaySupervisorState = "connected" | "reconnecting";

export interface GatewayState {
  type: "gateway.state";
  state: GatewaySupervisorState;
  reason: string;
}

export interface GatewayShutdown {
  type: "gateway.shutdown";
  reason: string;
}

export interface GatewayError {
  type: "gateway.error";
  error: { code: number; message: string };
}

/** Discriminated-union guard for gateway control frames. */
export function parseGatewayControlMessage(
  value: unknown,
): GatewayControlMessage | null {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  const record = value as Record<string, unknown>;
  switch (record["type"]) {
    case "session.claim":
      if (record["token"] === undefined || typeof record["token"] === "string") {
        return {
          type: "session.claim",
          ...(typeof record["token"] === "string" ? { token: record["token"] } : {}),
        };
      }
      return null;
    case "session.claimed":
      if (typeof record["sessionId"] === "string") {
        return { type: "session.claimed", sessionId: record["sessionId"], protocolVersion: 1 };
      }
      return null;
    case "session.denied":
      if (typeof record["code"] === "string" && typeof record["message"] === "string") {
        return { type: "session.denied", code: record["code"] as SessionDenialCode, message: record["message"] };
      }
      return null;
    case "gateway.state":
      if (
        (record["state"] === "connected" || record["state"] === "reconnecting") &&
        typeof record["reason"] === "string"
      ) {
        return { type: "gateway.state", state: record["state"], reason: record["reason"] };
      }
      return null;
    case "gateway.shutdown":
      if (typeof record["reason"] === "string") {
        return { type: "gateway.shutdown", reason: record["reason"] };
      }
      return null;
    case "gateway.error": {
      const error = record["error"];
      if (
        error !== null &&
        typeof error === "object" &&
        typeof (error as Record<string, unknown>)["code"] === "number" &&
        typeof (error as Record<string, unknown>)["message"] === "string"
      ) {
        return {
          type: "gateway.error",
          error: {
            code: (error as Record<string, unknown>)["code"] as number,
            message: (error as Record<string, unknown>)["message"] as string,
          },
        };
      }
      return null;
    }
    default:
      return null;
  }
}
