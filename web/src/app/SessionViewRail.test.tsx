// @vitest-environment jsdom
// The task-rail integration tests (WEB-002): every capability surface is
// reachable from the session surface, the rail buttons carry honest
// aria-pressed state, panels open, and Escape closes the panel and
// restores focus to its rail button (the 017 law). Rendered through the
// real FlauzAppProvider (the gateway cannot open in jsdom; the provider
// tolerates it exactly like production does).

import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import type { RenderResult } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SessionView } from "./SessionView";
import { FlauzAppProvider } from "../state/app";

function renderSessionView(): RenderResult {
  return render(
    <FlauzAppProvider>
      <SessionView threadId="thread-test-1" />
    </FlauzAppProvider>,
  );
}

const PANELS = [
  { button: "context-button", panel: "context-drawer" },
  { button: "environments-button", panel: "environments-drawer" },
  { button: "model-button", panel: "model-panel" },
  { button: "skills-button", panel: "skills-panel" },
  { button: "collaborators-button", panel: "collaborators-panel" },
  { button: "artifacts-button", panel: "artifacts-panel" },
] as const;

describe("the task control rail (WEB-002)", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders every capability rail button in a labelled group", () => {
    const container = renderSessionView();
    const rail = container.getByTestId("task-rail");
    expect(rail.getAttribute("role")).toBe("group");
    expect(rail.getAttribute("aria-label")).toBe("Task controls");
    for (const { button } of PANELS) {
      expect(container.getByTestId(button), `rail button ${button}`).not.toBeNull();
    }
  });

  it("opens each capability panel with aria-pressed truth", async () => {
    const container = renderSessionView();
    for (const { button, panel } of PANELS) {
      const railButton = container.getByTestId(button) as HTMLButtonElement;
      expect(railButton.getAttribute("aria-pressed")).toBe("false");
      fireEvent.click(railButton);
      await waitFor(() => {
        expect(container.getByTestId(panel), `panel ${panel}`).not.toBeNull();
      });
      expect(railButton.getAttribute("aria-pressed")).toBe("true");
      // Toggling again closes the panel.
      fireEvent.click(railButton);
      await waitFor(() => {
        expect(container.queryByTestId(panel)).toBeNull();
      });
      expect(railButton.getAttribute("aria-pressed")).toBe("false");
    }
  });

  it("shows exactly one panel at a time (the surfaces do not stack)", async () => {
    const container = renderSessionView();
    fireEvent.click(container.getByTestId("model-button"));
    await waitFor(() => {
      expect(container.getByTestId("model-panel")).not.toBeNull();
    });
    fireEvent.click(container.getByTestId("artifacts-button"));
    await waitFor(() => {
      expect(container.queryByTestId("model-panel")).toBeNull();
    });
    expect(container.getByTestId("artifacts-panel")).not.toBeNull();
  });

  it("Escape closes the open panel and restores focus to its rail button (the 017 law)", async () => {
    const container = renderSessionView();
    const railButton = container.getByTestId("skills-button") as HTMLButtonElement;
    railButton.focus();
    fireEvent.click(railButton);
    await waitFor(() => {
      expect(container.getByTestId("skills-panel")).not.toBeNull();
    });
    // Focus moved into the panel (the heading).
    const heading = container.getByTestId("skills-panel").querySelector(".flauz-panel-title");
    expect(document.activeElement).toBe(heading);
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => {
      expect(container.queryByTestId("skills-panel")).toBeNull();
    });
    expect(document.activeElement).toBe(railButton);
  });

  it("opens a panel through the palette/chord event (the fallback discovery path)", async () => {
    const container = renderSessionView();
    window.dispatchEvent(new CustomEvent("flauz:open-panel", { detail: { panel: "model" } }));
    await waitFor(() => {
      expect(container.getByTestId("model-panel")).not.toBeNull();
    });
  });

  it("does not render the Stop control while no turn runs (honest absence)", () => {
    const container = renderSessionView();
    expect(container.queryByTestId("interrupt-button")).toBeNull();
  });

  it("the rail buttons announce with the domain-neutral labels", () => {
    const container = renderSessionView();
    expect(container.getByTestId("model-button").textContent).toBe("Model");
    expect(container.getByTestId("skills-button").textContent).toBe("Skills");
    expect(container.getByTestId("collaborators-button").textContent).toBe("Collaborators");
    expect(container.getByTestId("artifacts-button").textContent).toBe("Artifacts");
  });
});
