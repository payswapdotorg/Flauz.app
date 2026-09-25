// The connection-state banner: ALWAYS visible, ALWAYS truthful (the
// contextual discovery surface for connection state — addendum §5).

import { useEffect, useState } from "react";
import { renderedState } from "../state/connection";
import { useFlauzApp } from "../state/app";
import type { StringKey } from "../strings/en";

const BANNER_LABEL: Record<string, StringKey> = {
  connected: "banner.connected",
  authenticating: "banner.authenticating",
  reconnecting: "banner.reconnecting",
  failed: "banner.failed",
};

export function ConnectionBanner() {
  const { state, t, manualRetry } = useFlauzApp();
  const connection = state.connection;
  const rendered = renderedState(connection);
  const label = t(BANNER_LABEL[rendered]);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    if (connection.nextRetryInMs === null) {
      return;
    }
    const timer = setTimeout(() => setTick((value) => value + 1), 250);
    return () => clearTimeout(timer);
  }, [connection.nextRetryInMs, tick]);

  const retryIn = connection.nextRetryInMs === null ? null : Math.max(0, Math.ceil(connection.nextRetryInMs / 1000));

  return (
    <div
      className={`flauz-banner flauz-banner-${rendered}`}
      role="status"
      aria-live="polite"
      data-testid="connection-banner"
      data-state={rendered}
    >
      <span className="flauz-banner-dot" aria-hidden="true" />
      <strong>{label}</strong>
      {connection.reason === null ? null : (
        <span className="flauz-banner-reason" data-testid="connection-reason">
          {connection.reason}
        </span>
      )}
      {retryIn !== null && rendered === "reconnecting" ? (
        <span className="flauz-banner-reason">{t("banner.nextRetry", { seconds: retryIn })}</span>
      ) : null}
      <span className="flauz-banner-spacer" />
      {connection.sessionId === null ? null : (
        <span className="flauz-banner-session" title={t("banner.session", { id: connection.sessionId })}>
          {connection.sessionId.slice(0, 15)}…
        </span>
      )}
      {rendered === "failed" ? (
        <button type="button" className="flauz-button" onClick={manualRetry}>
          {t("banner.retryNow")}
        </button>
      ) : null}
    </div>
  );
}
