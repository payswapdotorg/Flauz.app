// The keyboard-shortcuts overlay (Ctrl+/), mirroring the desktop's
// shortcuts sheet with the web's keyboard map.

import { useEffect, useRef } from "react";
import { useFlauzApp } from "../state/app";
import { KEYBOARD_SHORTCUTS, displayChord, useFocusTrap } from "../keyboard/keyboard";

export function ShortcutsOverlay({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useFlauzApp();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const closeRef = useRef<HTMLButtonElement | null>(null);
  useFocusTrap(open, containerRef);

  useEffect(() => {
    if (open) {
      closeRef.current?.focus();
    }
  }, [open]);

  if (!open) {
    return null;
  }

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
        aria-label={t("shortcuts.title")}
        ref={containerRef}
        style={{ width: 430 }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            onClose();
          }
        }}
      >
        <div className="flauz-overlay-header">{t("shortcuts.title")}</div>
        <div className="flauz-overlay-body">
          {KEYBOARD_SHORTCUTS.map((shortcut) => (
            <div className="flauz-shortcut-row" key={shortcut.id}>
              <span>{t(shortcut.descriptionKey)}</span>
              <span className="flauz-shortcut-chord">{displayChord(shortcut.chord)}</span>
            </div>
          ))}
        </div>
        <div className="flauz-overlay-footer">
          <button type="button" className="flauz-button" ref={closeRef} onClick={onClose}>
            {t("shortcuts.close")}
          </button>
        </div>
      </div>
    </div>
  );
}
