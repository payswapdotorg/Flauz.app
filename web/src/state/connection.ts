// The truthful connection-state machine (WEB-001; Wave-6 kernel
// addendum §5).
//
// The shell renders EXACTLY ONE user-meaningful state at all times:
// connected / authenticating / reconnecting / failed-with-reason —
// including the server-side session destruction cases. A silently
// wedged session is the worst failure mode; every state names what
// happened and what happens next.
//
// This module is a PURE state machine (fully unit-tested in
// connection.test.ts); the driver that feeds it from a GatewaySession
// lives in state/app.tsx.

export type ConnectionStatus =
  | "idle"
  | "connecting"
  | "authenticating"
  | "connected"
  | "reconnecting"
  | "failed";

/** The four canonical rendered states (addendum §5). */
export type RenderedConnectionState = "connected" | "authenticating" | "reconnecting" | "failed";

export interface ConnectionState {
  status: ConnectionStatus;
  /** What happened (named, user-readable through the strings layer). */
  reason: string | null;
  /** The raw reason key for the strings layer. */
  reasonKey: ConnectionReasonKey | null;
  /** What happens next (named). */
  attempt: number;
  nextRetryInMs: number | null;
  sessionId: string | null;
  /** The gateway's supervised app-server state (a second truth axis). */
  supervisorState: "unknown" | "connected" | "reconnecting";
  /** True once the account state is known and signed in. */
  signedIn: boolean;
  lastError: string | null;
  /** Auto-retry budget exhausted / handshake refused → user action. */
  needsUserAction: boolean;
  lastChangedAt: number;
}

export type ConnectionReasonKey =
  | "gateway.opening"
  | "gateway.claimed"
  | "gateway.attached"
  | "auth.checking"
  | "auth.signed_in"
  | "auth.signed_out"
  | "gateway.refused"
  | "link.closed"
  | "link.retry_scheduled"
  | "link.retry_started"
  | "link.failed"
  | "supervisor.reconnecting"
  | "supervisor.recovered"
  | "gateway.shutdown"
  | "manual.retry";

export type ConnectionEvent =
  | { type: "connect_started" }
  | { type: "session_claimed"; sessionId: string }
  | { type: "session_denied"; code: string; message: string }
  | { type: "gateway_state"; state: "connected" | "reconnecting"; reason: string }
  | { type: "auth_check_started" }
  | { type: "signed_in" }
  | { type: "signed_out" }
  | { type: "socket_closed"; code: number; reason: string }
  | { type: "retry_scheduled"; attempt: number; delayMs: number; lastError?: string | null }
  | { type: "retry_started"; attempt: number }
  | { type: "retry_budget_exhausted" }
  | { type: "manual_retry" }
  | { type: "gateway_shutdown"; reason: string };

/** The reconnect backoff schedule (browser side; the same rhythm as the
 * gateway/desktop supervisor: 1 s initial, doubling, 20 s cap). */
export const RECONNECT_INITIAL_DELAY_MS = 1_000;
export const RECONNECT_MAX_DELAY_MS = 20_000;
/** After this many failed attempts the state becomes failed-with-reason. */
export const RECONNECT_ATTEMPT_BUDGET = 8;

export function initialConnectionState(): ConnectionState {
  return {
    status: "idle",
    reason: null,
    reasonKey: null,
    attempt: 0,
    nextRetryInMs: null,
    sessionId: null,
    supervisorState: "unknown",
    signedIn: false,
    lastError: null,
    needsUserAction: false,
    lastChangedAt: 0,
  };
}

export function nextBackoffDelay(attempt: number): number {
  const delay = RECONNECT_INITIAL_DELAY_MS * 2 ** Math.max(0, attempt - 1);
  return Math.min(delay, RECONNECT_MAX_DELAY_MS);
}

/** The canonical rendered state (addendum §5's four). */
export function renderedState(state: ConnectionState): RenderedConnectionState {
  switch (state.status) {
    case "connected":
      return "connected";
    case "authenticating":
      return "authenticating";
    case "failed":
      return "failed";
    case "idle":
    case "connecting":
    case "reconnecting":
      return "reconnecting";
  }
}

