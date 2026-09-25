// The model/provider selection surface (WEB-002, scope 2): the picker
// rides the protocol's own turn/start params (model + effort) — the
// generated TurnStartParams fields — and shows the session's reported
// model (thread/resume) and the provider truth (account/read). The
// model CATALOG cannot be enumerated (no listing method in the frozen
// snapshot — the named gap), so the id is entered directly.

import { useEffect, useState } from "react";
import { useFlauzApp } from "../../state/app";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { sameModelChoice, type ModelChoice } from "../../state/turnParams";
import type { Account } from "../../protocol/generated";
import type { Translator } from "../../strings/en";
import { GapCard } from "./GapCard";
import { PanelFrame } from "./PanelFrame";

const MODEL_LISTING_CAPABILITY = protocolCapabilityStatuses().find(
  (status) => status.surface === "model-listing",
)!;

function providerLabel(account: Account | null, t: Translator): string {
  if (account === null) {
    return t("model.provider.none");
  }
  switch (account.type) {
    case "chatgpt":
      return t("model.provider.chatgpt");
    case "apiKey":
      return t("model.provider.apikey");
    case "amazonBedrock":
      return t("model.provider.bedrock");
  }
}

export function ModelPanel() {
  const { state, t, setModelChoice } = useFlauzApp();
  return (
    <ModelPanelView
      t={t}
      currentModel={state.currentSessionModel}
      currentEffort={state.currentSessionEffort}
      account={state.account}
      choice={state.modelChoice}
      onApplyChoice={(model, effort) => setModelChoice(model, effort)}
      capability={MODEL_LISTING_CAPABILITY}
    />
  );
}

export function ModelPanelView({
  t,
  currentModel,
  currentEffort,
  account,
  choice,
  onApplyChoice,
  capability,
}: {
  t: Translator;
  currentModel: string | null;
  currentEffort: string | null;
  account: Account | null;
  choice: ModelChoice;
  onApplyChoice: (model: string | null, effort: string | null) => void;
  capability: (typeof MODEL_LISTING_CAPABILITY) | null;
}) {
  const [modelDraft, setModelDraft] = useState(choice.model ?? "");
  const [effortDraft, setEffortDraft] = useState(choice.effort ?? "");
  const [applied, setApplied] = useState<ModelChoice>(choice);

  useEffect(() => {
    setModelDraft(choice.model ?? "");
    setEffortDraft(choice.effort ?? "");
    setApplied(choice);
  }, [choice]);

  const onApply = () => {
    const model = modelDraft.trim() === "" ? null : modelDraft.trim();
    const effort = effortDraft.trim() === "" ? null : effortDraft.trim();
    onApplyChoice(model, effort);
    setApplied({ model, effort });
  };

  const onClear = () => {
    onApplyChoice(null, null);
    setModelDraft("");
    setEffortDraft("");
    setApplied({ model: null, effort: null });
  };

  const appliedText =
    applied.model === null
      ? null
      : applied.effort === null
        ? t("model.next.applied.modelOnly", { model: applied.model })
        : t("model.next.applied", { model: applied.model, effort: applied.effort });

  return (
    <PanelFrame
      label={t("rail.model")}
      title={t("rail.model")}
      description={t("model.next.description")}
      testId="model-panel"
    >
      <section aria-label={t("model.current.title")}>
        <h3 className="flauz-panel-section-title">{t("model.current.title")}</h3>
        <dl className="flauz-kv-list">
          <div className="flauz-kv-row">
            <dt>{t("model.current.model")}</dt>
            <dd data-testid="model-current-model">{currentModel ?? t("model.current.unreported")}</dd>
          </div>
          <div className="flauz-kv-row">
            <dt>{t("model.current.effort")}</dt>
            <dd data-testid="model-current-effort">{currentEffort ?? t("model.current.unreported")}</dd>
          </div>
          <div className="flauz-kv-row">
            <dt>{t("model.current.provider")}</dt>
            <dd>{providerLabel(account, t)}</dd>
          </div>
        </dl>
      </section>
      <section aria-label={t("model.next.title")}>
        <h3 className="flauz-panel-section-title">{t("model.next.title")}</h3>
        <label className="flauz-label" htmlFor="model-input">
          {t("model.next.model.label")}
        </label>
        <input
          id="model-input"
          className="flauz-input"
          placeholder={t("model.next.model.placeholder")}
          value={modelDraft}
          onChange={(event) => setModelDraft(event.target.value)}
          data-testid="model-input"
        />
        <label className="flauz-label" htmlFor="model-effort-input" style={{ marginTop: 10 }}>
          {t("model.next.effort.label")}
        </label>
        <input
          id="model-effort-input"
          className="flauz-input"
          placeholder={t("model.next.effort.placeholder")}
          value={effortDraft}
          onChange={(event) => setEffortDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              onApply();
            }
          }}
          data-testid="model-effort-input"
        />
        <div className="flauz-panel-actions">
          <button type="button" className="flauz-button flauz-button-primary" onClick={onApply} data-testid="model-apply">
            {t("model.next.apply")}
          </button>
          {sameModelChoice(applied, { model: null, effort: null }) ? null : (
            <button type="button" className="flauz-button" onClick={onClear} data-testid="model-clear">
              {t("model.next.clear")}
            </button>
          )}
        </div>
        <p className="flauz-hint" role="status" data-testid="model-applied">
          {appliedText ?? t("model.next.description")}
        </p>
      </section>
      {capability === null ? null : (
        <GapCard
          capability={capability}
          t={t}
          whatKey="model.gap.what"
          whyKey="model.gap.why"
          unlockKey="model.gap.unlock"
        />
      )}
    </PanelFrame>
  );
}
