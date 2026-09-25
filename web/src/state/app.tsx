// The app state driver: wires the pure connection-state machine to a
// GatewaySession, drives the reconnect backoff, and holds the workspace
// data (account, sessions, current session timeline, approvals).

import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  type ReactNode,
} from "react";
import { GatewaySession, type BridgeEvent, type PendingServerRequest } from "../protocol/client";
import type { Account, ThreadSummary } from "../protocol/generated";
import {
  RECONNECT_ATTEMPT_BUDGET,
  connectionReducer,
  initialConnectionState,
  nextBackoffDelay,
  type ConnectionState,
} from "./connection";
import { createTranslator, type Translator } from "../strings/en";

export interface TimelineEntry {
  id: string;
  kind: "turn_started" | "turn_completed" | "message" | "command" | "notice";
  text: string;
  at: number;
}

export interface ApprovalCard {
  request: PendingServerRequest;
  resolved: "approved" | "approved_for_session" | "declined" | "declined_named" | null;
}

export type SessionsLoadState = "unloaded" | "loading" | "ready" | "error";

export interface AppState {
  connection: ConnectionState;
  account: Account | null;
  requiresOpenaiAuth: boolean;
  sessions: ThreadSummary[];
  sessionsState: SessionsLoadState;
  sessionsError: string | null;
  currentSessionId: string | null;
  currentSession: ThreadSummary | null;
  currentSessionState: "unloaded" | "loading" | "ready" | "error";
  currentSessionError: string | null;
  timeline: TimelineEntry[];
  approvals: ApprovalCard[];
  liveMessage: string | null;
}

type AppAction =
  | { type: "connection"; event: Parameters<typeof connectionReducer>[1] }
  | { type: "account_read"; account: Account | null; requiresOpenaiAuth: boolean }
  | { type: "sessions_load_started" }
  | { type: "sessions_loaded"; sessions: ThreadSummary[] }
  | { type: "sessions_error"; error: string }
  | { type: "session_opened"; sessionId: string }
  | { type: "session_closed" }
  | { type: "session_loaded"; thread: ThreadSummary }
  | { type: "session_error"; error: string }
  | { type: "timeline_entry"; entry: TimelineEntry }
  | { type: "approval_new"; request: PendingServerRequest }
  | { type: "approval_resolved"; id: number | string; outcome: ApprovalCard["resolved"] }
  | { type: "live_message"; message: string | null };

const MAX_TIMELINE_ENTRIES = 200;
const MAX_APPROVALS = 16;
const SESSIONS_PAGE_LIMIT = 50;

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "connection":
      return { ...state, connection: connectionReducer(state.connection, action.event) };
    case "account_read":
      return {
        ...state,
        account: action.account,
        requiresOpenaiAuth: action.requiresOpenaiAuth,
      };
    case "sessions_load_started":
      return { ...state, sessionsState: "loading", sessionsError: null };
    case "sessions_loaded":
      return { ...state, sessionsState: "ready", sessions: action.sessions, sessionsError: null };
    case "sessions_error":
      return { ...state, sessionsState: "error", sessionsError: action.error };
    case "session_opened":
      return {
        ...state,
        currentSessionId: action.sessionId,
        currentSession: null,
        currentSessionState: "loading",
        currentSessionError: null,
        timeline: [],
        liveMessage: null,
      };
    case "session_closed":
      return {
        ...state,
        currentSessionId: null,
        currentSession: null,
        currentSessionState: "unloaded",
        currentSessionError: null,
        timeline: [],
        liveMessage: null,
      };
    case "session_loaded":
      return {
        ...state,
        currentSession: action.thread,
        currentSessionState: "ready",
        currentSessionError: null,
      };
    case "session_error":
      return { ...state, currentSessionState: "error", currentSessionError: action.error };
    case "timeline_entry":
      return {
        ...state,
        timeline:
          state.timeline.length >= MAX_TIMELINE_ENTRIES
            ? [...state.timeline.slice(1), action.entry]
            : [...state.timeline, action.entry],
      };
    case "approval_new":
      if (state.approvals.length >= MAX_APPROVALS) {
        return state;
      }
      return {
        ...state,
        approvals: [
          ...state.approvals,
          { request: action.request, resolved: null },
        ],
      };
    case "approval_resolved":
      return {
        ...state,
        approvals: state.approvals.map((card) =>
          card.request.id === action.id ? { ...card, resolved: action.outcome } : card,
        ),
      };
    case "live_message":
      return { ...state, liveMessage: action.message };
    default:
      return state;
  }
}

