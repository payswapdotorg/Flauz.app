// The skills surface (WEB-002, scope 3): the app-server's skill
// list/unlock methods are not in the frozen protocol snapshot (the named
// gap — J-04's discovery law names the unlock path), but the protocol
// DOES carry skill references in turn input (the generated UserInput
// `skill` variant). This surface exposes that real path: reference an
// installed skill by name and location for the next turn.

import { useState } from "react";
import { useFlauzApp } from "../../state/app";
import { protocolCapabilityStatuses } from "../../state/capabilities";
import { GapCard } from "./GapCard";
import { PanelFrame } from "./PanelFrame";

const SKILL_LISTING_CAPABILITY = protocolCapabilityStatuses().find(
  (status) => status.surface === "skill-listing",
)!;

export function SkillsPanel() {
  const { t, referenceSkill } = useFlauzApp();
  return (
    <SkillsPanelView
      t={t}
      onReferenceSkill={(name, path, note) => void referenceSkill(name, path, note)}
      capability={SKILL_LISTING_CAPABILITY}
    />
  );
}

export function SkillsPanelView({
  t,
  onReferenceSkill,
  capability,
}: {
  t: ReturnType<typeof useFlauzApp>["t"];
  onReferenceSkill: (name: string, path: string, note: string | null) => void;
  capability: (typeof SKILL_LISTING_CAPABILITY) | null;
}) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [note, setNote] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const onUse = async () => {
    const skillName = name.trim();
    const skillPath = path.trim();
    if (skillName === "" || skillPath === "" || pending) {
      return;
    }
    setPending(true);
    setError(null);
    try {
      await onReferenceSkill(skillName, skillPath, note.trim() === "" ? null : note.trim());
      setName("");
      setPath("");
      setNote("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(false);
    }
  };

  return (
    <PanelFrame
      label={t("rail.skills")}
      title={t("rail.skills")}
      description={t("skills.reference.description")}
      testId="skills-panel"
    >
      <section aria-label={t("skills.reference.title")}>
        <h3 className="flauz-panel-section-title">{t("skills.reference.title")}</h3>
        <label className="flauz-label" htmlFor="skill-name-input">
          {t("skills.reference.name")}
        </label>
        <input
          id="skill-name-input"
          className="flauz-input"
          placeholder={t("skills.reference.name.placeholder")}
          value={name}
          disabled={pending}
          onChange={(event) => setName(event.target.value)}
          data-testid="skill-name-input"
        />
        <label className="flauz-label" htmlFor="skill-path-input" style={{ marginTop: 10 }}>
          {t("skills.reference.path")}
        </label>
        <input
          id="skill-path-input"
          className="flauz-input"
          placeholder={t("skills.reference.path.placeholder")}
          value={path}
          disabled={pending}
          onChange={(event) => setPath(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              void onUse();
            }
          }}
          data-testid="skill-path-input"
        />
        <label className="flauz-label" htmlFor="skill-note-input" style={{ marginTop: 10 }}>
          {t("skills.reference.note")}
        </label>
        <input
          id="skill-note-input"
          className="flauz-input"
          placeholder={t("skills.reference.note.placeholder")}
          value={note}
          disabled={pending}
          onChange={(event) => setNote(event.target.value)}
          data-testid="skill-note-input"
        />
        <div className="flauz-panel-actions">
          <button
            type="button"
            className="flauz-button flauz-button-primary"
            onClick={() => void onUse()}
            disabled={pending || name.trim() === "" || path.trim() === ""}
            data-testid="skill-use-button"
          >
            {t("skills.reference.use")}
          </button>
        </div>
        <p className="flauz-hint" role="status" data-testid="skills-reference-status">
          {pending ? t("skills.reference.pending") : error === null ? t("skills.reference.description") : null}
        </p>
        {error === null ? null : (
          <p className="flauz-error-text" role="alert">
            {error}
          </p>
        )}
      </section>
      {capability === null ? null : (
        <GapCard
          capability={capability}
          t={t}
          whatKey="skills.gap.what"
          whyKey="skills.gap.why"
          unlockKey="skills.gap.unlock"
        />
      )}
    </PanelFrame>
  );
}
