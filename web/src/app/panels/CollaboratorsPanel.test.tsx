// @vitest-environment jsdom
// The collaboration/presence surface tests (WEB-002): the named protocol
// gap, the shared-decisions truth (approvals as named records), and the
// F9 privacy note on the surface itself.

import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CollaboratorsPanelView } from "./CollaboratorsPanel";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { createTranslator } from "../../strings/en";

const t = createTranslator("en");
const COLLABORATION_CAPABILITY =
  protocolCapabilityStatuses().find((status) => status.surface === "collaboration") ?? null;

function renderPanel(overrides: Partial<Parameters<typeof CollaboratorsPanelView>[0]> = {}) {
  const onFocusApprovals = vi.fn();
  const props = {
    t,
    openApprovals: 0,
    onFocusApprovals,
    capability: COLLABORATION_CAPABILITY,
    ...overrides,
  };
  const container = render(<CollaboratorsPanelView {...props} />);
  return { container, onFocusApprovals };
}

describe("the collaboration/presence surface", () => {
  afterEach(() => {
    cleanup();
  });

  it("names the membership/presence gap with the collaboration contract as the unlock path", () => {
    const { container } = renderPanel();
    const gap = container.getByTestId("gap-collaboration");
    const text = gap.textContent ?? "";
    expect(gap.getAttribute("role")).toBe("note");
    expect(text).toContain("Seeing collaborators on this workspace");
    expect(text).toContain("no membership or presence methods");
    expect(text).toContain("owner, admin, contributor and viewer roles");
  });

  it("states the F9 privacy law on the surface", () => {
    const { container } = renderPanel();
    expect(container.getByTestId("collab-privacy-note").textContent).toContain(
      "private context",
    );
  });

  it("shows the shared-decisions count truthfully", () => {
    const { container } = renderPanel({ openApprovals: 2 });
    expect(container.getByTestId("collab-approval-count").textContent).toContain("2");
    expect(container.getByTestId("collab-approval-count").textContent).toContain("decisions waiting");
  });

  it("names the zero-decisions state honestly", () => {
    const { container } = renderPanel();
    expect(container.getByTestId("collab-approval-count").textContent).toContain(
      "No decisions are waiting",
    );
  });

  it("offers the show-the-decisions action only when decisions exist", () => {
    const empty = renderPanel();
    expect(empty.container.queryByTestId("collab-show-approvals")).toBeNull();
    cleanup();
    const busy = renderPanel({ openApprovals: 1 });
    fireEvent.click(busy.container.getByTestId("collab-show-approvals"));
    expect(busy.onFocusApprovals).toHaveBeenCalledTimes(1);
  });
});
