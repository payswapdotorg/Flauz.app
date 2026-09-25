// GENERATED FILE — DO NOT EDIT.
// Provenance: src/protocol/schema.json (the captured app-server schema export).
// Regenerate with: npm run protocol:export && npm run protocol:generate
// Hand-written protocol types in web/ are FORBIDDEN (Wave-6 kernel addendum §3).

export const APP_SERVER_SCHEMA_VERSION = 1;
export const APP_SERVER_SCHEMA_SOURCE = "CODEX_APP_SERVER_SCHEMA_EXPERIMENTAL=1 export (initial snapshot assembled from the frozen codex-protocol client surface at Flauz base 0bbccc24 because the worker sandbox lacked the codex binary; regenerate from the real export with: CODEX_RS_CODEX_BIN=<binary> npm run protocol:export && npm run protocol:generate)";

export type RequestFrame = {
  method: string;
  id: number;
  /** Method-specific params; absent when the method takes none. */
  params?: unknown;
};
export type NotificationFrame = {
  method: string;
  /** Method-specific params; may be absent. */
  params?: unknown;
};
export type ResponseFrame = {
  /** The id of the request this frame answers. */
  id: number | string;
  /** Present on success. */
  result?: unknown;
  error?: RpcError;
};

export type RpcError = {
  code: number;
  message: string;
};

export type Account = {
  type: "apiKey";
} | {
  type: "chatgpt";
  email?: string | null;
  planType: string;
} | {
  type: "amazonBedrock";
  usesCodexManagedCredentials: boolean;
};

export type ThreadSummary = {
  id: string;
  sessionId: string;
  forkedFromId?: string | null;
  parentThreadId?: string | null;
  preview: string;
  name?: string | null;
  cwd: string;
  createdAt: number;
  updatedAt: number;
  recencyAt?: number | null;
  /** Thread status payload (surface-shape varies by app-server version). */
  status?: unknown;
  /** Git state payload when present. */
  gitInfo?: unknown;
  turns?: unknown[];
};

export type TurnsPage = {
  data: unknown[];
  nextCursor?: string | null;
  backwardsCursor?: string | null;
};

export type Turn = {
  id: string;
  status: string;
  /** Structured error payload when the turn failed. */
  error?: unknown;
};

export type UserInput = {
  type: "text";
  text: string;
  textElements?: unknown[];
} | {
  type: "localImage";
  path: string;
  detail?: string;
} | {
  type: "mention";
  name: string;
  path: string;
} | {
  type: "skill";
  name: string;
  path: string;
};

export type ApprovalDecision = "accept" | "acceptForSession" | "decline" | "cancel";

export type AccountReadParams = {
  /** Refresh the account token server-side. */
  refreshToken?: boolean;
};

export type AccountReadResult = {
  account?: Account | null;
  requiresOpenaiAuth: boolean;
};

export type AccountLoginStartParams = {
  type: "apiKey";
  /** The API key (never logged, never stored by the web client). */
  apiKey: string;
} | {
  type: "chatgpt";
  codexStreamlinedLogin?: boolean;
  useHostedLoginSuccessPage?: boolean;
  appBrand?: "codex" | "chatgpt";
} | {
  type: "chatgptDeviceCode";
} | {
  type: "amazonBedrock";
  apiKey: string;
  region: string;
};

export type AccountLoginStartResult = {
  type: "apiKey";
} | {
  type: "chatgpt";
  loginId: string;
  /** The external authorization URL to open. */
  authUrl: string;
} | {
  type: "chatgptDeviceCode";
  loginId: string;
  verificationUrl: string;
  userCode: string;
} | {
  type: "chatgptAuthTokens";
} | {
  type: "amazonBedrock";
};

export type AccountLoginCancelParams = {
  loginId: string;
};

export type AccountLoginCancelResult = Record<string, unknown>;

export type AccountLogoutResult = Record<string, unknown>;

export type GetAuthStatusParams = {
  includeToken: boolean;
  refreshToken: boolean;
};

export type GetAuthStatusResult = {
  authMethod?: string | null;
  /** Auth token when requested; the web client never requests it and never logs it. */
  authToken?: string | null;
  accountId?: string | null;
  requiresOpenaiAuth: boolean;
};

export type ThreadListParams = {
  limit: number;
  sortKey: "recency_at";
  sortDirection: "desc";
  /** Must be true (the state-DB-only law). */
  useStateDbOnly: boolean;
  cursor?: string | null;
  archived?: boolean;
  cwd?: string | null;
  searchTerm?: string | null;
};

export type ThreadListResult = {
  data: ThreadSummary[];
  nextCursor?: string | null;
  backwardsCursor?: string | null;
};

export type ThreadSearchParams = {
  cursor?: string | null;
  limit?: number;
  sortKey?: "recency_at";
  sortDirection?: "desc";
  sourceKinds?: string[];
  archived?: boolean;
  searchTerm: string;
};

export type ThreadSearchResult = {
  data: {
  thread: ThreadSummary;
  snippet: string;
}[];
  nextCursor?: string | null;
};

export type ThreadReadParams = {
  threadId: string;
  includeTurns: boolean;
};

export type ThreadReadResult = {
  thread: ThreadSummary;
};

