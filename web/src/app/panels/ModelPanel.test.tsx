// @vitest-environment jsdom
// The model/provider-selection surface tests (WEB-002): the current
// model/effort/provider truth, the choice that rides the next turn, and
// the named catalog gap with its regeneration recovery path.

import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ModelPanelView } from "./ModelPanel";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { createTranslator } from "../../strings/en";
import type { Account } from "../../protocol/generated";

const t = createTranslator("en");
const MODEL_LISTING_CAPABILITY =
  protocolCapabilityStatuses().find((status) => status.surface === "model-listing") ?? null;

function renderPanel(overrides: Partial<Parameters<typeof ModelPanelView>[0]> = {}) {
  const onApplyChoice = vi.fn();
  const props = {
    t,
    currentModel: "gpt-5.6-sol",
    currentEffort: "high",
    account: { type: "chatgpt", planType: "pro" } as Account,
    choice: { model: null, effort: null },
    onApplyChoice,
    capability: MODEL_LISTING_CAPABILITY,
    ...overrides,
  };
  const container = render(<ModelPanelView {...props} />);
  return { container, onApplyChoice };
}

describe("the model/provider selection surface", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders the session's reported model, effort and provider", () => {
    const { container } = renderPanel();
    expect(container.getByTestId("model-current-model").textContent).toBe("gpt-5.6-sol");
    expect(container.getByTestId("model-current-effort").textContent).toBe("high");
    const body = container.getByTestId("model-panel").textContent ?? "";
    expect(body).toContain("Your ChatGPT account");
  });

  it("names the unreported state honestly", () => {
    const { container } = renderPanel({ currentModel: null, currentEffort: null });
    expect(container.getByTestId("model-current-model").textContent).toContain("Not reported");
    expect(container.getByTestId("model-current-effort").textContent).toContain("Not reported");
  });

  it("applies the model and effort choice for the next turn", () => {
    const { container, onApplyChoice } = renderPanel();
    fireEvent.change(container.getByTestId("model-input"), { target: { value: "gpt-5.6-mini" } });
    fireEvent.change(container.getByTestId("model-effort-input"), { target: { value: "medium" } });
    fireEvent.click(container.getByTestId("model-apply"));
    expect(onApplyChoice).toHaveBeenCalledWith("gpt-5.6-mini", "medium");
    expect(container.getByTestId("model-applied").textContent).toContain("gpt-5.6-mini");
    expect(container.getByTestId("model-applied").textContent).toContain("medium");
  });

  it("applies with Enter (the keyboard path)", () => {
    const { container, onApplyChoice } = renderPanel();
    fireEvent.change(container.getByTestId("model-input"), { target: { value: "m2" } });
    fireEvent.keyDown(container.getByTestId("model-effort-input"), { key: "Enter" });
    expect(onApplyChoice).toHaveBeenCalledWith("m2", null);
  });

  it("clearing returns to the runtime default", () => {
    const { container, onApplyChoice } = renderPanel({ choice: { model: "m2", effort: null } });
    fireEvent.click(container.getByTestId("model-clear"));
    expect(onApplyChoice).toHaveBeenCalledWith(null, null);
  });

  it("shows the applied choice when the panel reopens with one", () => {
    const { container } = renderPanel({ choice: { model: "gpt-5.6-mini", effort: "low" } });
    expect(container.getByTestId("model-applied").textContent).toContain("gpt-5.6-mini");
    expect(container.getByTestId("model-input").getAttribute("value")).toBe("gpt-5.6-mini");
  });

  it("renders the named catalog gap with the regeneration recovery path", () => {
    const { container } = renderPanel();
    const gap = container.getByTestId("gap-model-listing");
    const text = gap.textContent ?? "";
    expect(gap.getAttribute("role")).toBe("note");
    expect(text).toContain("Browsing the model catalog");
    expect(text).toContain("no model-listing method");
    expect(text).toContain("Enter the model id directly");
    expect(text).toContain("Regenerating the protocol snapshot");
  });

  it("labels the provider truthfully for every account shape", () => {
    const apiKey = renderPanel({ account: { type: "apiKey" } });
    expect(apiKey.container.getByTestId("model-panel").textContent).toContain("Your API key");
    cleanup();
    const signedOut = renderPanel({ account: null });
    expect(signedOut.container.getByTestId("model-panel").textContent).toContain("Not signed in");
  });
});
