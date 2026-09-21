//! The resource graph (CONTEXT-HARNESS-ARCHITECTURE §5, kernel §7).
//!
//! A [`Resource`] is the durable identity of a thing being worked with (a
//! website, repository, document, CRM record, dataset, host, folder, API
//! endpoint, browser session, device, ...). An [`AccessSurface`] is a way of
//! interacting with that resource; multiple surfaces may represent the same
//! resource, and the resource ID is stable when surfaces change. A
//! [`ResourceState`] is a bounded, attributable snapshot of what is known
//! about a resource.

use serde::{Deserialize, Serialize};

use crate::ids::{ArtifactId, EventId, LeaseId, ObservationId, ResourceId};
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, MAX_NAME_BYTES, MAX_RELATED_REFS, MAX_SURFACES_PER_RESOURCE, WorldError,
    ensure_list_bound, ensure_non_empty, ensure_str_bound,
};

/// The access surface kinds (kernel §7): how a resource is interacted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceKind {
    /// A browser UI surface.
    Browser,
    /// An API surface.
    Api,
    /// A CLI surface.
    Cli,
    /// An MCP surface.
    Mcp,
    /// Native desktop control.
    NativeDesktop,
    /// A file interface.
    File,
    /// A service integration.
    ServiceIntegration,
}

/// An access surface descriptor: carries the surface kind and references
/// the resource ID (kernel §7). A resource's identity is stable when its
/// surfaces change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessSurface {
    /// The surface kind.
    pub kind: SurfaceKind,
    /// The resource this surface accesses.
    pub resource_id: ResourceId,
}

impl AccessSurface {
    /// Builds an access surface descriptor.
    #[must_use]
    pub const fn new(kind: SurfaceKind, resource_id: ResourceId) -> Self {
        Self { kind, resource_id }
    }
}

/// A durable resource identity, decoupled from any particular access
/// surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical resource ID (`res_<ULID>`), stable across surface changes.
    pub id: ResourceId,
    /// Durable entity version, starting at 1, +1 per durable mutation
    /// (surface changes are durable mutations; the ID never changes).
    pub version: u64,
    /// Human-readable resource name.
    pub name: String,
    /// The access surfaces currently known for this resource. Each surface
    /// references this resource's ID.
    pub surfaces: Vec<AccessSurface>,
    /// The actor that created the resource.
    pub created_by: ActorRef,
    /// Creation timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl Resource {
    /// Builds a new resource at version 1.
    pub fn new(
        id: ResourceId,
        name: &str,
        surfaces: Vec<AccessSurface>,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, WorldError> {
        ensure_non_empty("resource name", name)?;
        ensure_str_bound("resource name", name, MAX_NAME_BYTES)?;
        Self::validate_surfaces(&id, &surfaces)?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            name: name.to_owned(),
            surfaces,
            created_by,
            created_at,
        })
    }

    fn validate_surfaces(id: &ResourceId, surfaces: &[AccessSurface]) -> Result<(), WorldError> {
        ensure_list_bound("resource surfaces", surfaces, MAX_SURFACES_PER_RESOURCE)?;
        for surface in surfaces {
            if &surface.resource_id != id {
                return Err(WorldError::invalid(
                    "access surface references a different resource id",
                ));
            }
        }
        Ok(())
    }

    /// Validates the resource.
    pub fn validate(&self) -> Result<(), WorldError> {
        Self::new(
            self.id.clone(),
            &self.name,
            self.surfaces.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(WorldError::invalid("resource version must be at least 1"));
        }
        Ok(())
    }
}

/// A bounded, attributable snapshot of what is known about a resource
/// (CONTEXT-HARNESS-ARCHITECTURE §5): identity, permissions, last verified
/// state, authoritative observation source where known, active leases,
/// pending mutations, related artifacts, and provenance. All content is
/// references or bounded descriptors, never inlined bulk.
///
/// `ResourceState` is a projection record, not an independently identified
/// entity: it carries no canonical ID of its own and reflects the resource
/// version it was captured at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceState {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// The resource this state describes.
    pub resource_id: ResourceId,
    /// The resource version this state was captured at.
    pub resource_version: u64,
    /// Bounded permission descriptors.
    pub permissions: Vec<String>,
    /// The latest observation record whose verification status is verified,
    /// where one exists.
    pub last_verified_observation: Option<ObservationId>,
    /// The surface kind that last authoritatively observed the resource,
    /// where known.
    pub authoritative_surface: Option<SurfaceKind>,
    /// Active lease references (held-ness is evaluated against a
    /// caller-supplied "now"; expired leases are not held, kernel §7).
    pub active_lease_ids: Vec<LeaseId>,
    /// References to pending mutation events.
    pub pending_mutation_ids: Vec<EventId>,
    /// Related artifact references.
    pub related_artifact_ids: Vec<ArtifactId>,
    /// The actor that captured this state.
    pub captured_by: ActorRef,
    /// Capture timestamp (caller-supplied).
    pub captured_at: Timestamp,
}

impl ResourceState {
    /// Validates the resource state against canonical bounds.
    pub fn validate(&self) -> Result<(), WorldError> {
        for permission in &self.permissions {
            ensure_non_empty("resource permission", permission)?;
            ensure_str_bound("resource permission", permission, MAX_NAME_BYTES)?;
        }
        ensure_list_bound("resource permissions", &self.permissions, MAX_RELATED_REFS)?;
        ensure_list_bound("active lease ids", &self.active_lease_ids, MAX_RELATED_REFS)?;
        ensure_list_bound(
            "pending mutation ids",
            &self.pending_mutation_ids,
            MAX_RELATED_REFS,
        )?;
        ensure_list_bound(
            "related artifact ids",
            &self.related_artifact_ids,
            MAX_RELATED_REFS,
        )?;
        if self.resource_version == 0 {
            return Err(WorldError::invalid(
                "resource state must reference a resource version of at least 1",
            ));
        }
        self.captured_by.validate()?;
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
    fn surface_kinds_serialize_to_the_kernel_vocabulary() {
        assert_eq!(
            ok(serde_json::to_string(&SurfaceKind::Browser)),
            "\"browser\""
        );
        assert_eq!(ok(serde_json::to_string(&SurfaceKind::Api)), "\"api\"");
        assert_eq!(ok(serde_json::to_string(&SurfaceKind::Cli)), "\"cli\"");
        assert_eq!(ok(serde_json::to_string(&SurfaceKind::Mcp)), "\"mcp\"");
        assert_eq!(
            ok(serde_json::to_string(&SurfaceKind::NativeDesktop)),
            "\"native_desktop\""
        );
        assert_eq!(ok(serde_json::to_string(&SurfaceKind::File)), "\"file\"");
        assert_eq!(
            ok(serde_json::to_string(&SurfaceKind::ServiceIntegration)),
            "\"service_integration\""
        );
        assert!(serde_json::from_str::<SurfaceKind>("\"web\"").is_err());
    }

    #[test]
    fn resource_rejects_surfaces_for_other_resources() {
        let id = ResourceId::generate();
        let other = ResourceId::generate();
        let actor = ok(ActorRef::system("flauz-harness"));
        let ts = ok(Timestamp::parse("2026-09-21T13:45:00Z"));
        let surface = AccessSurface::new(SurfaceKind::Browser, other);
        assert!(Resource::new(id.clone(), "Acme CRM", vec![surface], actor.clone(), ts).is_err());
        let good = AccessSurface::new(SurfaceKind::Browser, id.clone());
        assert!(Resource::new(id, "Acme CRM", vec![good], actor, ts).is_ok());
    }
}
