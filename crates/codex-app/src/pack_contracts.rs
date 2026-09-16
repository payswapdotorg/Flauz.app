//! Pack wire-contract mirror (PACK-UX-001).
//!
//! The serde-facing client mirror of the Codex Universal pack contracts
//! (PACK-001..005, `codex-rs/pack-contracts` in the codex repository).
//! Field names, optionality, and enum tagging follow
//! `docs/architecture/PACK-CLIENT-CONTRACT-MANIFEST.md` and the golden
//! fixtures byte-for-byte: `deny_unknown_fields` everywhere, camelCase
//! wire names, digests as opaque strings.
//!
//! Boundary rules (frozen client-adapter architecture):
//!
//! - the mirror CONSUMES pack contract responses; it never recomputes
//!   digests, never verifies platform integrity, and never becomes
//!   durable pack authority;
//! - candidate and promoted records are the same wire shape, so the
//!   catalog entry carries the record kind explicitly — the kind comes
//!   from the contract source, never from shape sniffing;
//! - until the app-server grows pack protocol methods, the embedded
//!   golden catalog (`pack_fixtures/catalog.json`, derived verbatim from
//!   the codex repository's byte-stable golden fixtures) stands in as the
//!   contract source. The view never fabricates pack data.

use serde::Deserialize;
use serde::Serialize;

use codex_core::PackAssuranceView;
use codex_core::PackCompositionView;
use codex_core::PackDependencyEntryView;
use codex_core::PackGovernanceView;
use codex_core::PackMissionView;
use codex_core::PackProvenanceView;
use codex_core::PackRecordKind;
use codex_core::PackRecordView;
use codex_core::PackState;
use codex_core::PackSystemStateView;

/// The embedded golden catalog: four pack records (root candidate, child
/// candidate, promoted revision, composed candidate) derived verbatim
/// from the codex repository's golden wire fixtures.
const GOLDEN_CATALOG: &str = include_str!("pack_fixtures/catalog.json");

// ---------------------------------------------------------------------------
// Wire mirror (serde types; camelCase + deny_unknown_fields everywhere)
// ---------------------------------------------------------------------------

/// One catalog entry: the record kind plus the record payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackCatalogEntryWire {
    /// The governed lifecycle position, stated by the contract source.
    pub kind: PackRecordKindWire,
    /// The record payload.
    pub record: PackRecordWire,
}

/// Record kinds on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackRecordKindWire {
    /// A proposed system state awaiting a promotion decision.
    Candidate,
    /// An immutable promoted revision.
    Promoted,
}

/// A pack record on the wire (candidate or promoted shape).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackRecordWire {
    /// Pack lineage id.
    pub pack_id: String,
    /// Semantic version, `"major.minor.patch"`.
    pub semantic_version: String,
    /// Parent lineage edge, absent on roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision: Option<PackParentRevisionWire>,
    /// The mission model.
    pub mission: PackMissionWire,
    /// Content digest of the mission (opaque).
    pub mission_digest: String,
    /// The governing policy set.
    pub policy_set: PackPolicySetWire,
    /// Content digest of the policy content (opaque).
    pub policy_digest: String,
    /// The resolved dependency lock.
    pub dependency_lock: PackDependencyLockWire,
    /// Content digest of the lock (opaque).
    pub dependency_lock_digest: String,
    /// The system state.
    pub system_state: PackSystemStateWire,
    /// Content digest of the system state (opaque).
    pub system_state_digest: String,
    /// Production provenance (descriptive).
    pub provenance: PackProvenanceWire,
    /// Two-parent composition record, on composed revisions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition: Option<PackCompositionWire>,
    /// Digest of the composition record, on composed revisions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_digest: Option<String>,
    /// The candidate revision this promoted revision was created from
    /// (promoted records only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promoted_from: Option<String>,
    /// The record's revision identity (opaque).
    pub revision_id: String,
}

/// Parent lineage edge on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackParentRevisionWire {
    /// The parent revision identity (opaque digest string).
    pub revision: String,
    /// Descriptive lineage label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// The mission model on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackMissionWire {
    /// Mission lineage id.
    pub id: String,
    /// The mission statement.
    pub statement: String,
    /// Who authored the mission content.
    pub author: PackMissionAuthorWire,
    /// Value objectives, most important first.
    pub value_model: PackValueModelWire,
    /// Operating-context notes.
    pub context_model: PackContextModelWire,
    /// Non-negotiable constraints.
    pub hard_constraints: Vec<PackStatementWire>,
    /// Overridable preferences.
    #[serde(default)]
    pub preferences: Vec<PackStatementWire>,
    /// Success measures.
    #[serde(default)]
    pub success_measures: Vec<PackStatementWire>,
}

