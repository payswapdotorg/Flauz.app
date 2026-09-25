// The command palette: the search/palette fallback discovery layer
// (never the sole mechanism — every palette command has a primary or
// contextual entry elsewhere in the shell).

import { useEffect, useMemo, useRef, useState } from "react";
import { useFlauzApp } from "../state/app";
import { useFocusTrap } from "../keyboard/keyboard";
import type { StringKey } from "../strings/en";

export interface PaletteCommand {
  id: string;
  labelKey: StringKey;
  run: () => void;
}

export function CommandPalette({
  open,
  onClose,
  extraCommands,
  onToggleTheme,
}: {
  open: boolean;
  onClose: () => void;
  extraCommands: PaletteCommand[];
  onToggleTheme: () => void;
}) {
  const { t, state, manualRetry, signOut } = useFlauzApp();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  useFocusTrap(open, containerRef);

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelected(0);
      inputRef.current?.focus();
    }
  }, [open]);

  const commands = useMemo<PaletteCommand[]>(() => {
    const base: PaletteCommand[] = [
      { id: "home", labelKey: "palette.command.home", run: () => (window.location.hash = "#/") },
      ...extraCommands,
      { id: "theme", labelKey: "palette.command.theme", run: onToggleTheme },
      {
        id: "reconnect",
        labelKey: "palette.command.reconnect",
        run: () => manualRetry(),
      },
      { id: "shortcuts", labelKey: "palette.command.shortcuts", run: () => window.dispatchEvent(new CustomEvent("flauz:shortcuts")) },
    ];
    if (state.connection.signedIn) {
      base.push({
        id: "signout",
        labelKey: "palette.command.signout",
        run: () => void signOut(),
      });
    }
    return base;
  }, [extraCommands, manualRetry, onToggleTheme, signOut, state.connection.signedIn]);

  const matches = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (normalized === "") {
      return commands;
    }
    return commands.filter((command) => t(command.labelKey).toLowerCase().includes(normalized));
  }, [commands, query, t]);

  if (!open) {
    return null;
  }

  const runSelected = () => {
    const command = matches[selected];
    if (command) {
      onClose();
      command.run();
    }
  };

  return (
    <div
      className="flauz-overlay-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        className="flauz-overlay"
        role="dialog"
        aria-modal="true"
        aria-label={t("palette.open")}
        ref={containerRef}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            onClose();
          } else if (event.key === "ArrowDown") {
            event.preventDefault();
            setSelected((current) => Math.min(current + 1, matches.length - 1));
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            setSelected((current) => Math.max(current - 1, 0));
          } else if (event.key === "Enter") {
            event.preventDefault();
            runSelected();
          }
        }}
      >
        <div className="flauz-overlay-header" style={{ display: "flex", gap: 10, alignItems: "center" }}>
          <input
            ref={inputRef}
            className="flauz-input"
            placeholder={t("palette.placeholder")}
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setSelected(0);
            }}
            aria-label={t("palette.placeholder")}
            data-testid="palette-input"
          />
        </div>
        <div className="flauz-overlay-body">
          {matches.length === 0 ? (
            <p style={{ color: "var(--flauz-text-muted)", margin: 0 }}>{t("palette.empty")}</p>
          ) : (
            <ul className="flauz-palette-list" role="listbox">
              {matches.map((command, index) => (
                <li key={command.id} role="option" aria-selected={index === selected}>
                  <button
                    type="button"
                    className="flauz-palette-item"
                    data-selected={index === selected}
                    onMouseEnter={() => setSelected(index)}
                    onClick={() => {
                      onClose();
                      command.run();
                    }}
                  >
                    {t(command.labelKey)}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="flauz-overlay-footer">
          <span>{t("palette.hint")}</span>
        </div>
      </div>
    </div>
  );
}
