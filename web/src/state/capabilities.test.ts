// The capability-engine tests (WEB-002): the named gaps are PROTOCOL-DERIVED
// (never asserted by hand) and the structural capabilities are verified
// against the schema snapshot itself.

import { describe, expect, it } from "vitest";
import schema from "../protocol/schema.json";
import {
  PROTOCOL_REQUEST_METHODS,
  SKILL_INPUT_TYPE,
  TURN_CWD_PARAM,
  TURN_MODEL_PARAMS,
  findNamespaceMethods,
  protocolCapabilityStatuses,
  type CapabilityStatus,
} from "./capabilities";

function statusFor(statuses: CapabilityStatus[], surface: string): CapabilityStatus | undefined {
  return statuses.find((entry) => entry.surface === surface);
}

describe("the protocol-derived capability statuses (frozen snapshot)", () => {
  const statuses = protocolCapabilityStatuses();

  it("derives every WEB-002 surface from the generated method constants", () => {
    const surfaces = statuses.map((status) => status.surface).sort();
    expect(surfaces).toEqual(["artifacts", "collaboration", "environments", "model-listing", "skill-listing"]);
  });

  it("names the environment-control gap from the snapshot (no environment/* methods)", () => {
    const environment = statusFor(statuses, "environments");
    expect(environment?.available).toBe(false);
    expect(environment?.presentMethods).toEqual([]);
  });

  it("names the model-listing gap from the snapshot (no model/* methods)", () => {
    const modelListing = statusFor(statuses, "model-listing");
    expect(modelListing?.available).toBe(false);
    expect(modelListing?.presentMethods).toEqual([]);
  });

  it("names the skill-listing gap from the snapshot (no skill(s)/* methods)", () => {
    const skillListing = statusFor(statuses, "skill-listing");
    expect(skillListing?.available).toBe(false);
    expect(skillListing?.presentMethods).toEqual([]);
  });

  it("names the collaboration/presence gap from the snapshot", () => {
    const collaboration = statusFor(statuses, "collaboration");
    expect(collaboration?.available).toBe(false);
    expect(collaboration?.presentMethods).toEqual([]);
  });

  it("names the artifacts/documents/sheets gap from the snapshot", () => {
    const artifacts = statusFor(statuses, "artifacts");
    expect(artifacts?.available).toBe(false);
    expect(artifacts?.presentMethods).toEqual([]);
  });

  it("derives from the generated REQUEST_METHODS, not a hand-written list", () => {
    // The engine's default input IS the generated constant — the gap
    // claims can never drift from the protocol snapshot.
    expect(PROTOCOL_REQUEST_METHODS).toContain("turn/start");
    expect(PROTOCOL_REQUEST_METHODS).toContain("thread/turns/list");
    expect(PROTOCOL_REQUEST_METHODS).not.toContain("model/list");
  });
});

describe("the engine flips when the protocol grows (regeneration stays truthful)", () => {
  it("marks a surface available once its namespace method exists", () => {
    const grown = protocolCapabilityStatuses([
      ...PROTOCOL_REQUEST_METHODS,
      "environment/list",
      "environment/attach",
    ]);
    const environment = statusFor(grown, "environments");
    expect(environment?.available).toBe(true);
    expect(environment?.presentMethods).toEqual(["environment/list", "environment/attach"]);
  });

  it("matches namespaced methods only on the first path segment", () => {
    expect(findNamespaceMethods(["model/list", "thread/list"], ["model"])).toEqual(["model/list"]);
    expect(findNamespaceMethods(["thread/list"], ["environment", "env"])).toEqual([]);
    expect(findNamespaceMethods(["env/create", "environments/list"], ["environment", "env"])).toEqual(["env/create"]);
  });

  it("keeps other surfaces absent when one grows", () => {
    const grown = protocolCapabilityStatuses([...PROTOCOL_REQUEST_METHODS, "skills/list"]);
    expect(statusFor(grown, "skill-listing")?.available).toBe(true);
    expect(statusFor(grown, "collaboration")?.available).toBe(false);
  });
});

describe("the structural capabilities ride the schema snapshot itself", () => {
  // The model/provider selection and the working-directory control ride
  // turn/start params; the skill reference rides the turn input. These
  // assertions read the captured schema document so a snapshot that
  // drops any of them fails this test loudly.

  it("turn/start carries the model, effort and cwd params", () => {
    const methods = schema.methods as Record<string, { params?: { properties?: Record<string, unknown> } }>;
    const turnStart = methods["turn/start"];
    expect(turnStart).toBeDefined();
    const properties = turnStart?.params?.properties ?? {};
    for (const param of TURN_MODEL_PARAMS) {
      expect(properties[param], `turn/start must carry the ${param} param`).toBeDefined();
    }
    expect(properties[TURN_CWD_PARAM], "turn/start must carry the cwd param").toBeDefined();
  });

  it("the turn input accepts a skill reference item", () => {
    const definitions = schema.definitions as Record<string, unknown>;
    const userInput = definitions["UserInput"] as { oneOf?: Array<{ properties?: Record<string, unknown> }> };
    expect(Array.isArray(userInput.oneOf)).toBe(true);
    const skillVariant = userInput.oneOf?.find((variant) => variant.properties?.["type"] !== undefined);
    expect(skillVariant, "the UserInput variants must exist").toBeDefined();
    const allVariants = userInput.oneOf ?? [];
    const hasSkillType = allVariants.some((variant) => {
      const typeField = variant.properties?.["type"] as { const?: string } | undefined;
      return typeField?.const === SKILL_INPUT_TYPE;
    });
    expect(hasSkillType, "UserInput must carry the skill reference variant").toBe(true);
  });

  it("thread/resume reports the session's model and reasoning effort", () => {
    const methods = schema.methods as Record<string, { result?: { properties?: Record<string, unknown> } }>;
    const properties = methods["thread/resume"]?.result?.properties ?? {};
    expect(properties["model"]).toBeDefined();
    expect(properties["reasoningEffort"]).toBeDefined();
  });
});