function initialAppState(): AppState {
  return {
    connection: initialConnectionState(),
    account: null,
    requiresOpenaiAuth: false,
    sessions: [],
    sessionsState: "unloaded",
    sessionsError: null,
    currentSessionId: null,
    currentSession: null,
    currentSessionState: "unloaded",
    currentSessionError: null,
    timeline: [],
    approvals: [],
    liveMessage: null,
  };
}

export interface FlauzApp {
  state: AppState;
  t: Translator;
  session: GatewaySession | null;
  connect: () => void;
  manualRetry: () => void;
  openSession: (threadId: string) => void;
  closeSession: () => void;
  refreshSessions: () => void;
  startTask: (objective: string) => Promise<void>;
  sendToSession: (message: string) => Promise<void>;
  decideApproval: (
    id: number | string,
    method: string,
    decision: "approve" | "approve_for_session" | "decline",
  ) => void;
  signInWithChatGpt: () => Promise<void>;
  signInWithApiKey: (apiKey: string) => Promise<void>;
  signOut: () => Promise<void>;
}

const AppContext = createContext<FlauzApp | null>(null);

export function useFlauzApp(): FlauzApp {
  const value = useContext(AppContext);
  if (value === null) {
    throw new Error("useFlauzApp requires the FlauzAppProvider");
  }
  return value;
}

export function gatewayUrl(): string {
  const configured = typeof import.meta !== "undefined" ? (import.meta as { env?: Record<string, string> }).env?.VITE_GATEWAY_URL : undefined;
  if (configured && configured !== "") {
    return configured;
  }
  if (typeof window === "undefined") {
    return "ws://127.0.0.1:8610/ws";
  }
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${protocol}//${window.location.host}/ws`;
}

