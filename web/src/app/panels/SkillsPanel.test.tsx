// @vitest-environment jsdom
// The skills-surface tests (WEB-002): the J-04 discovery law — the
// locked-skill state shows the named unlock path — plus the protocol-
// native skill reference form.

import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SkillsPanelView } from "./SkillsPanel";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { createTranslator } from "../../strings/en";

const t = createTranslator("en");
const SKILL_LISTING_CAPABILITY =
  protocolCapabilityStatuses().find((status) => status.surface === "skill-listing") ?? null;

function renderPanel(overrides: Partial<Parameters<typeof SkillsPanelView>[0]> = {}) {
  const onReferenceSkill = vi.fn().mockResolvedValue(undefined);
  const props = {
    t,
    onReferenceSkill,
    capability: SKILL_LISTING_CAPABILITY,
    ...overrides,
  };
  const container = render(<SkillsPanelView {...props} />);
  return { container, onReferenceSkill };
}

describe("the skills surface", () => {
  afterEach(() => {
    cleanup();
  });

  it("references a skill through the protocol's own input variant", async () => {
    const { container, onReferenceSkill } = renderPanel();
    fireEvent.change(container.getByTestId("skill-name-input"), {
      target: { value: "seasonal-planting" },
    });
    fireEvent.change(container.getByTestId("skill-path-input"), {
      target: { value: "/skills/seasonal-planting.md" },
    });
    fireEvent.change(container.getByTestId("skill-note-input"), {
      target: { value: "for the spring beds" },
    });
    fireEvent.click(container.getByTestId("skill-use-button"));
    await waitFor(() => {
      expect(onReferenceSkill).toHaveBeenCalledWith(
        "seasonal-planting",
        "/skills/seasonal-planting.md",
        "for the spring beds",
      );
    });
  });

  it("sends a null note when the note is empty", async () => {
    const { container, onReferenceSkill } = renderPanel();
    fireEvent.change(container.getByTestId("skill-name-input"), { target: { value: "s" } });
    fireEvent.change(container.getByTestId("skill-path-input"), { target: { value: "/s.md" } });
    fireEvent.click(container.getByTestId("skill-use-button"));
    await waitFor(() => {
      expect(onReferenceSkill).toHaveBeenCalledWith("s", "/s.md", null);
    });
  });

  it("keeps the form disabled until name and location are present", () => {
    const { container } = renderPanel();
    const button = container.getByTestId("skill-use-button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    fireEvent.change(container.getByTestId("skill-name-input"), { target: { value: "s" } });
    expect(button.disabled).toBe(true);
    fireEvent.change(container.getByTestId("skill-path-input"), { target: { value: "/s.md" } });
    expect(button.disabled).toBe(false);
  });

  it("Enter in the location field sends the reference (keyboard path)", async () => {
    const { container, onReferenceSkill } = renderPanel();
    fireEvent.change(container.getByTestId("skill-name-input"), { target: { value: "s" } });
    fireEvent.change(container.getByTestId("skill-path-input"), { target: { value: "/s.md" } });
    fireEvent.keyDown(container.getByTestId("skill-path-input"), { key: "Enter" });
    await waitFor(() => {
      expect(onReferenceSkill).toHaveBeenCalled();
    });
  });

  it("surfaces a failed reference with a named error, never silently", async () => {
    const onReferenceSkill = vi.fn().mockRejectedValue(new Error("the runtime refused the skill"));
    const view = render(
      <SkillsPanelView t={t} onReferenceSkill={onReferenceSkill} capability={SKILL_LISTING_CAPABILITY} />,
    );
    fireEvent.change(view.getByTestId("skill-name-input"), { target: { value: "s" } });
    fireEvent.change(view.getByTestId("skill-path-input"), { target: { value: "/s.md" } });
    fireEvent.click(view.getByTestId("skill-use-button"));
    await waitFor(() => {
      expect(view.getByRole("alert").textContent).toContain("the runtime refused the skill");
    });
  });

  it("shows the locked-skill unlock path (the J-04 discovery law)", () => {
    const { container } = renderPanel();
    const gap = container.getByTestId("gap-skill-listing");
    const text = gap.textContent ?? "";
    expect(gap.getAttribute("role")).toBe("note");
    expect(text).toContain("Listing the skills installed");
    expect(text).toContain("unlocking");
    expect(text).toContain("How it unlocks");
    expect(text).toContain("Reference an installed skill by name");
  });
});