export type ThreadResumeParams = {
  threadId: string;
  excludeTurns?: boolean;
  initialTurnsPage?: {
  cursor?: string | null;
  limit: number;
  sortDirection: "asc" | "desc";
  itemsView?: string | null;
};
};

export type ThreadResumeResult = {
  thread: ThreadSummary;
  initialTurnsPage?: TurnsPage | null;
  model?: string | null;
  reasoningEffort?: string | null;
  /** The thread's service tier when reported. */
  serviceTier?: unknown;
};

export type ThreadTurnsListParams = {
  threadId: string;
  limit: number;
  sortDirection: "asc" | "desc";
  cursor?: string | null;
  itemsView?: string | null;
};

export type ThreadTurnsListResult = TurnsPage;

export type TurnStartParams = {
  threadId: string;
  input: UserInput[];
  clientUserMessageId?: string;
  cwd?: string;
  model?: string;
  effort?: string;
  summary?: string;
};

export type TurnStartResult = {
  threadId?: string;
  turn?: Turn;
};

export type TurnInterruptParams = {
  threadId: string;
};

export type TurnInterruptResult = Record<string, unknown>;

export type TurnStartedParams = {
  threadId?: string;
  turn: Turn;
};

export type TurnCompletedParams = {
  threadId?: string;
  turn: Turn;
};

export type ItemAgentMessageDeltaParams = {
  threadId?: string;
  turnId?: string;
  itemId?: string;
  delta: string;
};

export type ItemCompletedParams = {
  threadId?: string;
  turnId?: string;
  item: {
  type: string;
  text?: string;
  command?: string;
};
};

export type ErrorParams = {
  error?: RpcError;
};

export type ThreadStatusChangedParams = {
  threadId?: string;
  /** The new thread status payload. */
  status?: unknown;
};

export type ItemCommandExecutionRequestApprovalParams = {
  threadId?: string;
  turnId?: string;
  itemId?: string;
  command: string;
  cwd?: string;
};

export type ItemCommandExecutionRequestApprovalResponse = {
  decision: ApprovalDecision;
};

export type ItemFileChangeRequestApprovalParams = {
  threadId?: string;
  turnId?: string;
  itemId?: string;
  changeSummary?: string;
};

export type ItemFileChangeRequestApprovalResponse = {
  decision: ApprovalDecision;
};

export type ItemPermissionsRequestApprovalParams = {
  threadId?: string;
  turnId?: string;
  details?: string[];
};

export const REQUEST_METHODS = [
  "account/read",
  "account/login/start",
  "account/login/cancel",
  "account/logout",
  "getAuthStatus",
  "thread/list",
  "thread/search",
  "thread/read",
  "thread/resume",
  "thread/turns/list",
  "turn/start",
  "turn/interrupt",
] as const;

export const NOTIFICATION_METHODS = [
  "account/login/completed",
  "turn/started",
  "turn/completed",
  "item/agentMessage/delta",
  "item/completed",
  "error",
  "thread/status/changed",
] as const;

export const SERVER_REQUEST_METHODS = [
  "item/commandExecution/requestApproval",
  "item/fileChange/requestApproval",
  "item/permissions/requestApproval",
] as const;

/** The per-method protocol surface: params/result shapes by method name. */
export interface ProtocolMethods {
  "account/read": { params: AccountReadParams; result: AccountReadResult };
  "account/login/start": { params: AccountLoginStartParams; result: AccountLoginStartResult };
  "account/login/cancel": { params: AccountLoginCancelParams; result: AccountLoginCancelResult };
  "account/logout": { params: undefined; result: AccountLogoutResult };
  "getAuthStatus": { params: GetAuthStatusParams; result: GetAuthStatusResult };
  "thread/list": { params: ThreadListParams; result: ThreadListResult };
  "thread/search": { params: ThreadSearchParams; result: ThreadSearchResult };
  "thread/read": { params: ThreadReadParams; result: ThreadReadResult };
  "thread/resume": { params: ThreadResumeParams; result: ThreadResumeResult };
  "thread/turns/list": { params: ThreadTurnsListParams; result: ThreadTurnsListResult };
  "turn/start": { params: TurnStartParams; result: TurnStartResult };
  "turn/interrupt": { params: TurnInterruptParams; result: TurnInterruptResult };
}

/** Notification params shapes by method name. */
export interface ProtocolNotifications {
  "account/login/completed": { params: unknown };
  "turn/started": { params: TurnStartedParams };
  "turn/completed": { params: TurnCompletedParams };
  "item/agentMessage/delta": { params: ItemAgentMessageDeltaParams };
  "item/completed": { params: ItemCompletedParams };
  "error": { params: ErrorParams };
  "thread/status/changed": { params: ThreadStatusChangedParams };
}

/** Server-initiated request shapes by method name. */
export interface ProtocolServerRequests {
  "item/commandExecution/requestApproval": { params: ItemCommandExecutionRequestApprovalParams; response: ItemCommandExecutionRequestApprovalResponse };
  "item/fileChange/requestApproval": { params: ItemFileChangeRequestApprovalParams; response: ItemFileChangeRequestApprovalResponse };
  "item/permissions/requestApproval": { params: ItemPermissionsRequestApprovalParams; response: unknown };
}

export type ProtocolMethodName = keyof ProtocolMethods;
export type ProtocolNotificationName = keyof ProtocolNotifications;
export type ProtocolServerRequestName = keyof ProtocolServerRequests;

