// The session-item extraction (WEB-002): the artifacts surface renders
// EXACTLY what the protocol's own turns page reports (`thread/turns/list`
// data entries), defensively decoded (the generated TurnsPage carries
// `unknown[]`) and hard-bounded. Items are labeled by their own `type`
// field — never re-typed into product concepts the protocol did not send.

/** One item the app-server reported for a session's turns. */
export interface SessionItem {
  id: string;
  /** The item's own protocol type (`agentMessage`, `commandExecution`, …). */
  type: string;
  text?: string;
  command?: string;
}

/** The bound on rendered items (bounded lists — the AGENTS.md law). */
export const MAX_SESSION_ITEMS = 200;

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function readOptionalString(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === "string" ? value : undefined;
}

function decodeItem(record: Record<string, unknown>): SessionItem | null {
  const type = readOptionalString(record, "type");
  if (type === undefined) {
    return null;
  }
  const id = readOptionalString(record, "id") ?? `${type}-${Math.random().toString(36).slice(2, 10)}`;
  const text = readOptionalString(record, "text");
  const command = readOptionalString(record, "command");
  if (text === undefined && command === undefined) {
    return { id, type };
  }
  return text === undefined ? { id, type, command } : command === undefined ? { id, type, text } : { id, type, text, command };
}

/**
 * Extracts session items from a turns-page data array. Accepts both the
 * turn-page entry shape (`{ id, item: { type, text, command } }` — what
 * the app-server reports) and a bare item shape; anything without a
 * `type` is skipped (honest, never guessed).
 */
export function extractSessionItems(data: unknown[]): SessionItem[] {
  const items: SessionItem[] = [];
  for (const entry of data) {
    if (!isRecord(entry)) {
      continue;
    }
    let candidate: Record<string, unknown> | null = entry;
    const nested = entry["item"];
    if (isRecord(nested)) {
      candidate = { ...nested };
      const id = readOptionalString(entry, "id");
      if (id !== undefined && candidate["id"] === undefined) {
        candidate["id"] = id;
      }
    }
    const item = decodeItem(candidate);
    if (item !== null) {
      items.push(item);
    }
    if (items.length >= MAX_SESSION_ITEMS) {
      break;
    }
  }
  return items;
}
