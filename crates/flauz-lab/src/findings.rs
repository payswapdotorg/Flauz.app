//! The named-findings registry (LAB-001, Wave-4 kernel addendum §4): the
//! F1 accessibility lessons as DATA — id, surface, description,
//! regression-guard status.
//!
//! The registry is what every future lab run references: a probe result
//! links findings by [`FindingId`], never by prose, so a divergence that
//! reproduces a known lesson is NAMED in the evidence rather than
//! re-narrated. [`builtin_findings_registry`] carries the five F1
//! lessons the work order names — PTY focus transfer, bracket swap,
//! modal traps, first-run keyboard swallowing, the N6 shifted-symbol
//! family — each with its honest guard status. The registry is
//! extendable: the operator adds entries as new findings land; records
//! reference them by id.
//!
//! The registry data is honest about what is guarded and what is merely
//! carried: [`GuardStatus::Guarded`] entries name the regression guard
//! that pins them (a seam test or a lab scene), [`GuardStatus::Unguarded`]
//! entries are carried forward with no guard yet.

use serde::{Deserialize, Serialize};

use crate::refs::{FindingId, SurfaceId};
use crate::{
    LabError, LabVersion, MAX_FINDINGS, ensure_explanation, ensure_list_bound, ensure_sorted_unique,
};

/// The regression-guard status of a named finding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum GuardStatus {
    /// The finding is carried forward with NO regression guard yet —
    /// honest: nothing fails loudly if it reappears.
    Unguarded,
    /// A named regression guard pins the finding — a seam test or an
    /// institutionalized lab scene that fails loudly on reintroduction.
    Guarded {
        /// The guard's named anchor (for example the lab scene or seam
        /// test that pins it).
        guard: String,
    },
}

impl GuardStatus {
    /// Validates the status's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when a guard anchor is empty or oversized.
    pub fn validate(&self) -> Result<(), LabError> {
        if let Self::Guarded { guard } = self {
            ensure_explanation("regression guard anchor", guard)?;
        }
        Ok(())
    }

    /// Whether a regression guard pins the finding.
    #[must_use]
    pub const fn guarded(&self) -> bool {
        matches!(self, Self::Guarded { .. })
    }
}

/// One named finding: an accessibility or behavior lesson, as data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingEntry {
    /// The finding's stable identifier (for example `pty-focus-transfer`)
    /// — the id records reference, never prose.
    pub id: FindingId,
    /// The UI surface the finding concerns (for example
    /// `terminal-dock`).
    pub surface: SurfaceId,
    /// The honest, bounded description of the lesson.
    pub description: String,
    /// The regression-guard status.
    pub guard: GuardStatus,
}

impl FindingEntry {
    /// Builds a finding entry, validating every bound.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when any field is empty, oversized or outside
    /// its grammar.
    pub fn new(
        id: &str,
        surface: &str,
        description: &str,
        guard: GuardStatus,
    ) -> Result<Self, LabError> {
        let id = FindingId::parse(id)?;
        let surface = SurfaceId::parse(surface)?;
        ensure_explanation("finding description", description)?;
        guard.validate()?;
        Ok(Self {
            id,
            surface,
            description: description.to_owned(),
            guard,
        })
    }

    /// Validates the entry's bounds.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the entry fails validation.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(
            self.id.as_str(),
            self.surface.as_str(),
            &self.description,
            self.guard.clone(),
        )?;
        Ok(())
    }
}

/// The named-findings registry: the findings an operator's lab carries,
/// sorted by id. Records reference findings by id; resolving a reference
/// against the registry is the caller's check (the conformance tests
/// prove every committed evidence link resolves).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingsRegistry {
    /// Contract schema version (`"v": 1`).
    pub v: LabVersion,
    /// The registry's entries (sorted by id, unique).
    pub findings: Vec<FindingEntry>,
}

impl FindingsRegistry {
    /// Builds a registry from entries, canonically ordering them by id.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when an entry fails validation, the list is
    /// oversized, or ids repeat.
    pub fn new(mut findings: Vec<FindingEntry>) -> Result<Self, LabError> {
        ensure_list_bound("findings", findings.len(), MAX_FINDINGS)?;
        findings.sort_by(|left, right| left.id.cmp(&right.id));
        let ids: Vec<&FindingId> = findings.iter().map(|finding| &finding.id).collect();
        ensure_sorted_unique("finding ids", &ids)?;
        for finding in &findings {
            finding.validate()?;
        }
        Ok(Self {
            v: LabVersion,
            findings,
        })
    }

