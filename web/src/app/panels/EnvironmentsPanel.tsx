// The environments surface (WEB-002, scope 1): remote environment
// control through the app-server protocol's environment methods. The
// frozen snapshot carries none (protocol-derived), so the surface
// renders the NAMED gap with its recovery path — plus the one real
// environment control the protocol does carry: the per-turn working
// directory (turn/start's own cwd param) and the working directory the
// runtime reports for the session (thread summary).

import { useEffect, useState } from "react";
import { useFlauzApp } from "../../state/app";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { GapCard } from "./GapCard";
import { PanelFrame } from "./PanelFrame";

const ENVIRONMENTS_CAPABILITY = protocolCapabilityStatuses().find(
  (status) => status.surface === "environments",
)!;

export function EnvironmentsPanel() {
  const { state, t, setTurnCwd } = useFlauzApp();
  return (
    <EnvironmentsPanelView
      t={t}
      reportedCwd={state.currentSession?.cwd ?? null}
      turnCwd={state.turnCwd}
      onApplyCwd={setTurnCwd}
      capability={ENVIRONMENTS_CAPABILITY}
    />
  );
}

export function EnvironmentsPanelView({
  t,
  reportedCwd,
  turnCwd,
  onApplyCwd,
  capability,
}: {
  t: ReturnType<typeof useFlauzApp>["t"];
  reportedCwd: string | null;
  turnCwd: string | null;
  onApplyCwd: (cwd: string | null) => void;
  capability: (typeof ENVIRONMENTS_CAPABILITY) | null;
}) {
  const [draft, setDraft] = useState(reportedCwd ?? "");
  const [applied, setApplied] = useState<string | null>(turnCwd);

  useEffect(() => {
    setDraft(reportedCwd ?? "");
    setApplied(turnCwd);
  }, [reportedCwd, turnCwd]);

  const onApply = () => {
    const value = draft.trim();
    const next = value === "" ? null : value;
    onApplyCwd(next);
    setApplied(next);
  };

  const onClear = () => {
    onApplyCwd(null);
    setDraft(reportedCwd ?? "");
    setApplied(null);
  };

  return (
    <PanelFrame
      label={t("environments.title")}
      title={t("environments.title")}
      description={t("rail.environments")}
      testId="environments-drawer"
    >
      <section aria-label={t("environments.current.title")}>
        <h3 className="flauz-panel-section-title">{t("environments.current.title")}</h3>
        <p className="flauz-kv">
          <span className="flauz-kv-key">{t("environments.current.cwd")}</span>
          <code className="flauz-kv-value" data-testid="environments-reported-cwd">
            {reportedCwd ?? t("environments.current.none")}
          </code>
        </p>
      </section>
      <section aria-label={t("environments.cwd.label")}>
        <h3 className="flauz-panel-section-title">{t("environments.cwd.label")}</h3>
        <label className="flauz-label" htmlFor="environments-cwd-input">
          {t("environments.cwd.label")}
        </label>
        <input
          id="environments-cwd-input"
          className="flauz-input"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              onApply();
            }
          }}
          data-testid="environments-cwd-input"
        />
        <div className="flauz-panel-actions">
          <button type="button" className="flauz-button flauz-button-primary" onClick={onApply} data-testid="environments-cwd-apply">
            {t("environments.cwd.apply")}
          </button>
          {applied === null ? null : (
            <button type="button" className="flauz-button" onClick={onClear} data-testid="environments-cwd-clear">
              {t("environments.cwd.clear")}
            </button>
          )}
        </div>
        <p className="flauz-hint" role="status" data-testid="environments-cwd-applied">
          {applied === null ? t("environments.cwd.hint") : t("environments.cwd.applied", { path: applied })}
        </p>
      </section>
      {capability === null ? null : (
        <GapCard
          capability={capability}
          t={t}
          whatKey="environments.gap.what"
          whyKey="environments.gap.why"
          unlockKey="environments.gap.unlock"
        />
      )}
    </PanelFrame>
  );
}