/// Mission authorship: user authority vs agent proposal.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackMissionAuthorWire {
    /// `"user"` or `"agentProposal"`.
    pub kind: String,
    /// The authoring principal (opaque).
    pub subject: String,
}

/// The value model on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackValueModelWire {
    /// Objectives, most important first.
    pub objectives: Vec<PackStatementWire>,
}

/// The context model on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackContextModelWire {
    /// Context notes (these power contextual activation).
    pub notes: Vec<PackStatementWire>,
}

/// A statement-bearing element on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackStatementWire {
    /// The statement text.
    pub statement: String,
}

/// The governing policy set on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackPolicySetWire {
    /// The pack constitution.
    pub constitution: PackConstitutionWire,
    /// Governing policies, in authored order.
    pub policies: Vec<PackPolicyWire>,
}

/// The pack constitution on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackConstitutionWire {
    /// Domain rules, in authored order. Rule variants are externally
    /// tagged camelCase objects; the mirror keeps them as raw values and
    /// renders the tag.
    pub rules: Vec<serde_json::Value>,
    /// Platform invariants acknowledged as unweakenable.
    pub protected_invariants: Vec<String>,
}

/// One governing policy on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackPolicyWire {
    /// What the policy governs (e.g. `"dependencyUpdates"`).
    pub scope: String,
    /// Policy statements, in authored order.
    pub statements: Vec<String>,
    /// The explicit authority boundary: what the policy governs plus the
    /// authorities it explicitly does not hold.
    pub authority_boundary: PackPolicyAuthorityBoundaryWire,
    /// Policies referenced by this record, by content-addressed identity
    /// (for example composed assurance policies).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_policies: Vec<String>,
}

/// A policy's explicit authority boundary on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackPolicyAuthorityBoundaryWire {
    /// What the boundary governs.
    pub governs: String,
    /// Authorities the policy explicitly does not hold (workflow
    /// transitions, credentials, evidence).
    pub retained_authorities: Vec<String>,
}

/// The dependency lock on the wire. Entries are keyed by composite
/// strings (`"workflow:<id>"` / `"capability:<id>"` / `"pack:<id>"`); the
/// key also repeats inside each entry (tamper evidence).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct PackDependencyLockWire {
    /// Resolved entries by declared dependency key.
    pub entries: std::collections::BTreeMap<String, PackDependencyEntryWire>,
}

/// One resolved dependency entry on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackDependencyEntryWire {
    /// The declared dependency key (repeats the map key).
    pub key: String,
    /// The resolved immutable identity.
    pub resolved: serde_json::Value,
    /// Content digest of the resolved artifact (opaque).
    pub content_digest: String,
}

/// The system state on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackSystemStateWire {
    /// Pinned immutable workflow versions.
    pub workflow_version_refs: Vec<PackWorkflowVersionRefWire>,
    /// Required capabilities.
    pub capability_refs: Vec<PackCapabilityRefWire>,
    /// Referenced governing/assurance policies.
    pub policy_refs: Vec<PackPolicyRefWire>,
    /// Opaque evaluation reference digests.
    #[serde(default)]
    pub evaluation_refs: Vec<String>,
    /// Opaque evidence reference digests.
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    /// Recorded rollback point, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_checkpoint: Option<PackRollbackCheckpointWire>,
}

/// A pinned workflow version reference on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackWorkflowVersionRefWire {
    /// The pinned immutable workflow version identity (opaque).
    pub workflow_version_id: String,
    /// Descriptive role within the pack state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// A required capability on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackCapabilityRefWire {
    /// The semantic capability identity.
    pub capability: String,
    /// Descriptive constraint on satisfaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
}

/// A policy reference on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackPolicyRefWire {
    /// The referenced policy identity (opaque).
    pub policy_id: String,
    /// The content digest the state was validated against (opaque).
    pub validated_content_digest: String,
}

/// A rollback checkpoint on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackRollbackCheckpointWire {
    /// The prior promoted revision a rollback would restore (opaque).
    pub target_revision: String,
    /// Digest of the target revision's system state (opaque).
    pub state_digest: String,
    /// Why the rollback point was recorded.
    pub reason: String,
}

