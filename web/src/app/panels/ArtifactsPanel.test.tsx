// @vitest-environment jsdom
// The artifacts-surface tests (WEB-002): honest empty/loading/error/
// success states over the protocol's own task items, the item-kind
// labels, and the named documents/sheets gap.

import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ArtifactsPanelView, itemKindLabelKey } from "./ArtifactsPanel";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { createTranslator } from "../../strings/en";
import type { SessionItem } from "../../state/items";

const t = createTranslator("en");
const ARTIFACTS_CAPABILITY =
  protocolCapabilityStatuses().find((status) => status.surface === "artifacts") ?? null;

const ITEMS: SessionItem[] = [
  { id: "i1", type: "agentMessage", text: "Done: the planting plan." },
  { id: "i2", type: "commandExecution", command: "npm install --no-audit" },
  { id: "i3", type: "fileChange", text: "planning.md — 14 lines changed" },
];

function renderPanel(overrides: Partial<Parameters<typeof ArtifactsPanelView>[0]> = {}) {
  const onRefresh = vi.fn();
  const props = {
    t,
    items: ITEMS,
    loadState: "ready" as const,
    error: null,
    onRefresh,
    capability: ARTIFACTS_CAPABILITY,
    ...overrides,
  };
  const container = render(<ArtifactsPanelView {...props} />);
  return { container, onRefresh };
}

describe("the artifacts surface", () => {
  afterEach(() => {
    cleanup();
  });

  it("lists the items the runtime reported, labeled by their own protocol type", () => {
    const { container } = renderPanel();
    const items = container.getAllByTestId("artifact-item");
    expect(items).toHaveLength(3);
    expect(items[0]?.getAttribute("data-item-type")).toBe("agentMessage");
    expect(items[0]?.textContent).toContain("Done: the planting plan.");
    expect(items[1]?.getAttribute("data-item-type")).toBe("commandExecution");
    expect(items[1]?.textContent).toContain("npm install --no-audit");
    expect(items[2]?.getAttribute("data-item-type")).toBe("fileChange");
    expect(items[2]?.textContent).toContain("planning.md — 14 lines changed");
  });

  it("uses domain-neutral kind labels, never invents product concepts", () => {
    expect(t(itemKindLabelKey("agentMessage") as Parameters<typeof t>[0])).toBe("Agent message");
    expect(t(itemKindLabelKey("commandExecution") as Parameters<typeof t>[0])).toBe("Command");
    expect(t(itemKindLabelKey("fileChange") as Parameters<typeof t>[0])).toBe("File change");
    expect(t(itemKindLabelKey("mysteryKind") as Parameters<typeof t>[0])).toBe("Item");
  });

  it("renders the honest loading state", () => {
    const { container } = renderPanel({ items: [], loadState: "loading" });
    expect(container.getByTestId("artifacts-loading").textContent).toContain("Loading");
  });

  it("renders the honest empty state with the next step", () => {
    const { container } = renderPanel({ items: [], loadState: "ready" });
    expect(container.getByTestId("artifacts-empty").textContent).toContain("Nothing reported yet");
  });

  it("renders the honest error state with the named cause and recovery", () => {
    const { container } = renderPanel({ items: [], loadState: "error", error: "the runtime is restarting" });
    const errorBox = container.getByTestId("artifacts-error");
    expect(errorBox.getAttribute("role")).toBe("alert");
    expect(errorBox.textContent).toContain("the runtime is restarting");
    expect(errorBox.textContent).toContain("Refresh");
  });

  it("refreshes on demand", () => {
    const { container, onRefresh } = renderPanel();
    fireEvent.click(container.getByTestId("artifacts-refresh"));
    expect(onRefresh).toHaveBeenCalledTimes(1);
  });

  it("names the documents/sheets gap with its recovery path", () => {
    const { container } = renderPanel();
    const gap = container.getByTestId("gap-artifacts");
    const text = gap.textContent ?? "";
    expect(gap.getAttribute("role")).toBe("note");
    expect(text).toContain("Documents, spreadsheets and versioned artifacts");
    expect(text).toContain("no artifact, document or sheet methods");
    expect(text).toContain("How it unlocks");
  });
});
