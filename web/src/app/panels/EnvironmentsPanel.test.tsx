// @vitest-environment jsdom
// The environments-surface component tests (WEB-002): the named gap is
// protocol-derived and always rendered with its recovery path, and the
// one real environment control (the per-turn working directory) rides
// its callback honestly.

import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EnvironmentsPanelView } from "./EnvironmentsPanel";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { createTranslator } from "../../strings/en";

const t = createTranslator("en");
const ENVIRONMENTS_CAPABILITY =
  protocolCapabilityStatuses().find((status) => status.surface === "environments") ?? null;

function renderPanel(overrides: Partial<Parameters<typeof EnvironmentsPanelView>[0]> = {}) {
  const onApplyCwd = vi.fn();
  const props = {
    t,
    reportedCwd: "/tmp/flauz-lab",
    turnCwd: null,
    onApplyCwd,
    capability: ENVIRONMENTS_CAPABILITY,
    ...overrides,
  };
  const container = render(<EnvironmentsPanelView {...props} />);
  return { container, onApplyCwd };
}

describe("the environments surface", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders the working directory the runtime reported", () => {
    const { container } = renderPanel();
    const reported = container.getByTestId("environments-reported-cwd");
    expect(reported.textContent).toContain("/tmp/flauz-lab");
  });

  it("names its unreported state honestly when the runtime sent none", () => {
    const { container } = renderPanel({ reportedCwd: null });
    expect(container.getByTestId("environments-reported-cwd").textContent).toContain(
      "has not reported",
    );
  });

  it("applies the working-directory choice (the protocol's own per-turn control)", () => {
    const { container, onApplyCwd } = renderPanel();
    fireEvent.change(container.getByTestId("environments-cwd-input"), {
      target: { value: "/tmp/garden" },
    });
    fireEvent.click(container.getByTestId("environments-cwd-apply"));
    expect(onApplyCwd).toHaveBeenCalledWith("/tmp/garden");
    expect(container.getByTestId("environments-cwd-applied").textContent).toContain("/tmp/garden");
  });

  it("Enter applies the working directory (the keyboard path)", () => {
    const { container, onApplyCwd } = renderPanel();
    fireEvent.change(container.getByTestId("environments-cwd-input"), {
      target: { value: "/tmp/garden" },
    });
    fireEvent.keyDown(container.getByTestId("environments-cwd-input"), { key: "Enter" });
    expect(onApplyCwd).toHaveBeenCalledWith("/tmp/garden");
  });

  it("clearing returns to the reported directory", () => {
    const { container, onApplyCwd } = renderPanel({ turnCwd: "/tmp/garden" });
    fireEvent.click(container.getByTestId("environments-cwd-clear"));
    expect(onApplyCwd).toHaveBeenCalledWith(null);
  });

  it("renders the named gap with what, why and the unlock path (J-04 law)", () => {
    const { container } = renderPanel();
    const gap = container.getByTestId("gap-environments");
    expect(gap.getAttribute("role")).toBe("note");
    const text = gap.textContent ?? "";
    expect(text).toContain("Listing, selecting and connecting remote environments");
    expect(text).toContain("carries no environment control methods");
    expect(text).toContain("How it unlocks");
    expect(text).toContain("protocol extension");
  });

  it("stays silent when the protocol carries the capability (regeneration truth)", () => {
    const { container } = renderPanel({
      capability: {
        surface: "environments",
        namespaces: ["environment"],
        presentMethods: ["environment/list"],
        available: true,
      },
    });
    expect(container.queryByTestId("gap-environments")).toBeNull();
  });
});
