// The artifacts surface (WEB-002, scope 5): the app-server's artifact
// methods are not in the frozen protocol snapshot (the named gap for
// documents/sheets/versioned artifacts), but the protocol DOES report
// task items (`thread/turns/list`). This surface lists EXACTLY those
// items — labeled by their own protocol type, never re-typed — with
// honest loading/error/empty/success states.

import { useEffect, useRef } from "react";
import { useFlauzApp } from "../../state/app";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import type { SessionItem } from "../../state/items";
import type { Translator } from "../../strings/en";
import { GapCard } from "./GapCard";
import { PanelFrame } from "./PanelFrame";

const ARTIFACTS_CAPABILITY = protocolCapabilityStatuses().find(
  (status) => status.surface === "artifacts",
)!;

const ITEM_KIND_LABEL: Record<string, string> = {
  agentMessage: "artifacts.kind.agentMessage",
  commandExecution: "artifacts.kind.commandExecution",
  fileChange: "artifacts.kind.fileChange",
};

export function itemKindLabelKey(type: string): string {
  return ITEM_KIND_LABEL[type] ?? "artifacts.kind.other";
}

export function ArtifactsPanel() {
  const { state, t, refreshSessionItems } = useFlauzApp();
  // Load the reported items exactly once per mount (the app-level
  // turn-completion hook and the Refresh control keep them current).
  const loadedOnceRef = useRef(false);
  useEffect(() => {
    if (loadedOnceRef.current) {
      return;
    }
    loadedOnceRef.current = true;
    refreshSessionItems();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return (
    <ArtifactsPanelView
      t={t}
      items={state.sessionItems}
      loadState={state.sessionItemsState}
      error={state.sessionItemsError}
      onRefresh={refreshSessionItems}
      capability={ARTIFACTS_CAPABILITY}
    />
  );
}

export function ArtifactsPanelView({
  t,
  items,
  loadState,
  error,
  onRefresh,
  capability,
}: {
  t: Translator;
  items: SessionItem[];
  loadState: "unloaded" | "loading" | "ready" | "error";
  error: string | null;
  onRefresh: () => void;
  capability: (typeof ARTIFACTS_CAPABILITY) | null;
}) {
  return (
    <PanelFrame
      label={t("rail.artifacts")}
      title={t("rail.artifacts")}
      description={t("artifacts.items.description")}
      testId="artifacts-panel"
    >
      <section aria-label={t("artifacts.items.title")}>
        <h3 className="flauz-panel-section-title">{t("artifacts.items.title")}</h3>
        {loadState === "loading" || loadState === "unloaded" ? (
          <p role="status" data-testid="artifacts-loading">
            {t("artifacts.items.loading")}
          </p>
        ) : null}
        {loadState === "error" ? (
          <div className="flauz-card" role="alert" data-testid="artifacts-error">
            <p className="flauz-error-text">{t("artifacts.error", { reason: error ?? "" })}</p>
            <button type="button" className="flauz-button" onClick={onRefresh}>
              {t("artifacts.refresh")}
            </button>
          </div>
        ) : null}
        {loadState === "ready" && items.length === 0 ? (
          <p className="flauz-empty-description" data-testid="artifacts-empty">
            {t("artifacts.items.empty")}
          </p>
        ) : null}
        {loadState === "ready" && items.length > 0 ? (
          <ul className="flauz-item-list" data-testid="artifacts-items">
            {items.map((item) => (
              <li className="flauz-item" key={item.id} data-testid="artifact-item" data-item-type={item.type}>
                <span className="flauz-item-kind" data-item-type={item.type}>
                  {t(itemKindLabelKey(item.type) as Parameters<Translator>[0])}
                </span>
                <span className="flauz-item-body">
                  {item.command !== undefined ? item.command : (item.text ?? "")}
                </span>
              </li>
            ))}
          </ul>
        ) : null}
        <div className="flauz-panel-actions">
          <button type="button" className="flauz-button" onClick={onRefresh} data-testid="artifacts-refresh">
            {t("artifacts.refresh")}
          </button>
        </div>
      </section>
      {capability === null ? null : (
        <GapCard
          capability={capability}
          t={t}
          whatKey="artifacts.gap.what"
          whyKey="artifacts.gap.why"
          unlockKey="artifacts.gap.unlock"
        />
      )}
    </PanelFrame>
  );
}
