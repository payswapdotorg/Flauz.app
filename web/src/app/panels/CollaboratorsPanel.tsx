// The collaboration/presence surface (WEB-002, scope 4): the flauz-collab
// contract's data shapes (membership roles, presence, permissions) arrive
// through the bridge only when the protocol carries them — it carries no
// membership or presence methods (the named gap). What IS real today:
// shared decisions surface as NAMED approval records (§6
// conflict-honesty — never auto-resolved, always attributed), and the F9
// privacy law is stated on the surface itself.

import { useFlauzApp } from "../../state/app";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { GapCard } from "./GapCard";
import { PanelFrame } from "./PanelFrame";

const COLLABORATION_CAPABILITY = protocolCapabilityStatuses().find(
  (status) => status.surface === "collaboration",
)!;

export function CollaboratorsPanel({ onFocusApprovals }: { onFocusApprovals: () => void }) {
  const { state, t } = useFlauzApp();
  const openApprovals = state.approvals.filter((card) => card.resolved === null).length;
  return (
    <CollaboratorsPanelView
      t={t}
      openApprovals={openApprovals}
      onFocusApprovals={onFocusApprovals}
      capability={COLLABORATION_CAPABILITY}
    />
  );
}

export function CollaboratorsPanelView({
  t,
  openApprovals,
  onFocusApprovals,
  capability,
}: {
  t: ReturnType<typeof useFlauzApp>["t"];
  openApprovals: number;
  onFocusApprovals: () => void;
  capability: (typeof COLLABORATION_CAPABILITY) | null;
}) {
  return (
    <PanelFrame
      label={t("rail.collaborators")}
      title={t("rail.collaborators")}
      description={t("collab.shared.description")}
      testId="collaborators-panel"
    >
      <section aria-label={t("collab.shared.title")}>
        <h3 className="flauz-panel-section-title">{t("collab.shared.title")}</h3>
        <p className="flauz-kv" data-testid="collab-approval-count">
          <span className="flauz-kv-key">{t("collab.shared.title")}</span>
          <span className="flauz-kv-value">
            {openApprovals === 0
              ? t("collab.shared.none")
              : t("collab.shared.count", { count: openApprovals })}
          </span>
        </p>
        {openApprovals === 0 ? null : (
          <div className="flauz-panel-actions">
            <button type="button" className="flauz-button" onClick={onFocusApprovals} data-testid="collab-show-approvals">
              {t("collab.shared.show")}
            </button>
          </div>
        )}
      </section>
      <p className="flauz-hint" data-testid="collab-privacy-note">
        {t("collab.privacy")}
      </p>
      {capability === null ? null : (
        <GapCard
          capability={capability}
          t={t}
          whatKey="collab.gap.what"
          whyKey="collab.gap.why"
          unlockKey="collab.gap.unlock"
        />
      )}
    </PanelFrame>
  );
}