export function connectionReducer(
  state: ConnectionState,
  event: ConnectionEvent,
): ConnectionState {
  const stamp = { lastChangedAt: nowMs() };
  switch (event.type) {
    case "connect_started":
      return {
        ...state,
        ...stamp,
        status: "connecting",
        reason: "opening the gateway session",
        reasonKey: "gateway.opening",
        attempt: 0,
        nextRetryInMs: null,
        needsUserAction: false,
        lastError: null,
      };
    case "session_claimed":
      if (state.status !== "connecting") {
        return state;
      }
      return {
        ...state,
        ...stamp,
        status: "authenticating",
        reason: "preparing your workspace session",
        reasonKey: "gateway.claimed",
        sessionId: event.sessionId,
      };
    case "session_denied":
      return {
        ...state,
        ...stamp,
        status: "failed",
        reason: `the gateway refused the session (${event.code}): ${event.message}`,
        reasonKey: "gateway.refused",
        lastError: `session refused (${event.code})`,
        needsUserAction: true,
        nextRetryInMs: null,
      };
    case "gateway_state":
      if (event.state === "connected") {
        // Recovery language when a previously live session is back
        // (link drop or supervised app-server restart), plain
        // attach language on the first connect.
        const recovered = state.signedIn;
        const next: ConnectionState = {
          ...state,
          ...stamp,
          supervisorState: "connected",
        };
        if (state.status === "authenticating" || state.status === "reconnecting" || state.status === "connected") {
          next.status = state.signedIn ? "connected" : "authenticating";
          next.reasonKey = state.signedIn
            ? recovered
              ? "supervisor.recovered"
              : "auth.checking"
            : "auth.checking";
          next.reason = state.signedIn
            ? recovered
              ? "the agent runtime recovered; your workspace is live again"
              : "the agent runtime is attached; your workspace is live"
            : "checking your sign-in";
          next.lastError = null;
          next.needsUserAction = false;
          next.nextRetryInMs = null;
        }
        return next;
      }
      // The supervised app-server died; the supervisor is restarting it.
      return {
        ...state,
        ...stamp,
        status: "reconnecting",
        supervisorState: "reconnecting",
        reason: `the agent runtime disconnected (${event.reason}); it is being restarted`,
        reasonKey: "supervisor.reconnecting",
        attempt: 0,
        nextRetryInMs: null,
      };
    case "auth_check_started":
      if (state.status !== "authenticating" && state.status !== "connected") {
        return state;
      }
      return {
        ...state,
        ...stamp,
        status: state.signedIn ? "connected" : "authenticating",
        reason: "checking your sign-in",
        reasonKey: "auth.checking",
      };
    case "signed_in":
      return {
        ...state,
        ...stamp,
        status: state.supervisorState === "connected" ? "connected" : "reconnecting",
        signedIn: true,
        reason: "signed in; your workspace is live",
        reasonKey: "auth.signed_in",
        lastError: null,
        needsUserAction: false,
      };
    case "signed_out":
      return {
        ...state,
        ...stamp,
        signedIn: false,
        status: state.supervisorState === "connected" ? "authenticating" : "reconnecting",
        reason: "sign in to open your workspace",
        reasonKey: "auth.signed_out",
      };
    case "socket_closed":
      if (state.status === "failed") {
        return state;
      }
      return {
        ...state,
        ...stamp,
        status: "reconnecting",
        reason:
          event.reason === ""
            ? `the gateway connection closed (code ${event.code})`
            : `the gateway connection closed (code ${event.code}): ${event.reason}`,
        reasonKey: "link.closed",
        lastError: `gateway connection closed (${event.code})`,
        attempt: 0,
        nextRetryInMs: null,
        supervisorState: "unknown",
      };
    case "retry_scheduled":
      return {
        ...state,
        ...stamp,
        status: "reconnecting",
        reason: `reconnecting (attempt ${event.attempt} of ${RECONNECT_ATTEMPT_BUDGET})`,
        reasonKey: "link.retry_scheduled",
        attempt: event.attempt,
        nextRetryInMs: event.delayMs,
        lastError: event.lastError ?? state.lastError,
      };
    case "retry_started":
      return {
        ...state,
        ...stamp,
        status: "connecting",
        reason: `reconnecting (attempt ${event.attempt})`,
        reasonKey: "link.retry_started",
        attempt: event.attempt,
        nextRetryInMs: null,
      };
    case "retry_budget_exhausted":
      return {
        ...state,
        ...stamp,
        status: "failed",
        reason:
          state.lastError === null
            ? "the gateway connection could not be re-established"
            : `the gateway connection could not be re-established (${state.lastError})`,
        reasonKey: "link.failed",
        needsUserAction: true,
        nextRetryInMs: null,
      };
    case "manual_retry":
      return {
        ...state,
        ...stamp,
        status: "connecting",
        reason: "reconnecting now",
        reasonKey: "manual.retry",
        attempt: 0,
        nextRetryInMs: null,
        needsUserAction: false,
        lastError: null,
      };
    case "gateway_shutdown":
      return {
        ...state,
        ...stamp,
        status: "reconnecting",
        reason: `the gateway is shutting down (${event.reason}); it will be back shortly`,
        reasonKey: "gateway.shutdown",
        attempt: 0,
        nextRetryInMs: null,
      };
    default:
      return state;
  }
}

function nowMs(): number {
  return typeof performance === "undefined" ? Date.now() : Math.round(performance.now());
}
