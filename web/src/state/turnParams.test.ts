// The turn-param builder + session-item extraction tests (WEB-002).

import { describe, expect, it } from "vitest";
import type { UserInput } from "../protocol/generated";
import {
  NO_MODEL_CHOICE,
  buildSkillReferenceInput,
  buildTurnStart,
  sameModelChoice,
  type ModelChoice,
} from "./turnParams";
import { MAX_SESSION_ITEMS, extractSessionItems } from "./items";

describe("buildTurnStart (the per-turn choice rides the protocol's own params)", () => {
  const text: UserInput[] = [{ type: "text", text: "plan the garden" }];

  it("carries no execution choice when none is set", () => {
    expect(buildTurnStart("t1", text, null, null)).toEqual({ threadId: "t1", input: text });
  });

  it("carries an empty choice as no choice (the runtime default)", () => {
    expect(buildTurnStart("t1", text, NO_MODEL_CHOICE, null)).toEqual({ threadId: "t1", input: text });
  });

  it("rides the model and effort on the turn params", () => {
    const choice: ModelChoice = { model: "gpt-5.6-mini", effort: "medium" };
    expect(buildTurnStart("t1", text, choice, null)).toEqual({
      threadId: "t1",
      input: text,
      model: "gpt-5.6-mini",
      effort: "medium",
    });
  });

  it("rides the working directory on the turn params", () => {
    expect(buildTurnStart("t1", text, null, "/tmp/garden")).toEqual({
      threadId: "t1",
      input: text,
      cwd: "/tmp/garden",
    });
  });

  it("rides model, effort and cwd together, trimming whitespace", () => {
    const choice: ModelChoice = { model: " m2 ", effort: " low " };
    expect(buildTurnStart("t1", text, choice, " /tmp/x ")).toEqual({
      threadId: "t1",
      input: text,
      model: "m2",
      effort: "low",
      cwd: "/tmp/x",
    });
  });

  it("ignores whitespace-only choices", () => {
    const choice: ModelChoice = { model: "   ", effort: "" };
    expect(buildTurnStart("t1", text, choice, "   ")).toEqual({ threadId: "t1", input: text });
  });
});

describe("buildSkillReferenceInput (the protocol's own skill input variant)", () => {
  it("builds the skill reference alone", () => {
    expect(buildSkillReferenceInput("planting", "/skills/planting.md", null)).toEqual([
      { type: "skill", name: "planting", path: "/skills/planting.md" },
    ]);
  });

  it("appends an optional text note after the reference", () => {
    expect(buildSkillReferenceInput("planting", "/skills/planting.md", "  use it now ")).toEqual([
      { type: "skill", name: "planting", path: "/skills/planting.md" },
      { type: "text", text: "use it now" },
    ]);
  });

  it("drops a whitespace-only note", () => {
    expect(buildSkillReferenceInput("planting", "/skills/planting.md", "   ")).toEqual([
      { type: "skill", name: "planting", path: "/skills/planting.md" },
    ]);
  });
});

describe("sameModelChoice", () => {
  it("treats null and undefined as no choice", () => {
    expect(sameModelChoice(null, undefined)).toBe(true);
    expect(sameModelChoice({ model: "m", effort: null }, { model: "m", effort: null })).toBe(true);
    expect(sameModelChoice({ model: "m", effort: "low" }, { model: "m", effort: "high" })).toBe(false);
  });
});

describe("extractSessionItems (honest decoding of the turns page)", () => {
  it("decodes the { id, item } turn-page entries", () => {
    const items = extractSessionItems([
      { id: "i1", item: { type: "agentMessage", text: "Done: the plan." } },
      { id: "i2", item: { type: "commandExecution", command: "npm install" } },
    ]);
    expect(items).toEqual([
      { id: "i1", type: "agentMessage", text: "Done: the plan." },
      { id: "i2", type: "commandExecution", command: "npm install" },
    ]);
  });

  it("decodes bare item shapes and synthesizes bounded ids", () => {
    const items = extractSessionItems([{ type: "fileChange", text: "planning.md — 14 lines" }]);
    expect(items).toHaveLength(1);
    expect(items[0]?.type).toBe("fileChange");
    expect(items[0]?.text).toBe("planning.md — 14 lines");
    expect(typeof items[0]?.id).toBe("string");
  });

  it("skips entries without a type (never guesses a product concept)", () => {
    expect(extractSessionItems([{}, { id: "x" }, "string", 42, null])).toEqual([]);
  });

  it("keeps the hard bound", () => {
    const many = Array.from({ length: MAX_SESSION_ITEMS + 50 }, (_, index) => ({
      id: `i${index}`,
      item: { type: "agentMessage", text: "x" },
    }));
    expect(extractSessionItems(many)).toHaveLength(MAX_SESSION_ITEMS);
  });
});
