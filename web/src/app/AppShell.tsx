// The app shell (WEB-001 foundation, WEB-002 capability wiring): layout,
// navigation, theming, keyboard map, routing (hash router), overlays,
// the always-truthful connection banner, the notice strip, and the
// capability-surface palette fallback (never the sole mechanism).

import { useCallback, useEffect, useState } from "react";
import { useFlauzApp } from "../state/app";
import { useTheme } from "../theme/theme";
import { useGlobalKeys } from "../keyboard/keyboard";
import { ConnectionBanner } from "./ConnectionBanner";
import { HomeView } from "./HomeView";
import { NewTaskView } from "./NewTaskView";
import { SessionView } from "./SessionView";
import { SignInView } from "./SignInView";
import { CommandPalette, type PaletteCommand } from "./CommandPalette";
import { ShortcutsOverlay } from "./ShortcutsOverlay";
import type { StringKey } from "../strings/en";

type Route = { name: "home" } | { name: "newtask" } | { name: "session"; threadId: string };

function parseHash(hash: string): Route {
  const path = hash.replace(/^#\/?/, "");
  if (path === "new") {
    return { name: "newtask" };
  }
  const session = /^session\/([^/]+)$/.exec(path);
  if (session?.[1]) {
    return { name: "session", threadId: decodeURIComponent(session[1]) };
  }
  return { name: "home" };
}

/** The capability panels in rail order (the Ctrl+Alt+Shift+1..6 chords). */
const PANEL_CHORD_ORDER = [
  "context",
  "environments",
  "model",
  "skills",
  "collaborators",
  "artifacts",
] as const;

type ChordPanel = (typeof PANEL_CHORD_ORDER)[number];

export function AppShell() {
  const { state, t, setNotice } = useFlauzApp();
  const { theme, toggleTheme } = useTheme();
  const [route, setRoute] = useState<Route>(() =>
    typeof window === "undefined" ? { name: "home" } : parseHash(window.location.hash),
  );
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);

  useEffect(() => {
    const onHashChange = () => {
      setRoute(parseHash(window.location.hash));
    };
    window.addEventListener("hashchange", onHashChange);
    return () => {
      window.removeEventListener("hashchange", onHashChange);
    };
  }, []);

  useEffect(() => {
    const onShortcuts = () => setShortcutsOpen(true);
    window.addEventListener("flauz:shortcuts", onShortcuts);
    return () => {
      window.removeEventListener("flauz:shortcuts", onShortcuts);
    };
  }, []);

  const openSession = useFlauzApp().openSession;
  useEffect(() => {
    if (route.name === "session" && state.currentSessionId !== route.threadId) {
      openSession(route.threadId);
    }
  }, [route, state.currentSessionId, openSession]);

  /** Opens a capability panel on the current task (the palette fallback). */
  const openCapabilityPanel = useCallback(
    (panel: ChordPanel, surfaceLabelKey: StringKey) => {
      if (state.currentSessionId !== null) {
        window.location.hash = `#/session/${encodeURIComponent(state.currentSessionId)}`;
        // The panel event must land AFTER the session view mounts (the
        // hash change is asynchronous); a short delay guarantees the
        // order from any route.
        window.setTimeout(() => {
          window.dispatchEvent(new CustomEvent("flauz:open-panel", { detail: { panel } }));
        }, 60);
      } else {
        // No task is open: the surface is task-scoped, so the palette
        // names where to go instead of silently doing nothing.
        setNotice(t("palette.noSession", { surface: t(surfaceLabelKey).toLowerCase() }));
      }
    },
    [state.currentSessionId, setNotice, t],
  );

  const onGlobalKey = useCallback(
    (event: KeyboardEvent) => {
      const isPalette = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k";
      const isShortcuts = (event.ctrlKey || event.metaKey) && event.key === "/";
      const isPanelChord =
        event.ctrlKey && event.altKey && event.shiftKey && /^[1-6]$/.test(event.key);
      if (isPalette) {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      } else if (isShortcuts) {
        event.preventDefault();
        setShortcutsOpen((open) => !open);
      } else if (isPanelChord && state.currentSessionId !== null) {
        event.preventDefault();
        const panel = PANEL_CHORD_ORDER[Number(event.key) - 1];
        if (panel !== undefined) {
          window.dispatchEvent(new CustomEvent("flauz:open-panel", { detail: { panel } }));
        }
      } else if (event.key === "Escape") {
        setPaletteOpen(false);
        setShortcutsOpen(false);
      }
    },
    [state.currentSessionId],
  );
  useGlobalKeys(onGlobalKey);

  const navigateToNewTask = useCallback(() => {
    window.location.hash = "#/new";
  }, []);

  const signedIn = state.connection.signedIn;
  const showSignIn = !signedIn && state.connection.status !== "failed";

  const paletteCommands: PaletteCommand[] = [
    { id: "newtask", labelKey: "palette.command.newtask", run: navigateToNewTask },
    {
      id: "environments",
      labelKey: "palette.command.environments",
      run: () => openCapabilityPanel("environments", "rail.environments"),
    },
    {
      id: "model",
      labelKey: "palette.command.model",
      run: () => openCapabilityPanel("model", "rail.model"),
    },
    {
      id: "skills",
      labelKey: "palette.command.skills",
      run: () => openCapabilityPanel("skills", "rail.skills"),
    },
    {
      id: "collaborators",
      labelKey: "palette.command.collaborators",
      run: () => openCapabilityPanel("collaborators", "rail.collaborators"),
    },
    {
      id: "artifacts",
      labelKey: "palette.command.artifacts",
      run: () => openCapabilityPanel("artifacts", "rail.artifacts"),
    },
  ];

  return (
    <div className="flauz-shell">
      <header className="flauz-header">
        <span className="flauz-header-title">{t("app.title")}</span>
        <span className="flauz-header-subtitle">{t("app.subtitle")}</span>
        <span className="flauz-header-spacer" />
        <button
          type="button"
          className="flauz-button"
          onClick={() => setPaletteOpen(true)}
          aria-label={t("palette.open")}
          data-testid="palette-button"
        >
          {t("palette.open")} <span className="flauz-palette-chord">Ctrl K</span>
        </button>
        <button
          type="button"
          className="flauz-button"
          onClick={toggleTheme}
          aria-label={t("theme.toggle")}
          data-testid="theme-button"
        >
          {theme === "light" ? "🌙" : "☀️"}
        </button>
      </header>
      <ConnectionBanner />
      {state.notice === null ? null : (
        <div className="flauz-notice" role="status" data-testid="notice-strip">
          <span>{state.notice}</span>
          <span className="flauz-header-spacer" />
          <button type="button" className="flauz-button" onClick={() => setNotice(null)}>
            {t("notice.dismiss")}
          </button>
        </div>
      )}
      <div className="flauz-main">
        <nav className="flauz-nav" aria-label={t("workspace.title")}>
          <button
            type="button"
            className="flauz-button"
            style={{ textAlign: "left" }}
            onClick={() => {
              window.location.hash = "#/";
            }}
          >
            {t("workspace.title")}
          </button>
          <button
            type="button"
            className="flauz-button"
            style={{ textAlign: "left" }}
            onClick={navigateToNewTask}
          >
            {t("workspace.newTask.button")}
          </button>
        </nav>
        <main className="flauz-content">
          {state.connection.status === "failed" && !signedIn ? (
            <section className="flauz-card" role="alert">
              <h2 className="flauz-card-title">{t("banner.failed")}</h2>
              <p>{state.connection.reason}</p>
            </section>
          ) : showSignIn ? (
            <SignInView />
          ) : route.name === "newtask" ? (
            <NewTaskView onCancel={() => (window.location.hash = "#/")} />
          ) : route.name === "session" ? (
            <SessionView threadId={route.threadId} />
          ) : (
            <HomeView onNewTask={navigateToNewTask} />
          )}
        </main>
      </div>
      <footer className="flauz-footer">
        <span>{t("footer.product")}</span>
        <span>{t("footer.gatewayLocal")}</span>
      </footer>
      <CommandPalette
        open={paletteOpen}
        onClose={() => setPaletteOpen(false)}
        extraCommands={paletteCommands}
        onToggleTheme={toggleTheme}
      />
      <ShortcutsOverlay open={shortcutsOpen} onClose={() => setShortcutsOpen(false)} />
    </div>
  );
}
