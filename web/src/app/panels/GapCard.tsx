// The named-gap card (WEB-002): every capability the frozen protocol
// surface does not carry renders as a NAMED gap — what is missing, why,
// and how it unlocks (the J-04 law: never a silent no-op, never a
// fabrication). The availability itself is protocol-derived
// (state/capabilities.ts); this component only renders it.

import type { CapabilityStatus } from "../../state/capabilities";
import type { Translator } from "../../strings/en";

export interface GapCardProps {
  /** The protocol-derived availability check (never asserted by hand). */
  capability: CapabilityStatus;
  /** The strings: what is missing / why / how it unlocks. */
  whatKey: Parameters<Translator>[0];
  whyKey: Parameters<Translator>[0];
  unlockKey: Parameters<Translator>[0];
  t: Translator;
  testId?: string;
}

export function GapCard({ capability, whatKey, whyKey, unlockKey, t, testId }: GapCardProps) {
  if (capability.available) {
    // The protocol now carries this capability: the gap card stays
    // silent (the surface renders its real capability instead).
    return null;
  }
  return (
    <section
      className="flauz-gap"
      role="note"
      aria-label={t("gap.title")}
      data-testid={testId ?? `gap-${capability.surface}`}
      data-surface={capability.surface}
    >
      <p className="flauz-gap-title">
        <span className="flauz-gap-badge" aria-hidden="true">
          {t("gap.badge")}
        </span>
        {t("gap.title")}
      </p>
      <dl className="flauz-gap-list">
        <div className="flauz-gap-row">
          <dt>{t("gap.what")}</dt>
          <dd>{t(whatKey)}</dd>
        </div>
        <div className="flauz-gap-row">
          <dt>{t("gap.why")}</dt>
          <dd>{t(whyKey)}</dd>
        </div>
        <div className="flauz-gap-row">
          <dt>{t("gap.unlock")}</dt>
          <dd>{t(unlockKey)}</dd>
        </div>
      </dl>
    </section>
  );
}
