// The new-task composer (J-01 web variant; WEB-002 adds the contextual
// model affordance): describe the objective in plain language; the turn
// starts on the agent runtime through the frozen protocol surface (no
// fabricated local task state), riding the current model/effort choice
// (J-14's contextual entry — the choice is visible at compose time).

import { useEffect, useRef, useState } from "react";
import { useFlauzApp } from "../state/app";

export function NewTaskView({ onCancel }: { onCancel: () => void }) {
  const { t, startTask, state, setModelChoice } = useFlauzApp();
  const [objective, setObjective] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [modelEditorOpen, setModelEditorOpen] = useState(false);
  const [modelDraft, setModelDraft] = useState(state.modelChoice.model ?? "");
  const [effortDraft, setEffortDraft] = useState(state.modelChoice.effort ?? "");
  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const onStart = async () => {
    const text = objective.trim();
    if (text === "" || pending) {
      return;
    }
    setPending(true);
    setError(null);
    try {
      await startTask(text);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setPending(false);
    }
  };

  const applyModel = () => {
    setModelChoice(
      modelDraft.trim() === "" ? null : modelDraft.trim(),
      effortDraft.trim() === "" ? null : effortDraft.trim(),
    );
    setModelEditorOpen(false);
  };

  const modelSummary =
    state.modelChoice.model === null
      ? t("newtask.model.default")
      : t("newtask.model.summary", { model: state.modelChoice.model });

  return (
    <section className="flauz-card" aria-labelledby="newtask-title" data-testid="newtask-view">
      <h2 className="flauz-card-title" id="newtask-title">
        {t("newtask.title")}
      </h2>
      <p className="flauz-card-description">{t("newtask.description")}</p>
      <label className="flauz-label" htmlFor="newtask-objective">
        {t("newtask.objective.label")}
      </label>
      <textarea
        id="newtask-objective"
        ref={inputRef}
        className="flauz-input"
        placeholder={t("newtask.objective.placeholder")}
        value={objective}
        disabled={pending}
        onChange={(event) => setObjective(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
            void onStart();
          }
        }}
      />
      {error === null ? null : (
        <p className="flauz-error-text" role="alert" style={{ marginTop: 10 }}>
          {t("newtask.error", { reason: error })}
        </p>
      )}
      <div className="flauz-newtask-model" data-testid="newtask-model">
        <span className="flauz-hint" style={{ marginTop: 0 }} data-testid="newtask-model-summary">
          {modelSummary}
        </span>
        <button
          type="button"
          className="flauz-button"
          aria-expanded={modelEditorOpen}
          aria-controls="newtask-model-editor"
          onClick={() => setModelEditorOpen((open) => !open)}
          data-testid="newtask-model-change"
        >
          {t("newtask.model.change")}
        </button>
      </div>
      {modelEditorOpen ? (
        <div id="newtask-model-editor" className="flauz-newtask-model-editor">
          <label className="flauz-label" htmlFor="newtask-model-input">
            {t("newtask.model.label")}
          </label>
          <input
            id="newtask-model-input"
            className="flauz-input"
            placeholder={t("newtask.model.placeholder")}
            value={modelDraft}
            onChange={(event) => setModelDraft(event.target.value)}
            data-testid="newtask-model-input"
          />
          <label className="flauz-label" htmlFor="newtask-model-effort" style={{ marginTop: 8 }}>
            {t("newtask.model.effort")}
          </label>
          <input
            id="newtask-model-effort"
            className="flauz-input"
            value={effortDraft}
            onChange={(event) => setEffortDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                applyModel();
              }
            }}
            data-testid="newtask-model-effort"
          />
          <div className="flauz-panel-actions">
            <button type="button" className="flauz-button flauz-button-primary" onClick={applyModel} data-testid="newtask-model-apply">
              {t("model.next.apply")}
            </button>
            <button
              type="button"
              className="flauz-button"
              onClick={() => setModelEditorOpen(false)}
            >
              {t("newtask.model.hide")}
            </button>
          </div>
          <p className="flauz-hint">{t("model.next.description")}</p>
        </div>
      ) : null}
      <div style={{ display: "flex", gap: 10, marginTop: 14 }}>
        <button
          type="button"
          className="flauz-button flauz-button-primary"
          onClick={() => void onStart()}
          disabled={pending || objective.trim() === ""}
          data-testid="newtask-start"
        >
          {t("newtask.start")}
        </button>
        <button type="button" className="flauz-button" onClick={onCancel}>
          {t("newtask.cancel")}
        </button>
      </div>
    </section>
  );
}