export function FlauzAppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, undefined, initialAppState);
  const sessionRef = useRef<GatewaySession | null>(null);
  const retryTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const attemptRef = useRef(0);
  const t = useMemo(() => createTranslator("en"), []);

  const clearRetryTimer = () => {
    if (retryTimer.current !== null) {
      clearTimeout(retryTimer.current);
      retryTimer.current = null;
    }
  };

  const connectOnce = async () => {
    dispatch({ type: "connection", event: { type: attemptRef.current === 0 ? "connect_started" : "retry_started", attempt: attemptRef.current } });
    let session: GatewaySession | null = null;
    try {
      session = await GatewaySession.connect(gatewayUrl());
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      scheduleRetry(message);
      return;
    }
    sessionRef.current = session;
    attemptRef.current = 0;

    session.onGatewayMessage((message) => {
      if (message.type === "session.claimed") {
        dispatch({ type: "connection", event: { type: "session_claimed", sessionId: message.sessionId } });
      } else if (message.type === "gateway.state") {
        dispatch({ type: "connection", event: { type: "gateway_state", state: message.state, reason: message.reason } });
        if (message.state === "connected") {
          void resyncAfterRecovery();
        }
      } else if (message.type === "session.denied") {
        dispatch({ type: "connection", event: { type: "session_denied", code: message.code, message: message.message } });
      } else if (message.type === "gateway.shutdown") {
        dispatch({ type: "connection", event: { type: "gateway_shutdown", reason: message.reason } });
      } else if (message.type === "gateway.error") {
        dispatch({ type: "live_message", message: message.error.message });
      }
    });

    session.onClose((code, reason) => {
      sessionRef.current = null;
      dispatch({ type: "connection", event: { type: "socket_closed", code, reason } });
      scheduleRetry(reason || `gateway closed (${code})`);
    });

    session.onServerRequest((request) => {
      dispatch({ type: "approval_new", request });
    });

    session.onNotification((event) => {
      handleBridgeEvent(event);
    });
  };

  const scheduleRetry = (lastError: string) => {
    clearRetryTimer();
    attemptRef.current += 1;
    if (attemptRef.current > RECONNECT_ATTEMPT_BUDGET) {
      attemptRef.current = RECONNECT_ATTEMPT_BUDGET;
      dispatch({ type: "connection", event: { type: "retry_budget_exhausted" } });
      return;
    }
    const delay = nextBackoffDelay(attemptRef.current);
    dispatch({
      type: "connection",
      event: { type: "retry_scheduled", attempt: attemptRef.current, delayMs: delay, lastError },
    });
    retryTimer.current = setTimeout(() => {
      retryTimer.current = null;
      void connectOnce();
    }, delay);
  };

  const resyncAfterRecovery = async () => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    dispatch({ type: "connection", event: { type: "auth_check_started" } });
    try {
      const account = await session.request("account/read", { refreshToken: false });
      dispatch({
        type: "account_read",
        account: account.account ?? null,
        requiresOpenaiAuth: account.requiresOpenaiAuth,
      });
      if (account.account !== null) {
        dispatch({ type: "connection", event: { type: "signed_in" } });
      } else {
        dispatch({ type: "connection", event: { type: "signed_out" } });
      }
    } catch (error) {
      dispatch({ type: "connection", event: { type: "signed_out" } });
      dispatch({
        type: "live_message",
        message: `could not check the sign-in state: ${error instanceof Error ? error.message : String(error)}`,
      });
    }
    void refreshSessions();
  };

  const refreshSessions = async () => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    dispatch({ type: "sessions_load_started" });
    try {
      const result = await session.request("thread/list", {
        limit: SESSIONS_PAGE_LIMIT,
        sortKey: "recency_at",
        sortDirection: "desc",
        useStateDbOnly: true,
      });
      dispatch({ type: "sessions_loaded", sessions: result.data });
    } catch (error) {
      dispatch({
        type: "sessions_error",
        error: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const handleBridgeEvent = (event: BridgeEvent) => {
    if (event.kind === "notification") {
      if (event.method === "account/login/completed") {
        // The app-server finished the external authorization; re-check
        // the account state.
        void resyncAfterRecovery();
      } else if (event.method === "turn/started") {
        const turn = (event.params as { turn?: { id?: string } }).turn;
        dispatch({
          type: "timeline_entry",
          entry: {
            id: `started-${turn?.id ?? Math.random().toString(36).slice(2)}`,
            kind: "turn_started",
            text: t("session.turn.running"),
            at: Date.now(),
          },
        });
      } else if (event.method === "turn/completed") {
        const params = event.params as { turn?: { id?: string; status?: string } };
        dispatch({
          type: "timeline_entry",
          entry: {
            id: `completed-${params.turn?.id ?? Math.random().toString(36).slice(2)}`,
            kind: params.turn?.status === "completed" ? "turn_completed" : "notice",
            text:
              params.turn?.status === "completed"
                ? t("session.turn.completed")
                : t("session.turn.failed", { reason: params.turn?.status ?? "unknown" }),
            at: Date.now(),
          },
        });
      } else if (event.method === "item/agentMessage/delta") {
        const delta = (event.params as { delta?: string }).delta ?? "";
        dispatch({ type: "live_message", message: delta });
      } else if (event.method === "item/completed") {
        const item = (event.params as { item?: { type?: string; text?: string; command?: string } }).item;
        if (item?.type === "agentMessage" && typeof item.text === "string") {
          dispatch({
            type: "timeline_entry",
            entry: {
              id: `message-${Math.random().toString(36).slice(2)}`,
              kind: "message",
              text: item.text,
              at: Date.now(),
            },
          });
          dispatch({ type: "live_message", message: null });
        }
      } else if (event.method === "error") {
        const message = (event.params as { error?: { message?: string } }).error?.message;
        if (message) {
          dispatch({ type: "live_message", message: `agent runtime: ${message}` });
        }
      }
    }
  };

  useEffect(() => {
    void connectOnce();
    return () => {
      clearRetryTimer();
      sessionRef.current?.close();
      sessionRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadSession = (threadId: string) => {
    const session = sessionRef.current;
    if (session === null) {
      dispatch({ type: "session_error", error: t("session.connecting") });
      return;
    }
    void session
      .request("thread/resume", {
        threadId,
        excludeTurns: false,
        initialTurnsPage: { limit: 20, sortDirection: "asc", itemsView: "summary" },
      })
      .then((result) => {
        dispatch({ type: "session_loaded", thread: result.thread });
      })
      .catch((error: unknown) => {
        dispatch({
          type: "session_error",
          error: error instanceof Error ? error.message : String(error),
        });
      });
  };

  const openSession = (threadId: string) => {
    window.location.hash = `#/session/${encodeURIComponent(threadId)}`;
    if (state.currentSessionId === threadId && state.currentSessionState !== "unloaded") {
      return;
    }
    dispatch({ type: "session_opened", sessionId: threadId });
    loadSession(threadId);
  };

  const startTask = async (objective: string) => {
    const session = sessionRef.current;
    if (session === null) {
      throw new Error("the gateway connection is not open");
    }
    const threadId = `web-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
    // Reset the session surface BEFORE the turn so streamed events land
    // in a clean timeline, then start the task, navigate, and load.
    dispatch({ type: "session_opened", sessionId: threadId });
    await session.request("turn/start", {
      threadId,
      input: [{ type: "text", text: objective }],
    });
    window.location.hash = `#/session/${encodeURIComponent(threadId)}`;
    loadSession(threadId);
    void refreshSessions();
  };

  const sendToSession = async (message: string) => {
    const session = sessionRef.current;
    const threadId = state.currentSessionId;
    if (session === null || threadId === null) {
      return;
    }
    await session.request("turn/start", {
      threadId,
      input: [{ type: "text", text: message }],
    });
    dispatch({
      type: "timeline_entry",
      entry: { id: `user-${Date.now()}`, kind: "notice", text: message, at: Date.now() },
    });
  };

  const decideApproval = (
    id: number | string,
    method: string,
    decision: "approve" | "approve_for_session" | "decline",
  ) => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    if (method === "item/permissions/requestApproval") {
      session.respondServerRequestError(
        id,
        -32000,
        "declined by the web client: permission profile selection arrives with the capability surfaces",
      );
      dispatch({ type: "approval_resolved", id, outcome: "declined_named" });
      return;
    }
    const decisionValue =
      decision === "approve" ? "accept" : decision === "approve_for_session" ? "acceptForSession" : "decline";
    session.respondServerRequest(id, { decision: decisionValue });
    dispatch({
      type: "approval_resolved",
      id,
      outcome: decision === "decline" ? "declined" : decision === "approve" ? "approved" : "approved_for_session",
    });
  };

  const signInWithChatGpt = async () => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    const login = await session.request("account/login/start", { type: "chatgpt" });
    if (login.type !== "chatgpt") {
      throw new Error(`unexpected login response: ${login.type}`);
    }
    window.open(login.authUrl, "_blank", "noopener,noreferrer");
  };

  const signInWithApiKey = async (apiKey: string) => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    await session.request("account/login/start", { type: "apiKey", apiKey });
    void resyncAfterRecovery();
  };

  const signOut = async () => {
    const session = sessionRef.current;
    if (session === null) {
      return;
    }
    await session.request("account/logout");
    void resyncAfterRecovery();
  };

  const app: FlauzApp = {
    state,
    t,
    session: sessionRef.current,
    connect: () => {
      clearRetryTimer();
      attemptRef.current = 0;
      void connectOnce();
    },
    manualRetry: () => {
      clearRetryTimer();
      attemptRef.current = 0;
      dispatch({ type: "connection", event: { type: "manual_retry" } });
      void connectOnce();
    },
    openSession,
    closeSession: () => {
      dispatch({ type: "session_closed" });
    },
    refreshSessions: () => {
      void refreshSessions();
    },
    startTask,
    sendToSession,
    decideApproval,
    signInWithChatGpt,
    signInWithApiKey,
    signOut,
  };

  return <AppContext.Provider value={app}>{children}</AppContext.Provider>;
}