/// Production provenance on the wire (descriptive; never authority).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackProvenanceWire {
    /// The producing principal.
    pub producer: PackProducerWire,
    /// Parent revision named by provenance, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision: Option<String>,
}

/// The producing principal on the wire. The shape is variant-dependent:
/// user/agent principals carry `subject`; control-plane principals carry
/// `plane`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackProducerWire {
    /// `"user"`, `"agent"`, or `"controlPlane"`.
    pub kind: String,
    /// The principal (opaque), on user/agent producers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The control plane (opaque), on control-plane producers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<String>,
}

/// The two-parent composition record on the wire (PACK-005).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackCompositionWire {
    /// The base parent's full revision identity tuple.
    pub base: PackRevisionIdentityWire,
    /// The overlay parent's full revision identity tuple.
    pub overlay: PackRevisionIdentityWire,
    /// `"specialization"` or `"orthogonal"`.
    pub relation: String,
    /// The composition strategy.
    pub strategy: PackCompositionStrategyWire,
    /// Contextual activation rules (absent = unconditionally active).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activations: Vec<PackContextualActivationWire>,
}

/// A revision identity tuple on the wire. NOTE: `parentRevision` here is a
/// bare string, unlike the record-level lineage edge object — two shapes,
/// mirrored exactly (see the client-contract manifest, hazard 1).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackRevisionIdentityWire {
    /// Pack lineage id.
    pub pack: String,
    /// Semantic version string.
    pub semantic_version: String,
    /// System-state digest (opaque).
    pub system_state_digest: String,
    /// Mission digest (opaque).
    pub mission_digest: String,
    /// Policy digest (opaque).
    pub policy_digest: String,
    /// Dependency-lock digest (opaque).
    pub dependency_lock_digest: String,
    /// Parent revision identity (opaque string), when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision: Option<String>,
    /// Composition digest (opaque), when the parent was itself composed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_digest: Option<String>,
}

/// The composition strategy on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackCompositionStrategyWire {
    /// `"requireCompatible"` or `"union"`.
    pub activation_merge: String,
}

/// One contextual activation rule on the wire.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackContextualActivationWire {
    /// The bound target (workflow version or capability).
    pub target: serde_json::Value,
    /// `"always"` or a context-note condition.
    pub scope: serde_json::Value,
}

/// The assurance policy on the wire: seven independent dimensions, each
/// absent when not required (there is no global deterministic mode).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct PackAssurancePolicyWire {
    /// Determinism dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub determinism: Option<serde_json::Value>,
    /// Replay dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay: Option<serde_json::Value>,
    /// Approval dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<serde_json::Value>,
    /// Evidence dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    /// Model pinning dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_pinning: Option<serde_json::Value>,
    /// Dependency pinning dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_pinning: Option<serde_json::Value>,
    /// Environment pinning dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_pinning: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Catalog loading + mapping into the domain view models
// ---------------------------------------------------------------------------

/// The wire catalog document.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PackCatalogWire {
    /// The catalog entries.
    records: Vec<PackCatalogEntryWire>,
    /// Assurance policy content keyed by the policy identity a system
    /// state references (assurance artifacts are referenced by policy id,
    /// not embedded in records).
    #[serde(default)]
    assurance_policies: Vec<PackAssurancePolicyEntryWire>,
}

/// One assurance-policy side-table entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PackAssurancePolicyEntryWire {
    /// The referenced policy identity (opaque).
    policy_id: String,
    /// The assurance policy content.
    policy: PackAssurancePolicyWire,
}

/// Loads the embedded golden catalog into the pack view state.
///
/// This is the pack contract seam: today the golden fixture catalog stands
/// in for the future app-server pack protocol. A load failure surfaces as
/// an error banner (never swallowed, never fabricated).
pub fn load_pack_state() -> PackState {
    match load_pack_entries(GOLDEN_CATALOG) {
        Ok(records) => PackState {
            selected: records.first().map(|record| record.revision_id.clone()),
            records,
            error: None,
        },
        Err(error) => PackState {
            records: Vec::new(),
            selected: None,
            error: Some(error),
        },
    }
}

/// Parses and maps a wire catalog document.
fn load_pack_entries(document: &str) -> Result<Vec<PackRecordView>, String> {
    let catalog: PackCatalogWire =
        serde_json::from_str(document).map_err(|error| format!("pack catalog: {error}"))?;
    let assurance_by_id: std::collections::BTreeMap<String, PackAssurancePolicyWire> = catalog
        .assurance_policies
        .into_iter()
        .map(|entry| (entry.policy_id, entry.policy))
        .collect();
    catalog
        .records
        .into_iter()
        .map(|entry| {
            let kind = match entry.kind {
                PackRecordKindWire::Candidate => PackRecordKind::Candidate,
                PackRecordKindWire::Promoted => PackRecordKind::Promoted,
            };
            map_record(kind, entry.record, &assurance_by_id)
        })
        .collect()
}

