// The session view (WEB-001 foundation, WEB-002 capability rail): the
// live task surface — connect (thread/resume), the truthful
// live/connected state, the bounded activity timeline, the composer, the
// approvals queue (§6 named records, aria-live announced), the Stop-this-
// turn control (turn/interrupt, cancellation-honest), and the task
// control rail: Context, Environments, Model, Skills, Collaborators,
// Artifacts — every surface keyboard-complete (tab/Escape semantics,
// focus restoration per the 017 law).

import { useEffect, useRef, useState } from "react";
import { useFlauzApp, type ApprovalCard } from "../state/app";
import { EnvironmentsPanel } from "./panels/EnvironmentsPanel";
import { ModelPanel } from "./panels/ModelPanel";
import { SkillsPanel } from "./panels/SkillsPanel";
import { CollaboratorsPanel } from "./panels/CollaboratorsPanel";
import { ArtifactsPanel } from "./panels/ArtifactsPanel";

type PanelId = "context" | "environments" | "model" | "skills" | "collaborators" | "artifacts";

const PANEL_BUTTON_TESTID: Record<PanelId, string> = {
  context: "context-button",
  environments: "environments-button",
  model: "model-button",
  skills: "skills-button",
  collaborators: "collaborators-button",
  artifacts: "artifacts-button",
};

const PANEL_LABEL_KEY: Record<PanelId, "rail.context" | "rail.environments" | "rail.model" | "rail.skills" | "rail.collaborators" | "rail.artifacts"> = {
  context: "rail.context",
  environments: "rail.environments",
  model: "rail.model",
  skills: "rail.skills",
  collaborators: "rail.collaborators",
  artifacts: "rail.artifacts",
};

export function SessionView({ threadId }: { threadId: string }) {
  const { state, t, sendToSession, decideApproval, interruptTurn } = useFlauzApp();
  const [message, setMessage] = useState("");
  const [sendError, setSendError] = useState<string | null>(null);
  const [drawer, setDrawer] = useState<PanelId | "none">("none");
  const sessionState = state.currentSessionState;
  const live = state.connection.status === "connected" && state.connection.supervisorState === "connected";
  const railButtonRefs = useRef<Partial<Record<PanelId, HTMLButtonElement | null>>>({});
  const approvalsRef = useRef<HTMLElement | null>(null);

  // The palette/chord entry: open a capability panel by name.
  useEffect(() => {
    const onOpenPanel = (event: Event) => {
      const detail = (event as CustomEvent<{ panel: PanelId }>).detail;
      if (detail && detail.panel) {
        setDrawer(detail.panel);
      }
    };
    window.addEventListener("flauz:open-panel", onOpenPanel);
    return () => {
      window.removeEventListener("flauz:open-panel", onOpenPanel);
    };
  }, []);

  // The 017 law: Escape closes the open panel and restores focus to
  // its rail button.
  useEffect(() => {
    if (drawer === "none") {
      return;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        const panel = drawer;
        setDrawer("none");
        // Hand focus back to the rail button that opened the panel.
        window.setTimeout(() => railButtonRefs.current[panel]?.focus(), 0);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [drawer]);

  const openDrawer = (panel: PanelId) => {
    setDrawer((current) => (current === panel ? "none" : panel));
  };

  const focusApprovals = () => {
    approvalsRef.current?.scrollIntoView({ block: "center" });
    const firstButton = approvalsRef.current?.querySelector<HTMLElement>("button");
    firstButton?.focus();
  };

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

  const rail: PanelId[] = ["context", "environments", "model", "skills", "collaborators", "artifacts"];

  return (
    <div className="flauz-session-split">
      <div className="flauz-session-main">
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

        <div ref={(node) => {
          approvalsRef.current = node;
        }}>
          <ApprovalQueue approvals={state.approvals} onDecide={decideApproval} />
        </div>

        <div
          className="flauz-rail"
          role="group"
          aria-label={t("session.rail.label")}
          data-testid="task-rail"
        >
          {rail.map((panel) => (
            <button
              key={panel}
              type="button"
              className="flauz-button flauz-rail-button"
              aria-pressed={drawer === panel}
              ref={(node) => {
                railButtonRefs.current[panel] = node;
              }}
              onClick={() => openDrawer(panel)}
              data-testid={PANEL_BUTTON_TESTID[panel]}
            >
              {t(PANEL_LABEL_KEY[panel])}
            </button>
          ))}
          {state.turnRunning ? (
            <button
              type="button"
              className="flauz-button flauz-rail-button flauz-button-interrupt"
              onClick={interruptTurn}
              data-testid="interrupt-button"
            >
              {t("session.interrupt")}
            </button>
          ) : null}
        </div>

        <section aria-label={t("session.timeline")} className="flauz-timeline">
          {state.timeline.length === 0 ? (
            <p style={{ color: "var(--flauz-text-muted)" }}>{t("session.timeline.empty")}</p>
          ) : (
            state.timeline.map((entry) => (
              <article key={entry.id} className="flauz-timeline-entry" data-kind={entry.kind}>
                {entry.kind === "turn_started" || entry.kind === "turn_completed" || entry.kind === "turn_cancelled" ? (
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
      {drawer === "environments" ? <EnvironmentsPanel /> : null}
      {drawer === "model" ? <ModelPanel /> : null}
      {drawer === "skills" ? <SkillsPanel /> : null}
      {drawer === "collaborators" ? <CollaboratorsPanel onFocusApprovals={focusApprovals} /> : null}
      {drawer === "artifacts" ? <ArtifactsPanel /> : null}
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
  const decided = approvals.filter((card) => card.resolved !== null);
  if (open.length === 0 && decided.length === 0) {
    return null;
  }
  return (
    <section
      aria-label={t("approvals.title")}
      aria-live="polite"
      data-testid="approval-queue"
      className="flauz-approvals"
    >
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
      {decided.length === 0
        ? null
        : decided.map((card) => {
            // §6 conflict-honesty: a decided approval stays a NAMED
            // record — who decided (you) and what was decided, never a
            // silent disappearance.
            const outcome =
              card.resolved === "approved"
                ? t("approvals.approved")
                : card.resolved === "approved_for_session"
                  ? t("approvals.approvedForSession.record")
                  : t("approvals.declined");
            return (
              <div
                className="flauz-approval flauz-approval-decided"
                key={String(card.request.id)}
                data-testid="approval-record"
                data-outcome={card.resolved}
              >
                <p className="flauz-approval-title">
                  {t("approvals.decided.record")} — {outcome}
                </p>
              </div>
            );
          })}
    </section>
  );
}

function ContextDrawer() {
  const { state, t } = useFlauzApp();
  return (
    <aside className="flauz-side flauz-panel" aria-label={t("context.title")} data-testid="context-drawer">
      <h2 className="flauz-panel-title" tabIndex={-1}>
        {t("context.title")}
      </h2>
      <p className="flauz-panel-description">{t("context.description")}</p>
      <h3 className="flauz-panel-section-title">{t("context.objective")}</h3>
      <p style={{ marginTop: 0 }}>{state.currentSession?.preview ?? t("context.empty")}</p>
      <h3 className="flauz-panel-section-title" style={{ marginTop: 14 }}>{t("context.activity")}</h3>
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
