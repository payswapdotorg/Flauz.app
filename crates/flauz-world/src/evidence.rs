//! The evidence plane (CONTEXT-HARNESS-ARCHITECTURE §6, kernel §7).
//!
//! An agent statement is not automatically truth. [`Observation`] records
//! what a surface directly showed. [`Claim`] records an assertion. Neither is
//! verified fact: [`Evidence`] is the verified record, and it always carries
//! verifier actor attribution, a verification event reference and a
//! timestamp. A Claim can never be typed, serialized, or parsed as Evidence.
//!
//! Conflicting observations about the same subject stay distinct records
//! with surface attribution; there is no merging and no silent dedup.

use serde::{Deserialize, Serialize};

use crate::ids::{ArtifactId, ClaimId, EventId, EvidenceId, ObservationId, ResourceId};
use crate::refs::ActorRef;
use crate::resource::SurfaceKind;
use crate::time::Timestamp;
use crate::{
    ContractVersion, MAX_RELATED_REFS, MAX_STATEMENT_BYTES, WorldError, ensure_list_bound,
    ensure_non_empty, ensure_str_bound,
};

/// The verification states of a claim or observation (kernel §7):
/// `claimed | observed | verified | contradicted | stale | unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    /// Asserted by an actor, not yet observed or verified.
    Claimed,
    /// Directly seen through an access surface.
    Observed,
    /// Verified with evidence (verifier actor + verification event +
    /// timestamp).
    Verified,
    /// Contradicted by other evidence.
    Contradicted,
    /// No longer current.
    Stale,
    /// Verification state unknown.
    Unknown,
}

/// A record of something directly observed through an access surface.
/// Observations about the same subject from different surfaces or actors
/// stay distinct records with surface attribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical observation ID (`obs_<ULID>`).
    pub id: ObservationId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The resource this observation is about.
    pub resource_id: ResourceId,
    /// The access surface through which the observation was made.
    pub surface: SurfaceKind,
    /// The actor that observed.
    pub actor: ActorRef,
    /// Observation timestamp (caller-supplied).
    pub observed_at: Timestamp,
    /// The bounded statement of what was observed.
    pub statement: String,
    /// The verification status; observations start observed.
    pub verification: VerificationStatus,
    /// Supporting artifact references.
    pub supporting_artifact_ids: Vec<ArtifactId>,
    /// The evidence that verified this observation, if any.
    pub verifying_evidence_id: Option<EvidenceId>,
    /// Evidence references that contradict this observation, where
    /// applicable.
    pub contradicting_evidence_ids: Vec<EvidenceId>,
}

impl Observation {
    /// Builds a new observation at version 1. New observations start with
    /// verification status `observed`; status changes are durable updates.
    pub fn new(
        id: ObservationId,
        resource_id: ResourceId,
        surface: SurfaceKind,
        actor: ActorRef,
        observed_at: Timestamp,
        statement: &str,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("observation statement", statement)?;
        ensure_str_bound("observation statement", statement, MAX_STATEMENT_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            resource_id,
            surface,
            actor,
            observed_at,
            statement: statement.to_owned(),
            verification: VerificationStatus::Observed,
            supporting_artifact_ids: Vec::new(),
            verifying_evidence_id: None,
            contradicting_evidence_ids: Vec::new(),
        })
    }

    /// Validates the observation.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            self.resource_id.clone(),
            self.surface,
            self.actor.clone(),
            self.observed_at,
            &self.statement,
        )?;
        ensure_list_bound(
            "supporting artifact ids",
            &self.supporting_artifact_ids,
            MAX_RELATED_REFS,
        )?;
        ensure_list_bound(
            "contradicting evidence ids",
            &self.contradicting_evidence_ids,
            MAX_RELATED_REFS,
        )?;
        if self.version == 0 {
            return Err(WorldError::invalid(
                "observation version must be at least 1",
            ));
        }
        Ok(())
    }
}

/// An assertion by an actor about a resource. A claim is not truth: it is
/// verified (or contradicted) through the evidence plane, and a Claim can
/// never be typed, serialized or parsed as [`Evidence`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical claim ID (`claim_<ULID>`).
    pub id: ClaimId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The resource this claim is about.
    pub resource_id: ResourceId,
    /// The access surface the claim derives from, if any.
    pub surface: Option<SurfaceKind>,
    /// The actor that made the claim.
    pub actor: ActorRef,
    /// Claim timestamp (caller-supplied).
    pub claimed_at: Timestamp,
    /// The bounded statement being claimed.
    pub statement: String,
    /// The verification status; claims start claimed.
    pub verification: VerificationStatus,
    /// Supporting artifact references.
    pub supporting_artifact_ids: Vec<ArtifactId>,
    /// The evidence that verified this claim, if any.
    pub verifying_evidence_id: Option<EvidenceId>,
    /// Evidence references that contradict this claim, where applicable.
    pub contradicting_evidence_ids: Vec<EvidenceId>,
}

