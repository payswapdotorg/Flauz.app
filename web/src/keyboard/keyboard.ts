// The keyboard map and focus management: global chords, overlay focus
// trap + restoration (the F1 017 focus law), and roving list focus.

import { useEffect, type RefObject } from "react";

export interface KeyboardShortcut {
  id: string;
  chord: string;
  descriptionKey: "shortcuts.palette" | "shortcuts.shortcuts" | "shortcuts.escape";
}

export const KEYBOARD_SHORTCUTS: KeyboardShortcut[] = [
  { id: "palette", chord: "Ctrl+K", descriptionKey: "shortcuts.palette" },
  { id: "shortcuts", chord: "Ctrl+/", descriptionKey: "shortcuts.shortcuts" },
  { id: "escape", chord: "Esc", descriptionKey: "shortcuts.escape" },
];

/** Installs a global keydown handler (skips when the user is typing in
 * an input, except for Escape and modifier chords). */
export function useGlobalKeys(handler: (event: KeyboardEvent) => void) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target;
      const typing =
        target instanceof HTMLElement &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
      const modifierChord = event.ctrlKey || event.metaKey;
      if (typing && !modifierChord && event.key !== "Escape") {
        return;
      }
      handler(event);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [handler]);
}

/** Traps Tab focus inside `containerRef` while active, restores focus to
 * the previously focused element on release (the 017 law). */
export function useFocusTrap(active: boolean, containerRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    if (!active) {
      return;
    }
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const container = containerRef.current;
    if (container !== null) {
      const focusables = focusableElements(container);
      if (focusables.length > 0) {
        focusables[0]?.focus();
      }
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Tab") {
        return;
      }
      const current = containerRef.current;
      if (current === null) {
        return;
      }
      const focusables = focusableElements(current);
      if (focusables.length === 0) {
        return;
      }
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (first === undefined || last === undefined) {
        return;
      }
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => {
      document.removeEventListener("keydown", onKeyDown, true);
      previous?.focus();
    };
  }, [active, containerRef]);
}

function focusableElements(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      "button:not([disabled]), [href], input:not([disabled]), select, textarea, [tabindex]:not([tabindex='-1'])",
    ),
  ).filter((element) => element.offsetParent !== null || element === document.activeElement);
}

/** Formats a key chord for display (platform-aware). */
export function displayChord(chord: string): string {
  if (typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform)) {
    return chord.replace("Ctrl+", "⌘");
  }
  return chord;
}
