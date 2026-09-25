// The connection-state machine tests (WEB-001): every state names what
// happened, exactly one canonical state is rendered at a time, the
// kill → reconnecting → failed truth table holds, and recovery works.

import { describe, expect, it } from "vitest";
import {
  RECONNECT_ATTEMPT_BUDGET,
  RECONNECT_INITIAL_DELAY_MS,
  RECONNECT_MAX_DELAY_MS,
  connectionReducer,
  initialConnectionState,
  nextBackoffDelay,
  renderedState,
} from "./connection";

function boot(): ReturnType<typeof initialConnectionState> {
  let state = initialConnectionState();
  state = connectionReducer(state, { type: "connect_started" });
  state = connectionReducer(state, { type: "session_claimed", sessionId: "gwsess_test" });
  state = connectionReducer(state, {
    type: "gateway_state",
    state: "connected",
    reason: "attached",
  });
  return state;
}

function signIn(state: ReturnType<typeof initialConnectionState>) {
  return connectionReducer(state, { type: "signed_in" });
}

describe("the truthful connection-state machine", () => {
  it("starts idle and reaches connected through the named path", () => {
    const state = signIn(boot());
    expect(state.status).toBe("connected");
    expect(state.signedIn).toBe(true);
    expect(state.sessionId).toBe("gwsess_test");
    expect(state.reason).not.toBeNull();
    expect(renderedState(state)).toBe("connected");
  });

  it("renders exactly one of the four canonical states at every step", () => {
    let state = initialConnectionState();
    const seen = new Set<string>();
    seen.add(renderedState(state));
    state = connectionReducer(state, { type: "connect_started" });
    seen.add(renderedState(state));
    state = connectionReducer(state, { type: "session_claimed", sessionId: "gwsess_test" });
    seen.add(renderedState(state));
    state = connectionReducer(state, { type: "gateway_state", state: "connected", reason: "attached" });
    seen.add(renderedState(state));
    state = signIn(state);
    seen.add(renderedState(state));
    expect([...seen].every((state) => ["connected", "authenticating", "reconnecting", "failed"].includes(state))).toBe(
      true,
    );
  });

  it("an unsigned account lands in authenticating with the sign-in reason", () => {
    const state = connectionReducer(boot(), { type: "signed_out" });
    expect(state.status).toBe("authenticating");
    expect(renderedState(state)).toBe("authenticating");
    expect(state.reason).toContain("sign in");
  });

  it("a JSON-RPC-before-handshake refusal becomes failed with the named code", () => {
    let state = initialConnectionState();
    state = connectionReducer(state, { type: "connect_started" });
    state = connectionReducer(state, {
      type: "session_denied",
      code: "handshake_required",
      message: "claim a session before sending app-server frames",
    });
    expect(state.status).toBe("failed");
    expect(renderedState(state)).toBe("failed");
    expect(state.reason).toContain("handshake_required");
    expect(state.needsUserAction).toBe(true);
  });

  it("killing the gateway shows a NAMED reconnecting state and recovers", () => {
    let state = signIn(boot());
    state = connectionReducer(state, { type: "socket_closed", code: 1006, reason: "" });
    expect(state.status).toBe("reconnecting");
    expect(renderedState(state)).toBe("reconnecting");
    expect(state.reason).toContain("1006");
    expect(state.signedIn).toBe(true);
    // A retry starts and the gateway comes back.
    state = connectionReducer(state, { type: "retry_started", attempt: 1 });
    expect(state.status).toBe("connecting");
    state = connectionReducer(state, { type: "session_claimed", sessionId: "gwsess_test2" });
    state = connectionReducer(state, {
      type: "gateway_state",
      state: "connected",
      reason: "the supervised app-server restarted (restart 2)",
    });
    expect(state.status).toBe("connected");
    expect(state.supervisorState).toBe("connected");
    expect(state.reason).toContain("recovered");
  });

  it("the server-side app-server death surfaces as reconnecting (never silent)", () => {
    let state = signIn(boot());
    state = connectionReducer(state, {
      type: "gateway_state",
      state: "reconnecting",
      reason: "the app-server closed its transport; restarting (attempt 1, in 1000 ms)",
    });
    expect(state.status).toBe("reconnecting");
    expect(state.supervisorState).toBe("reconnecting");
    expect(state.reason).toContain("app-server");
    expect(state.signedIn).toBe(true);
  });

  it("graceful gateway shutdown is a named reconnecting state", () => {
    const state = connectionReducer(signIn(boot()), {
      type: "gateway_shutdown",
      reason: "the gateway is shutting down",
    });
    expect(state.status).toBe("reconnecting");
    expect(state.reason).toContain("shutting down");
  });

  it("exhausted retries become failed-with-reason needing user action", () => {
    let state = signIn(boot());
    state = connectionReducer(state, { type: "socket_closed", code: 1006, reason: "gateway down" });
    for (let attempt = 1; attempt <= RECONNECT_ATTEMPT_BUDGET; attempt += 1) {
      state = connectionReducer(state, {
        type: "retry_scheduled",
        attempt,
        delayMs: nextBackoffDelay(attempt),
      });
      expect(state.status).toBe("reconnecting");
      state = connectionReducer(state, { type: "retry_started", attempt });
      state = connectionReducer(state, { type: "socket_closed", code: 1006, reason: "gateway down" });
    }
    state = connectionReducer(state, { type: "retry_budget_exhausted" });
    expect(state.status).toBe("failed");
    expect(renderedState(state)).toBe("failed");
    expect(state.reason).toContain("could not be re-established");
    expect(state.reason).toContain("1006");
    expect(state.needsUserAction).toBe(true);
    // Manual retry leaves the failed state.
    state = connectionReducer(state, { type: "manual_retry" });
    expect(state.status).toBe("connecting");
    expect(state.needsUserAction).toBe(false);
  });

  it("the backoff schedule doubles and caps at 20 s", () => {
    expect(nextBackoffDelay(1)).toBe(RECONNECT_INITIAL_DELAY_MS);
    expect(nextBackoffDelay(2)).toBe(2_000);
    expect(nextBackoffDelay(3)).toBe(4_000);
    expect(nextBackoffDelay(5)).toBe(16_000);
    expect(nextBackoffDelay(6)).toBe(RECONNECT_MAX_DELAY_MS);
    expect(nextBackoffDelay(9)).toBe(RECONNECT_MAX_DELAY_MS);
  });

  it("a session denial while signed in still fails truthfully", () => {
    let state = signIn(boot());
    state = connectionReducer(state, {
      type: "session_denied",
      code: "invalid_token",
      message: "a valid provisioned session token is required",
    });
    expect(state.status).toBe("failed");
    expect(state.reason).toContain("invalid_token");
  });
});