/// Maps one wire record into the domain view model. Values are carried
/// verbatim; nothing is recomputed. Assurance dimensions resolve through
/// the side table by the policy identities the system state references.
fn map_record(
    kind: PackRecordKind,
    wire: PackRecordWire,
    assurance_by_id: &std::collections::BTreeMap<String, PackAssurancePolicyWire>,
) -> Result<PackRecordView, String> {
    let governance = PackGovernanceView {
        constitution_rules: wire
            .policy_set
            .constitution
            .rules
            .iter()
            .map(|rule| {
                rule.as_object()
                    .and_then(|object| object.keys().next().cloned())
                    .unwrap_or_else(|| "unknown".to_owned())
            })
            .collect(),
        protected_invariants: wire.policy_set.constitution.protected_invariants,
        policy_scopes: wire
            .policy_set
            .policies
            .iter()
            .map(|policy| policy.scope.clone())
            .collect(),
    };

    let dependencies = wire
        .dependency_lock
        .entries
        .values()
        .map(|entry| {
            let (resolved_kind, resolved_identity) = resolved_identity(&entry.resolved);
            PackDependencyEntryView {
                key: entry.key.clone(),
                resolved_kind,
                resolved_identity,
                content_digest: entry.content_digest.clone(),
            }
        })
        .collect();

    let rollback_checkpoint = wire
        .system_state
        .rollback_checkpoint
        .as_ref()
        .map(|checkpoint| {
            format!(
                "{} · {} ({})",
                short_digest(&checkpoint.target_revision),
                short_digest(&checkpoint.state_digest),
                checkpoint.reason
            )
        });

    // Resolve the assurance dimensions from the side table for every
    // policy reference that resolves to a carried assurance policy.
    let mut assurance = PackAssuranceView::default();
    for reference in &wire.system_state.policy_refs {
        if let Some(policy) = assurance_by_id.get(&reference.policy_id) {
            assurance = map_assurance(policy);
        }
    }

    let composition = wire.composition.as_ref().map(|record| PackCompositionView {
        base_revision: format!("{} @ {}", record.base.pack, record.base.semantic_version),
        overlay_revision: format!(
            "{} @ {}",
            record.overlay.pack, record.overlay.semantic_version
        ),
        relation: record.relation.clone(),
        strategy: record.strategy.activation_merge.clone(),
        activations: record
            .activations
            .iter()
            .map(|activation| {
                (
                    activation_target_label(&activation.target),
                    activation_scope_label(&activation.scope),
                )
            })
            .collect(),
    });

    Ok(PackRecordView {
        kind,
        pack_id: wire.pack_id,
        semantic_version: wire.semantic_version,
        revision_id: wire.revision_id,
        parent_revision: wire
            .parent_revision
            .map(|parent| (parent.revision, parent.label)),
        promoted_from: wire.promoted_from,
        mission: PackMissionView {
            id: wire.mission.id,
            statement: wire.mission.statement,
            user_authored: wire.mission.author.kind == "user",
            author_subject: wire.mission.author.subject,
            objectives: wire
                .mission
                .value_model
                .objectives
                .into_iter()
                .map(|objective| objective.statement)
                .collect(),
            context_notes: wire
                .mission
                .context_model
                .notes
                .into_iter()
                .map(|note| note.statement)
                .collect(),
            hard_constraints: wire
                .mission
                .hard_constraints
                .into_iter()
                .map(|constraint| constraint.statement)
                .collect(),
        },
        governance,
        dependencies,
        system_state: PackSystemStateView {
            workflow_versions: wire
                .system_state
                .workflow_version_refs
                .into_iter()
                .map(|reference| (reference.workflow_version_id, reference.role))
                .collect(),
            capabilities: wire
                .system_state
                .capability_refs
                .into_iter()
                .map(|reference| (reference.capability, reference.constraint))
                .collect(),
            policies: wire
                .system_state
                .policy_refs
                .into_iter()
                .map(|reference| (reference.policy_id, reference.validated_content_digest))
                .collect(),
            evaluations: wire.system_state.evaluation_refs,
            evidence: wire.system_state.evidence_refs,
            rollback_checkpoint,
        },
        assurance,
        provenance: PackProvenanceView {
            producer_kind: wire.provenance.producer.kind,
            producer_subject: wire
                .provenance
                .producer
                .subject
                .or(wire.provenance.producer.plane)
                .unwrap_or_else(|| "unknown".to_owned()),
            parent_revision: wire.provenance.parent_revision,
        },
        composition,
    })
}