    /// Validates the registry against the canonical rules.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the registry fails validation.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::new(self.findings.clone())?;
        Ok(())
    }

    /// Resolves a finding reference by id — the check every record's
    /// finding links must pass.
    #[must_use]
    pub fn get(&self, id: &FindingId) -> Option<&FindingEntry> {
        self.findings.iter().find(|finding| &finding.id == id)
    }

    /// Whether every finding reference in `links` resolves in this
    /// registry.
    #[must_use]
    pub fn resolves_all(&self, links: &[FindingId]) -> bool {
        links.iter().all(|link| self.get(link).is_some())
    }
}

/// The builtin registry: the F1 accessibility lessons as data (addendum
/// §4) — the five findings the work order names, each with its honest
/// guard status. Extend by building a new [`FindingsRegistry`] with added
/// entries; records keep referencing findings by id.
///
/// # Errors
///
/// Returns [`LabError`] if any builtin entry fails canonical validation
/// (a bug in the registry, not in caller data).
pub fn builtin_findings_registry() -> Result<FindingsRegistry, LabError> {
    FindingsRegistry::new(vec![
        FindingEntry::new(
            "pty-focus-transfer",
            "terminal-dock",
            "Opening the terminal dock starts a live PTY, but keyboard focus does not transfer \
             to it on open; typed input keeps landing on the previously focused surface (F1 \
             a11y N5, carried to the F2 a11y binding items).",
            GuardStatus::Unguarded,
        )?,
        FindingEntry::new(
            "bracket-swap-chords",
            "chat-list",
            "The previous/next chat chords (Ctrl+Shift+]/[) produced no selection move in the \
             lab configuration while Ctrl+PageDown moved it positively; chord families must \
             be re-evidenced positively per configuration (F1 a11y N6).",
            GuardStatus::Unguarded,
        )?,
        FindingEntry::new(
            "modal-focus-traps",
            "confirmation-modals",
            "Destructive confirmation modals must confine Tab/Shift+Tab focus to their actions \
             and close on scoped Escape, returning focus to the surface that opened them (the \
             d19 trap-ladder contract).",
            GuardStatus::Guarded {
                guard: "the d19 trap-ladder lab evidence and the tab/shift-tab KeyBindings"
                    .to_owned(),
            },
        )?,
        FindingEntry::new(
            "first-run-keyboard-swallowing",
            "first-run-promo-modal",
            "A fresh-profile promo modal swallows all keyboard input until dismissed; Escape \
             is the verified dismissal, so first-run scenes carry a defensive \
             capture-then-Escape block that is a no-op when the modal is absent (the rc.14 \
             lesson).",
            GuardStatus::Unguarded,
        )?,
        FindingEntry::new(
            "shifted-symbol-chord-companions",
            "keybindings",
            "Digit-family chords carry shifted-symbol companions (for example alt+^ beside \
             Ctrl+Alt+Shift+6/7) that must be exercised alongside the primary chord: a binding \
             whose listener answers only one form dispatches the other into the void (the N6 \
             family + the d25 Gate-B missing-listener lesson).",
            GuardStatus::Guarded {
                guard: "the d24/d25 gate scenes exercise the companion form on every digit \
                        chord, and the seam tests pin the listener form, not just the binding"
                    .to_owned(),
            },
        )?,
    ])
}

