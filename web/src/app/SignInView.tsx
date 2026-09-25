// The sign-in surface: authentication flows through the app-server's
// own auth API over the transparent bridge (credentials never touch the
// gateway, logs, or storage).

import { useState } from "react";
import { useFlauzApp } from "../state/app";

export function SignInView() {
  const { t, signInWithChatGpt, signInWithApiKey, state } = useFlauzApp();
  const [apiKey, setApiKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<"chatgpt" | "apikey" | null>(null);
  const runtimeAttached = state.connection.supervisorState === "connected";

  const onChatGpt = async () => {
    setError(null);
    setPending("chatgpt");
    try {
      await signInWithChatGpt();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(null);
    }
  };

  const onApiKey = async () => {
    if (apiKey.trim() === "") {
      return;
    }
    setError(null);
    setPending("apikey");
    try {
      await signInWithApiKey(apiKey.trim());
      setApiKey("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(null);
    }
  };

  return (
    <section className="flauz-card" aria-labelledby="signin-title" data-testid="signin-view">
      <h2 className="flauz-card-title" id="signin-title">
        {t("signin.title")}
      </h2>
      <p className="flauz-card-description">{t("signin.description")}</p>
      {!runtimeAttached ? <p className="flauz-hint">{t("signin.waitingRuntime")}</p> : null}
      <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div>
          <button
            type="button"
            className="flauz-button flauz-button-primary"
            onClick={() => void onChatGpt()}
            disabled={!runtimeAttached || pending !== null}
            data-testid="signin-chatgpt"
          >
            {t("signin.chatgpt.button")}
          </button>
          {pending === "chatgpt" ? (
            <p className="flauz-hint" role="status">
              {t("signin.chatgpt.pending")}
            </p>
          ) : null}
        </div>
        <div>
          <label className="flauz-label" htmlFor="signin-apikey">
            {t("signin.apikey.label")}
          </label>
          <input
            id="signin-apikey"
            className="flauz-input"
            type="password"
            autoComplete="off"
            value={apiKey}
            disabled={!runtimeAttached || pending !== null}
            onChange={(event) => setApiKey(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                void onApiKey();
              }
            }}
          />
          <p className="flauz-hint">{t("signin.apikey.hint")}</p>
          <button
            type="button"
            className="flauz-button"
            onClick={() => void onApiKey()}
            disabled={!runtimeAttached || pending !== null || apiKey.trim() === ""}
            style={{ marginTop: 8 }}
            data-testid="signin-apikey"
          >
            {t("signin.apikey.button")}
          </button>
        </div>
        {error === null ? null : (
          <p className="flauz-error-text" role="alert" data-testid="signin-error">
            {t("signin.error", { reason: error })}
          </p>
        )}
      </div>
    </section>
  );
}
