// @vitest-environment jsdom
// The connection banner component tests (WEB-001): the banner is always
// present, always truthful, and renders exactly one of the four
// canonical states with its named reason.

import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ConnectionBanner } from "../app/ConnectionBanner";
import { FlauzAppProvider } from "./app";
import { connectionReducer, initialConnectionState, renderedState } from "./connection";

function renderBannerInProvider(): HTMLElement {
  const { container } = render(
    <FlauzAppProvider>
      <ConnectionBanner />
    </FlauzAppProvider>,
  );
  return container;
}

describe("the connection banner", () => {
  afterEach(() => {
    cleanup();
  });

  it("is always present with a live status role", () => {
    const container = renderBannerInProvider();
    const banner = container.querySelector('[data-testid="connection-banner"]');
    expect(banner).not.toBeNull();
    expect(banner?.getAttribute("role")).toBe("status");
    expect(banner?.getAttribute("aria-live")).toBe("polite");
  });

  it("renders exactly one of the four canonical states", () => {
    const container = renderBannerInProvider();
    const banner = container.querySelector('[data-testid="connection-banner"]');
    const dataState = banner?.getAttribute("data-state") ?? "";
    expect(["connected", "authenticating", "reconnecting", "failed"]).toContain(dataState);
  });

  it("never renders without a label", () => {
    const container = renderBannerInProvider();
    const banner = container.querySelector('[data-testid="connection-banner"]');
    const label = banner?.querySelector("strong")?.textContent ?? "";
    expect(label.length).toBeGreaterThan(0);
  });
});

describe("the banner state mapping (pure machine truth)", () => {
  it("every machine state maps to exactly one canonical rendered state", () => {
    const statuses = [
      "idle",
      "connecting",
      "authenticating",
      "connected",
      "reconnecting",
      "failed",
    ] as const;
    const rendered = statuses.map((status) =>
      renderedState({ ...initialConnectionState(), status }),
    );
    expect(rendered).toEqual([
      "reconnecting",
      "reconnecting",
      "authenticating",
      "connected",
      "reconnecting",
      "failed",
    ]);
  });

  it("the failure path surfaces the named reason through the machine", () => {
    let connection = initialConnectionState();
    connection = connectionReducer(connection, { type: "connect_started" });
    connection = connectionReducer(connection, {
      type: "session_denied",
      code: "handshake_required",
      message: "claim a session before sending app-server frames",
    });
    expect(renderedState(connection)).toBe("failed");
    expect(connection.reason).toContain("handshake_required");
    expect(connection.needsUserAction).toBe(true);
  });
});