/// Extracts the resolved identity from a raw `resolved` value.
fn resolved_identity(resolved: &serde_json::Value) -> (String, String) {
    if let Some(object) = resolved.as_object() {
        for (kind, value) in object {
            if let Some(identity) = value.as_str() {
                return (kind.clone(), identity.to_owned());
            }
        }
    }
    ("unknown".to_owned(), resolved.to_string())
}

/// Short display form of an opaque digest (never parsed for meaning).
fn short_digest(digest: &str) -> String {
    if digest.len() <= 14 {
        digest.to_owned()
    } else {
        format!("{}…", &digest[..13])
    }
}

/// Renders an activation target's label.
fn activation_target_label(target: &serde_json::Value) -> String {
    if let Some(object) = target.as_object() {
        for (kind, value) in object {
            if let Some(identity) = value.as_str() {
                return format!("{kind}:{}", short_digest(identity));
            }
        }
    }
    target.to_string()
}

/// Renders an activation scope's label.
fn activation_scope_label(scope: &serde_json::Value) -> String {
    if let Some(condition) = scope.as_str() {
        return condition.to_owned();
    }
    if let Some(object) = scope.as_object()
        && let Some(note) = object.get("whenContextNote").and_then(|v| v.as_str())
    {
        return format!("when context: {note}");
    }
    scope.to_string()
}

