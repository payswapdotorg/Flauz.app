// The new-task composer (J-01 web variant): describe the objective in
// plain language; the turn starts on the agent runtime through the
// frozen protocol surface (no fabricated local task state).

import { useEffect, useRef, useState } from "react";
import { useFlauzApp } from "../state/app";

export function NewTaskView({ onCancel }: { onCancel: () => void }) {
  const { t, startTask } = useFlauzApp();
  const [objective, setObjective] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
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
