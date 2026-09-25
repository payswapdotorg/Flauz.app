// The workspace home: the primary discovery surface (WEB-001) — the
// sessions list, the first-run empty state, and the primary
// "Start a new task" entry.

import { useFlauzApp } from "../state/app";

function formatWhen(updatedAt: number): string {
  const delta = Date.now() - updatedAt * 1000;
  const minutes = Math.round(delta / 60_000);
  if (minutes < 1) {
    return "just now";
  }
  if (minutes < 60) {
    return `${minutes} min ago`;
  }
  const hours = Math.round(minutes / 60);
  if (hours < 24) {
    return `${hours} h ago`;
  }
  const days = Math.round(hours / 24);
  return `${days} d ago`;
}

export function HomeView({ onNewTask }: { onNewTask: () => void }) {
  const { state, t, openSession, refreshSessions } = useFlauzApp();
  const sessionsState = state.sessionsState;
  const sessions = state.sessions;

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
        <div>
          <h1 style={{ margin: 0, fontSize: 20 }}>{t("workspace.title")}</h1>
          <p style={{ margin: 0, color: "var(--flauz-text-muted)", fontSize: 13.5 }}>
            {t("workspace.sessions.description")}
          </p>
        </div>
        <div style={{ flex: 1 }} />
        <button
          type="button"
          className="flauz-button flauz-button-primary"
          onClick={onNewTask}
          data-testid="start-new-task"
        >
          {t("workspace.newTask.button")}
        </button>
      </div>

      {sessionsState === "loading" || sessionsState === "unloaded" ? (
        <p role="status" style={{ color: "var(--flauz-text-muted)" }}>
          {t("banner.reconnecting")}…
        </p>
      ) : null}

      {sessionsState === "error" ? (
        <div className="flauz-card" role="alert">
          <h2 className="flauz-card-title">{t("workspace.loadError", { reason: state.sessionsError ?? "" })}</h2>
          <button type="button" className="flauz-button" onClick={refreshSessions}>
            {t("workspace.retryLoad")}
          </button>
        </div>
      ) : null}

      {sessionsState === "ready" && sessions.length === 0 ? (
        <div className="flauz-card flauz-empty" data-testid="workspace-empty">
          <h2 className="flauz-empty-title">{t("workspace.empty.title")}</h2>
          <p className="flauz-empty-description">{t("workspace.empty.description")}</p>
          <button
            type="button"
            className="flauz-button flauz-button-primary"
            onClick={onNewTask}
            data-testid="empty-start-task"
          >
            {t("workspace.empty.action")}
          </button>
          <p className="flauz-hint" style={{ marginTop: 14 }}>
            {t("workspace.empty.secondary")}
          </p>
        </div>
      ) : null}

      {sessionsState === "ready" && sessions.length > 0 ? (
        <div className="flauz-sessions" data-testid="sessions-list">
          {sessions.map((thread) => (
            <button
              key={thread.id}
              type="button"
              className="flauz-session-row"
              onClick={() => openSession(thread.id)}
              data-testid="session-row"
              aria-label={`${t("workspace.session.open")}: ${thread.name ?? thread.preview}`}
            >
              <div style={{ minWidth: 0, flex: 1 }}>
                <div className="flauz-session-name">
                  {thread.name ?? (thread.preview === "" ? thread.id : thread.preview)}
                </div>
                <div className="flauz-session-preview">{thread.preview}</div>
              </div>
              <span className="flauz-session-when">
                {t("workspace.session.updated", { when: formatWhen(thread.recencyAt ?? thread.updatedAt) })}
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