impl Claim {
    /// Builds a new claim at version 1. New claims start with verification
    /// status `claimed`; verification is performed by producing evidence.
    pub fn new(
        id: ClaimId,
        resource_id: ResourceId,
        surface: Option<SurfaceKind>,
        actor: ActorRef,
        claimed_at: Timestamp,
        statement: &str,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("claim statement", statement)?;
        ensure_str_bound("claim statement", statement, MAX_STATEMENT_BYTES)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            resource_id,
            surface,
            actor,
            claimed_at,
            statement: statement.to_owned(),
            verification: VerificationStatus::Claimed,
            supporting_artifact_ids: Vec::new(),
            verifying_evidence_id: None,
            contradicting_evidence_ids: Vec::new(),
        })
    }

    /// Validates the claim.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            self.resource_id.clone(),
            self.surface,
            self.actor.clone(),
            self.claimed_at,
            &self.statement,
        )?;
        ensure_list_bound(
            "supporting artifact ids",
            &self.supporting_artifact_ids,
            MAX_RELATED_REFS,
        )?;
        ensure_list_bound(
            "contradicting evidence ids",
            &self.contradicting_evidence_ids,
            MAX_RELATED_REFS,
        )?;
        if self.version == 0 {
            return Err(WorldError::invalid("claim version must be at least 1"));
        }
        Ok(())
    }
}

/// A verified record. Evidence always requires verifier actor attribution, a
/// verification event reference and a timestamp (kernel §7); these fields
/// have no counterpart on [`Claim`], so a Claim can never be constructed,
/// serialized, or parsed as Evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical evidence ID (`evd_<ULID>`).
    pub id: EvidenceId,
    /// Durable entity version, starting at 1.
    pub version: u64,
    /// The claim this evidence verifies.
    pub claim_id: ClaimId,
    /// The verifier actor attribution (required).
    pub verifier: ActorRef,
    /// The verification event reference (required).
    pub verification_event_id: EventId,
    /// The verification timestamp (required, caller-supplied).
    pub verified_at: Timestamp,
}

impl Evidence {
    /// Builds new evidence. The verification event reference must reference
    /// an existing `evidence.verified` event appended by the store.
    pub fn new(
        id: EvidenceId,
        claim_id: ClaimId,
        verifier: ActorRef,
        verification_event_id: EventId,
        verified_at: Timestamp,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            claim_id,
            verifier,
            verification_event_id,
            verified_at,
        })
    }

    /// Validates the evidence.
    pub fn validate(&self) -> Result<(), WorldError> {
        self.verifier.validate()?;
        if self.version == 0 {
            return Err(WorldError::invalid("evidence version must be at least 1"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn verification_statuses_serialize_to_the_kernel_vocabulary() {
        for (status, text) in [
            (VerificationStatus::Claimed, "\"claimed\""),
            (VerificationStatus::Observed, "\"observed\""),
            (VerificationStatus::Verified, "\"verified\""),
            (VerificationStatus::Contradicted, "\"contradicted\""),
            (VerificationStatus::Stale, "\"stale\""),
            (VerificationStatus::Unknown, "\"unknown\""),
        ] {
            assert_eq!(ok(serde_json::to_string(&status)), text);
        }
        assert!(serde_json::from_str::<VerificationStatus>("\"proven\"").is_err());
    }

    #[test]
    fn observations_and_claims_start_unverified() {
        let actor = ok(ActorRef::agent("agent_01J8ZQ5V8K3T2B7N6X4R9DQPG6"));
        let ts = ok(Timestamp::parse("2026-09-21T13:45:00Z"));
        let observation = ok(Observation::new(
            ObservationId::generate(),
            ResourceId::generate(),
            SurfaceKind::Browser,
            actor.clone(),
            ts,
            "The dashboard shows 3 open tickets",
        ));
        assert_eq!(observation.verification, VerificationStatus::Observed);

        let claim = ok(Claim::new(
            ClaimId::generate(),
            ResourceId::generate(),
            Some(SurfaceKind::Api),
            actor,
            ts,
            "The ticket count matches the CRM export",
        ));
        assert_eq!(claim.verification, VerificationStatus::Claimed);
    }
}