/// The finding ids of the builtin registry, in canonical order — the
/// frozen vocabulary conformance tests pin.
pub const BUILTIN_FINDING_IDS: [&str; 5] = [
    "bracket-swap-chords",
    "first-run-keyboard-swallowing",
    "modal-focus-traps",
    "pty-focus-transfer",
    "shifted-symbol-chord-companions",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MAX_EXPLANATION_BYTES, MAX_NAME_BYTES, ensure_name, ensure_non_empty, ensure_str_bound,
    };

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn the_builtin_registry_carries_the_f1_lessons_as_data() {
        let registry = ok(builtin_findings_registry());
        ok(registry.validate());
        assert_eq!(
            registry
                .findings
                .iter()
                .map(|finding| finding.id.as_str().to_owned())
                .collect::<Vec<String>>(),
            BUILTIN_FINDING_IDS,
            "the five F1 lessons, canonically ordered"
        );
        // Every reference by id resolves; unknown ids do not.
        for id in BUILTIN_FINDING_IDS {
            let finding = registry
                .get(&ok(FindingId::parse(id)))
                .unwrap_or_else(|| panic!("the builtin finding {id} must resolve by id"));
            ok(finding.validate());
            assert!(!finding.description.is_empty());
        }
        assert!(
            registry
                .get(&ok(FindingId::parse("no-such-finding")))
                .is_none(),
            "unknown ids do not resolve"
        );
        // The guard statuses are honest: the trap + companion lessons are
        // guarded; PTY focus transfer, bracket swap and the first-run
        // swallow are carried unguarded.
        let guarded: Vec<&str> = registry
            .findings
            .iter()
            .filter(|finding| finding.guard.guarded())
            .map(|finding| finding.id.as_str())
            .collect();
        assert_eq!(
            guarded,
            vec!["modal-focus-traps", "shifted-symbol-chord-companions"]
        );
    }

    #[test]
    fn guard_statuses_serialize_internally_tagged() {
        let unguarded = GuardStatus::Unguarded;
        assert_eq!(
            ok(serde_json::to_string(&unguarded)),
            "{\"kind\":\"unguarded\"}"
        );
        let guarded = GuardStatus::Guarded {
            guard: "the d19 trap-ladder lab evidence".to_owned(),
        };
        assert_eq!(
            ok(serde_json::to_string(&guarded)),
            "{\"kind\":\"guarded\",\"guard\":\"the d19 trap-ladder lab evidence\"}"
        );
        let reloaded: GuardStatus = ok(serde_json::from_str(&ok(serde_json::to_string(&guarded))));
        assert_eq!(reloaded, guarded);
        assert!(guarded.guarded());
        assert!(!unguarded.guarded());
        assert!(
            GuardStatus::Guarded {
                guard: String::new()
            }
            .validate()
            .is_err()
        );
        assert!(
            GuardStatus::Guarded {
                guard: "g".repeat(MAX_EXPLANATION_BYTES + 1)
            }
            .validate()
            .is_err()
        );
        assert!(
            serde_json::from_str::<GuardStatus>("{\"kind\":\"maybe\"}").is_err(),
            "unknown guard kinds are rejected"
        );
    }

    #[test]
    fn registries_round_trip_and_reject_inconsistent_shapes() {
        let registry = ok(builtin_findings_registry());
        let serialized = ok(serde_json::to_string_pretty(&registry));
        let reloaded: FindingsRegistry = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, registry);

        // Duplicate ids and oversized lists are rejected; entries sort by
        // id on construction.
        let duplicated = FindingsRegistry::new(vec![
            ok(FindingEntry::new(
                "zeta-finding",
                "task-surface",
                "A finding.",
                GuardStatus::Unguarded,
            )),
            ok(FindingEntry::new(
                "zeta-finding",
                "task-surface",
                "The same finding again.",
                GuardStatus::Unguarded,
            )),
        ]);
        assert!(duplicated.is_err(), "finding ids must be unique");
        let unordered = FindingsRegistry::new(vec![
            ok(FindingEntry::new(
                "zeta-finding",
                "task-surface",
                "A finding.",
                GuardStatus::Unguarded,
            )),
            ok(FindingEntry::new(
                "alpha-finding",
                "task-surface",
                "Another finding.",
                GuardStatus::Unguarded,
            )),
        ]);
        assert!(
            unordered.is_ok_and(|registry| registry.findings[0].id.as_str() == "alpha-finding"),
            "entries are canonically ordered by id"
        );
        let unknown_field = serialized.replace("\"v\":", "\"version\":");
        assert!(
            serde_json::from_str::<FindingsRegistry>(&unknown_field).is_err(),
            "unknown fields are rejected"
        );
        // Bound checks surface as named errors.
        assert!(FindingEntry::new("", "task-surface", "d", GuardStatus::Unguarded).is_err());
        assert!(
            FindingEntry::new(
                "a-finding",
                "task-surface",
                &"d".repeat(MAX_EXPLANATION_BYTES + 1),
                GuardStatus::Unguarded
            )
            .is_err()
        );
        assert!(
            FindingEntry::new(
                "a-finding",
                &"s".repeat(MAX_NAME_BYTES + 1),
                "d",
                GuardStatus::Unguarded
            )
            .is_err()
        );
        assert!(ensure_non_empty("field", "").is_err());
        assert!(ensure_name("field", "").is_err());
        assert!(ensure_str_bound("field", "x", 0).is_err());
    }
}
