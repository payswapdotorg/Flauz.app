// The session view: the live task surface — connect (thread/resume),
// the truthful live/connected state, the bounded activity timeline, the
// composer, the approvals queue, and the Context (J-02) and
// Environments (J-05) drawers with honest foundation-grade content.

import { useState } from "react";
import { useFlauzApp, type ApprovalCard } from "../state/app";

export function SessionView({ threadId }: { threadId: string }) {
  const { state, t, sendToSession, decideApproval } = useFlauzApp();
  const [message, setMessage] = useState("");
  const [sendError, setSendError] = useState<string | null>(null);
  const [drawer, setDrawer] = useState<"none" | "context" | "environments">("none");
  const sessionState = state.currentSessionState;
  const live = state.connection.status === "connected" && state.connection.supervisorState === "connected";

  const onSend = async () => {
    const text = message.trim();
    if (text === "") {
      return;
    }
    setSendError(null);
    try {
      await sendToSession(text);
      setMessage("");
    } catch (cause) {
      setSendError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  return (
    <div style={{ display: "flex", minHeight: 0, flex: 1 }}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <button
          type="button"
          className="flauz-button"
          style={{ marginBottom: 12 }}
          onClick={() => {
            window.location.hash = "#/";
          }}
        >
          {t("session.back")}
        </button>
        <h1 style={{ margin: "0 0 4px", fontSize: 20 }}>
          {state.currentSession?.name ?? state.currentSession?.preview ?? threadId}
        </h1>
        <p
          role="status"
          style={{
            margin: "0 0 12px",
            fontSize: 13.5,
            color: live ? "var(--flauz-ok)" : "var(--flauz-warning)",
          }}
          data-testid="session-connection"
        >
          {sessionState === "loading" || sessionState === "unloaded"
            ? t("session.connecting")
            : sessionState === "error"
              ? t("session.turn.failed", { reason: state.currentSessionError ?? "" })
              : live
                ? t("session.connected")
                : t("session.turn.running")}
        </p>

        <ApprovalQueue approvals={state.approvals} onDecide={decideApproval} />

        <div style={{ display: "flex", gap: 8, marginBottom: 10 }}>
          <button
            type="button"
            className="flauz-button"
            aria-pressed={drawer === "context"}
            onClick={() => setDrawer(drawer === "context" ? "none" : "context")}
            data-testid="context-button"
          >
            {t("context.button")}
          </button>
          <button
            type="button"
            className="flauz-button"
            aria-pressed={drawer === "environments"}
            onClick={() => setDrawer(drawer === "environments" ? "none" : "environments")}
            data-testid="environments-button"
          >
            {t("environments.button")}
          </button>
        </div>

        <section aria-label={t("session.timeline")} className="flauz-timeline">
          {state.timeline.length === 0 ? (
            <p style={{ color: "var(--flauz-text-muted)" }}>{t("session.timeline.empty")}</p>
          ) : (
            state.timeline.map((entry) => (
              <article key={entry.id} className="flauz-timeline-entry" data-kind={entry.kind}>
                {entry.kind === "turn_started" || entry.kind === "turn_completed" ? (
                  <span className="flauz-timeline-turn">{entry.text}</span>
                ) : (
                  entry.text
                )}
              </article>
            ))
          )}
        </section>

        {state.liveMessage === null ? null : (
          <p className="flauz-live" role="status" data-testid="live-message">
            {state.liveMessage}
          </p>
        )}

        <div style={{ marginTop: 14 }}>
          <label className="flauz-label" htmlFor="session-composer">
            {t("session.composer.label")}
          </label>
          <textarea
            id="session-composer"
            className="flauz-input"
            placeholder={t("session.composer.placeholder")}
            value={message}
            disabled={!live}
            onChange={(event) => setMessage(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
                void onSend();
              }
            }}
          />
          {sendError === null ? null : (
            <p className="flauz-error-text" role="alert">
              {sendError}
            </p>
          )}
          <div style={{ display: "flex", gap: 10, marginTop: 8, alignItems: "center" }}>
            <button
              type="button"
              className="flauz-button flauz-button-primary"
              onClick={() => void onSend()}
              disabled={!live || message.trim() === ""}
            >
              {t("session.composer.send")}
            </button>
            <span className="flauz-hint">{t("session.composer.hint")}</span>
          </div>
          <p className="flauz-hint" style={{ marginTop: 10 }}>
            {t("session.nextStep")}
          </p>
        </div>
      </div>
      {drawer === "context" ? <ContextDrawer /> : null}
      {drawer === "environments" ? <EnvironmentsDrawer /> : null}
    </div>
  );
}

function ApprovalQueue({
  approvals,
  onDecide,
}: {
  approvals: ApprovalCard[];
  onDecide: (id: number | string, method: string, decision: "approve" | "approve_for_session" | "decline") => void;
}) {
  const { t } = useFlauzApp();
  const open = approvals.filter((card) => card.resolved === null);
  if (open.length === 0) {
    return null;
  }
  return (
    <section aria-label={t("approvals.title")} data-testid="approval-queue">
      {open.map((card) => {
        const params = (card.request.params ?? {}) as {
          command?: string;
          changeSummary?: string;
          details?: string[];
        };
        const title =
          card.request.method === "item/commandExecution/requestApproval"
            ? t("approvals.command")
            : card.request.method === "item/fileChange/requestApproval"
              ? t("approvals.fileChange")
              : t("approvals.permissions");
        const body =
          card.request.method === "item/commandExecution/requestApproval"
            ? (params.command ?? "")
            : card.request.method === "item/fileChange/requestApproval"
              ? (params.changeSummary ?? "")
              : (params.details ?? []).join("\n");
        return (
          <div className="flauz-approval" key={String(card.request.id)} data-testid="approval-card">
            <p className="flauz-approval-title">
              {t("approvals.title")} — {title}
            </p>
            <pre className="flauz-approval-command">{body}</pre>
            <div className="flauz-approval-actions">
              {card.request.method === "item/permissions/requestApproval" ? (
                <button
                  type="button"
                  className="flauz-button"
                  onClick={() => onDecide(card.request.id, card.request.method, "decline")}
                >
                  {t("approvals.decline")}
                </button>
              ) : (
                <>
                  <button
                    type="button"
                    className="flauz-button flauz-button-primary"
                    onClick={() => onDecide(card.request.id, card.request.method, "approve")}
                    data-testid="approval-approve"
                  >
                    {t("approvals.approve")}
                  </button>
                  <button
                    type="button"
                    className="flauz-button"
                    onClick={() => onDecide(card.request.id, card.request.method, "approve_for_session")}
                  >
                    {t("approvals.approveForSession")}
                  </button>
                  <button
                    type="button"
                    className="flauz-button"
                    onClick={() => onDecide(card.request.id, card.request.method, "decline")}
                    data-testid="approval-decline"
                  >
                    {t("approvals.decline")}
                  </button>
                </>
              )}
            </div>
            {card.request.method === "item/permissions/requestApproval" ? (
              <p className="flauz-hint" style={{ marginTop: 8 }}>
                {t("approvals.permissions.declined")}
              </p>
            ) : null}
          </div>
        );
      })}
    </section>
  );
}

function ContextDrawer() {
  const { state, t } = useFlauzApp();
  return (
    <aside className="flauz-side" aria-label={t("context.title")} data-testid="context-drawer">
      <h2 style={{ margin: "0 0 4px", fontSize: 16 }}>{t("context.title")}</h2>
      <p style={{ color: "var(--flauz-text-muted)", fontSize: 13 }}>{t("context.description")}</p>
      <h3 style={{ fontSize: 13, marginBottom: 4 }}>{t("context.objective")}</h3>
      <p style={{ marginTop: 0 }}>{state.currentSession?.preview ?? t("context.empty")}</p>
      <h3 style={{ fontSize: 13, margin: "14px 0 4px" }}>{t("context.activity")}</h3>
      {state.timeline.length === 0 ? (
        <p style={{ marginTop: 0, color: "var(--flauz-text-muted)" }}>{t("context.empty")}</p>
      ) : (
        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13.5 }}>
          {state.timeline.slice(-8).map((entry) => (
            <li key={entry.id} style={{ marginBottom: 4 }}>
              {entry.text.length > 90 ? `${entry.text.slice(0, 90)}…` : entry.text}
            </li>
          ))}
        </ul>
      )}
      <p className="flauz-hint" style={{ marginTop: 14 }}>
        {t("context.nextStep")}
      </p>
    </aside>
  );
}

function EnvironmentsDrawer() {
  const { t } = useFlauzApp();
  return (
    <aside className="flauz-side" aria-label={t("environments.title")} data-testid="environments-drawer">
      <h2 style={{ margin: "0 0 4px", fontSize: 16 }}>{t("environments.title")}</h2>
      <div className="flauz-card" style={{ background: "var(--flauz-surface-muted)" }}>
        <p className="flauz-empty-title" style={{ fontSize: 15 }}>
          {t("environments.empty.title")}
        </p>
        <p className="flauz-empty-description" style={{ fontSize: 13 }}>
          {t("environments.empty.description")}
        </p>
        <p className="flauz-hint">{t("environments.nextStep")}</p>
      </div>
    </aside>
  );
}
