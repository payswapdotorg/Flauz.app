// The protocol-derived capability engine (WEB-002).
//
// The Wave-6 kernel addendum §1 law: the app-server JSON-RPC protocol is
// the ONLY capability contract — a capability the frozen protocol surface
// does not carry is a NAMED GAP with a recovery path, never a fabricated
// one. This module makes that law MECHANICAL: every capability surface
// derives its availability from the generated protocol constants
// (`REQUEST_METHODS` — the schema-export snapshot), so the web client
// can never claim a capability the protocol does not expose.
//
// Two flavors of truth live here:
//   1. METHOD-NAMESPACE capabilities — does the snapshot carry any
//      `<namespace>/*` request method? (environments, model listing,
//      skill listing, collaboration, artifacts.)
//   2. STRUCTURAL capabilities — the turn request itself carries
//      `model` / `effort` / `cwd` params and accepts a `skill` input
//      item (the generated TurnStartParams / UserInput types). These
//      are the protocol surfaces the capability panels RIDE.
//
// The engine is pure: tests feed hypothetical method lists and watch
// statuses flip (the gap cards stay truthful when the snapshot is
// regenerated from the real app-server export).

import { REQUEST_METHODS } from "../protocol/generated";

/** The capability surfaces WEB-002 owns. */
export type CapabilitySurfaceId =
  | "environments"
  | "model-listing"
  | "skill-listing"
  | "collaboration"
  | "artifacts";

/** A namespace capability check: which methods exist, derived — never asserted. */
export interface CapabilityStatus {
  surface: CapabilitySurfaceId;
  /** The protocol namespaces this capability needs (roadmap mapping). */
  namespaces: readonly string[];
  /** The snapshot methods actually found in those namespaces (today: none). */
  presentMethods: string[];
  /** True when at least one method exists in every required namespace. */
  available: boolean;
}

/** The frozen snapshot's request-method inventory (derived at module load). */
export const PROTOCOL_REQUEST_METHODS: readonly string[] = REQUEST_METHODS;

/**
 * Finds the request methods under the given namespaces (first path
 * segment). Example: namespaces `["environment", "env"]` over a list
 * containing `environment/list` match it; `thread/list` never matches.
 */
export function findNamespaceMethods(
  methods: readonly string[],
  namespaces: readonly string[],
): string[] {
  const wanted = new Set(namespaces);
  return methods.filter((method) => {
    const separator = method.indexOf("/");
    const namespace = separator === -1 ? method : method.slice(0, separator);
    return wanted.has(namespace);
  });
}

/** The roadmap capability definitions: what each surface needs. */
const CAPABILITY_NAMESPACES: ReadonlyArray<{
  surface: CapabilitySurfaceId;
  namespaces: readonly string[];
}> = [
  {
    surface: "environments",
    namespaces: ["environment", "env", "sandbox", "workspace/environment"],
  },
  { surface: "model-listing", namespaces: ["model"] },
  { surface: "skill-listing", namespaces: ["skill", "skills"] },
  {
    surface: "collaboration",
    namespaces: ["workspace", "member", "membership", "presence", "collab", "share"],
  },
  { surface: "artifacts", namespaces: ["artifact", "document", "sheet"] },
];

/**
 * Derives every capability surface's status from a request-method
 * inventory (defaults: the frozen snapshot's generated constants).
 */
export function protocolCapabilityStatuses(
  methods: readonly string[] = PROTOCOL_REQUEST_METHODS,
): CapabilityStatus[] {
  return CAPABILITY_NAMESPACES.map((definition) => {
    const presentMethods = findNamespaceMethods(methods, definition.namespaces);
    return {
      surface: definition.surface,
      namespaces: definition.namespaces,
      presentMethods,
      available: presentMethods.length > 0,
    };
  });
}

// ---------------------------------------------------------------------------
// The structural capabilities (the generated turn surface the panels ride).
//
// `turn/start`'s params carry `model`, `effort` and `cwd`, and its input
// array accepts a `{ type: "skill", name, path }` item — all from the
// generated TurnStartParams / UserInput types (src/protocol/generated.ts,
// the schema export). The web client never invents turn params beyond
// these; the tests verify them against schema.json so a snapshot
// regeneration that drops them fails loudly.

/** The turn params the model/provider selection rides (per the generated type). */
export const TURN_MODEL_PARAMS = ["model", "effort"] as const;

/** The turn param the working-directory (execution context) selection rides. */
export const TURN_CWD_PARAM = "cwd" as const;

/** The turn input item type that references a skill (per the generated UserInput). */
export const SKILL_INPUT_TYPE = "skill" as const;