/// Maps a wire assurance policy into the dimension view model. Raw
/// dimension values are rendered as their wire tags; pins carry identity
/// plus digest (never credentials).
fn map_assurance(policy: &PackAssurancePolicyWire) -> PackAssuranceView {
    let dimension_label = |value: &serde_json::Value| -> Option<String> {
        value
            .as_object()
            .and_then(|object| {
                object.values().next().and_then(|inner| {
                    inner
                        .as_str()
                        .map(|text| text.to_owned())
                        .or_else(|| Some(inner.to_string()))
                })
            })
            .or_else(|| value.as_str().map(|text| text.to_owned()))
    };
    let pin_pair = |value: &serde_json::Value| -> Option<(String, String)> {
        value
            .as_object()
            .and_then(|object| object.get("pin"))
            .map(|pin| {
                let identity = pin
                    .get("model")
                    .or_else(|| pin.get("dependency"))
                    .or_else(|| pin.get("environment"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                let digest = pin
                    .get("digest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                (identity, digest)
            })
    };
    PackAssuranceView {
        determinism: policy.determinism.as_ref().and_then(dimension_label),
        replay: policy.replay.as_ref().and_then(dimension_label),
        approval: policy.approval.as_ref().and_then(dimension_label),
        evidence: policy.evidence.as_ref().and_then(dimension_label),
        model_pinning: policy.model_pinning.as_ref().and_then(pin_pair),
        dependency_pinning: policy.dependency_pinning.as_ref().and_then(pin_pair),
        environment_pinning: policy.environment_pinning.as_ref().and_then(pin_pair),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn golden_catalog_loads_into_view_models() {
        let state = load_pack_state();
        assert!(
            state.error.is_none(),
            "catalog must load: {:?}",
            state.error
        );
        assert_eq!(state.records.len(), 4, "root, child, promoted, composed");
        assert!(state.selected.is_some(), "first record pre-selected");

        let kinds: Vec<&str> = state
            .records
            .iter()
            .map(|record| match record.kind {
                PackRecordKind::Candidate => "candidate",
                PackRecordKind::Promoted => "promoted",
            })
            .collect();
        assert_eq!(kinds, ["candidate", "candidate", "promoted", "candidate"]);
    }

    #[test]
    fn candidate_and_promoted_are_distinct_lifecycle_facts() {
        let state = load_pack_state();
        let promoted = state
            .records
            .iter()
            .find(|record| record.kind == PackRecordKind::Promoted)
            .expect("promoted record present");
        assert!(promoted.promoted_from.is_some());
        let candidates_with_parent = state
            .records
            .iter()
            .filter(|record| {
                record.kind == PackRecordKind::Candidate && record.parent_revision.is_some()
            })
            .count();
        assert!(candidates_with_parent >= 1);
    }

    #[test]
    fn composed_record_carries_two_parent_provenance() {
        let state = load_pack_state();
        let composed = state
            .records
            .iter()
            .find(|record| record.composition.is_some())
            .expect("composed record present");
        let composition = composed.composition.as_ref().expect("composition view");
        assert_eq!(composition.relation, "orthogonal");
        assert!(!composition.base_revision.is_empty());
        assert!(!composition.overlay_revision.is_empty());
    }

    #[test]
    fn assurance_dimensions_resolve_through_policy_references() {
        let state = load_pack_state();
        let composed = state
            .records
            .iter()
            .find(|record| record.composition.is_some())
            .expect("composed record present");
        let required = composed.assurance.required();
        assert!(
            required.iter().any(|(name, _)| *name == "model pinning"),
            "the composed record requires model pinning: {required:?}"
        );
        // Independent dimensions: the golden assurance policy pins only
        // the model; absence of the others is "no requirement".
        assert!(composed.assurance.determinism.is_none());
    }

    #[test]
    fn mission_authority_is_explicit() {
        let state = load_pack_state();
        for record in &state.records {
            assert!(
                record.mission.user_authored,
                "catalog missions are user-authored"
            );
            assert!(!record.mission.statement.is_empty());
        }
    }

    #[test]
    fn wire_mirror_round_trips_and_rejects_unknown_fields() {
        let entry = PackCatalogEntryWire {
            kind: PackRecordKindWire::Candidate,
            record: minimal_record_wire(),
        };
        let wire = serde_json::to_value(&entry).expect("serializes");
        let revived: PackCatalogEntryWire =
            serde_json::from_value(wire.clone()).expect("round-trips");
        assert_eq!(revived.kind, entry.kind);

        let mut tampered = wire;
        tampered
            .as_object_mut()
            .expect("object")
            .insert("__sneaky".to_owned(), json!(true));
        assert!(serde_json::from_value::<PackCatalogEntryWire>(tampered).is_err());
    }

    #[test]
    fn identity_tuple_parent_is_a_bare_string_not_an_object() {
        // Hazard 1 from the manifest: identity-tuple parentRevision is a
        // string while record-level parentRevision is an object. The
        // mirror must accept the string form here.
        let identity = json!({
            "pack": "fixture",
            "semanticVersion": "0.1.0",
            "systemStateDigest": "sha256:a",
            "missionDigest": "sha256:b",
            "policyDigest": "sha256:c",
            "dependencyLockDigest": "sha256:d",
            "parentRevision": "sha256:parent"
        });
        let revived: PackRevisionIdentityWire =
            serde_json::from_value(identity).expect("string parent revives");
        assert_eq!(revived.parent_revision.as_deref(), Some("sha256:parent"));
    }

    fn minimal_record_wire() -> PackRecordWire {
        PackRecordWire {
            pack_id: "fixture".to_owned(),
            semantic_version: "0.1.0".to_owned(),
            parent_revision: None,
            mission: PackMissionWire {
                id: "m".to_owned(),
                statement: "s".to_owned(),
                author: PackMissionAuthorWire {
                    kind: "user".to_owned(),
                    subject: "org:fixture".to_owned(),
                },
                value_model: PackValueModelWire { objectives: vec![] },
                context_model: PackContextModelWire { notes: vec![] },
                hard_constraints: vec![],
                preferences: vec![],
                success_measures: vec![],
            },
            mission_digest: "sha256:m".to_owned(),
            policy_set: PackPolicySetWire {
                constitution: PackConstitutionWire {
                    rules: vec![],
                    protected_invariants: vec![],
                },
                policies: vec![],
            },
            policy_digest: "sha256:p".to_owned(),
            dependency_lock: PackDependencyLockWire::default(),
            dependency_lock_digest: "sha256:l".to_owned(),
            system_state: PackSystemStateWire {
                workflow_version_refs: vec![],
                capability_refs: vec![],
                policy_refs: vec![],
                evaluation_refs: vec![],
                evidence_refs: vec![],
                rollback_checkpoint: None,
            },
            system_state_digest: "sha256:s".to_owned(),
            provenance: PackProvenanceWire {
                producer: PackProducerWire {
                    kind: "user".to_owned(),
                    subject: Some("org:fixture".to_owned()),
                    plane: None,
                },
                parent_revision: None,
            },
            composition: None,
            composition_digest: None,
            promoted_from: None,
            revision_id: "sha256:r".to_owned(),
        }
    }
}
