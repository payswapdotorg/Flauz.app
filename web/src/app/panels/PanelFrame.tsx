// The shared capability-panel frame (WEB-002): an aside with a labelled
// heading, a description, and focus handed to the heading when the panel
// opens (keyboard users land somewhere meaningful); the rail button's
// Escape handling restores focus (the 017 law, owned by SessionView).

import { useEffect, useRef, type ReactNode } from "react";

export interface PanelFrameProps {
  /** The accessible name of the panel (its aria-label). */
  label: string;
  /** The heading text. */
  title: string;
  /** One-line description of what the panel shows. */
  description?: string;
  testId: string;
  children: ReactNode;
}

export function PanelFrame({ label, title, description, testId, children }: PanelFrameProps) {
  const headingRef = useRef<HTMLHeadingElement | null>(null);

  useEffect(() => {
    // Hand focus to the heading so keyboard users land in the panel.
    headingRef.current?.focus();
  }, []);

  return (
    <aside className="flauz-side flauz-panel" aria-label={label} data-testid={testId}>
      <h2 className="flauz-panel-title" tabIndex={-1} ref={headingRef}>
        {title}
      </h2>
      {description === undefined ? null : <p className="flauz-panel-description">{description}</p>}
      {children}
    </aside>
  );
}
