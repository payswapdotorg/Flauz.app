//! The local execution environment provider (ENV-001): wraps the four F1
//! surfaces — terminal, browser, Computer Use and Git — as
//! [`Environment`] facets behind the frozen [`ExecutionProvider`]
//! contract (Wave-2 kernel addendum §1: no trait signature changes).
//!
//! The F1 surface topology is read from `codex-app` **for the descriptor
//! shape only** — this crate never imports `codex-app`. The mapping:
//!
//! | F1 surface | [`LocalSurface`] facet | Advertised capabilities |
//! |---|---|---|
//! | terminal | [`LocalSurface::Terminal`] | `terminal` |
//! | browser | [`LocalSurface::Browser`] | `browser`, `browser.input`, `browser.navigation` |
//! | Computer Use | [`LocalSurface::ComputerUse`] | `computer.screen`, `desktop.gui`, `keyboard`, `mouse`, `window` |
//! | git | [`LocalSurface::Git`] | `filesystem.read`, `filesystem.write`, `git` |
//!
//! A local environment is the user's own machine (constitution:
//! "Desktop clients can control local and remote environments"): it needs
//! no [`ProviderConnection`](crate::ProviderConnection), carries no
//! credential state at all, and executes with
//! [`Locality::Local`](crate::Locality).
//!
//! # Honest capability sourcing (addendum §4)
//!
//! The provider can only offer what the F1 surfaces actually provide.
//! Sourcing a spec whose capabilities fall outside the local topology is
//! rejected with the missing keys NAMED in the error — never a silent
//! fall-through to a lesser environment. The same rule guards
//! [`LocalEnvironment::from_descriptor`] when reconstructing the live
//! surface from durable state.
//!
//! # Determinism (kernel §7)
//!
//! No I/O, no wall clock: creation timestamps and actors come from the
//! [`EnvironmentSpec`] the caller supplies. The only entropy source is
//! canonical `env_` ULID generation (the private `ulid` module), exactly
//! as the frozen [`ExecutionProvider`] contract prescribes.
//!
//! # Durable state
//!
//! Sourced environments are durable [`EnvironmentDescriptor`] records at
//! version 1 (kernel §3); later durable mutations and optimistic
//! concurrency belong to the [`ExecStore`](crate::ExecStore) path, not to
//! the sourcing provider. The live [`LocalEnvironment`] wrapper is
//! stateless over its descriptor, so environment identity — the canonical
//! `env_` ULID — is carried entirely by the durable record and survives
//! every serialization round-trip.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;
use crate::environment::{
    Environment, EnvironmentDescriptor, EnvironmentSpec, EnvironmentStatus, ExecutionProvider,
    Locality,
};
use crate::ids::EnvironmentId;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{ExecError, MAX_ENVIRONMENTS_PER_PROVIDER};

/// The neutral kind label of the local execution provider.
pub const LOCAL_PROVIDER_KIND: &str = "local";

/// One F1 surface as an [`Environment`] facet of the local machine. The
/// facet set is the frozen F1 topology: terminal, browser, Computer Use
/// and Git. Each facet carries its capability advertisement (the frozen
/// [`CapabilityId`](crate::CapabilityId) vocabulary); a facet is wrapped
/// by a [`LocalEnvironment`] exactly when all of its capabilities are
/// offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalSurface {
    /// The F1 terminal surface: shell execution on the user's machine.
    Terminal,
    /// The F1 browser surface: navigation and input.
    Browser,
    /// The F1 Computer Use surface: screen, keyboard, mouse and window
    /// control (the compound `desktop.gui` capability).
    ComputerUse,
    /// The F1 Git surface: repository operations over the local
    /// filesystem.
    Git,
}

impl LocalSurface {
    /// The F1 surface label of this facet (`terminal`, `browser`,
    /// `computer-use`, `git`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::Browser => "browser",
            Self::ComputerUse => "computer-use",
            Self::Git => "git",
        }
    }

    /// The capability advertisement of this facet: valid, sorted,
    /// deduplicated keys (the kernel advertisement rule).
    #[must_use]
    pub fn capabilities(self) -> Vec<CapabilityId> {
        match self {
            Self::Terminal => vec![frozen_capability("terminal")],
            Self::Browser => vec![
                frozen_capability("browser"),
                frozen_capability("browser.input"),
                frozen_capability("browser.navigation"),
            ],
            Self::ComputerUse => vec![
                frozen_capability("computer.screen"),
                frozen_capability("desktop.gui"),
                frozen_capability("keyboard"),
                frozen_capability("mouse"),
                frozen_capability("window"),
            ],
            Self::Git => vec![
                frozen_capability("filesystem.read"),
                frozen_capability("filesystem.write"),
                frozen_capability("git"),
            ],
        }
    }
}

/// The frozen F1 surface topology: the four facets every
/// [`LocalEnvironmentProvider`] wraps.
pub const LOCAL_SURFACES: [LocalSurface; 4] = [
    LocalSurface::Terminal,
    LocalSurface::Browser,
    LocalSurface::ComputerUse,
    LocalSurface::Git,
];

/// The capability advertisement of the full F1 surface topology: the
/// sorted, deduplicated union of every [`LocalSurface`] facet's
/// capabilities. This is everything a local environment can honestly
/// offer; anything beyond it cannot be sourced locally.
#[must_use]
pub fn local_surface_capabilities() -> Vec<CapabilityId> {
    let mut capabilities: Vec<CapabilityId> = LOCAL_SURFACES
        .iter()
        .flat_map(|surface| surface.capabilities())
        .collect();
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

/// Panics on the impossible: a frozen capability key that must always
/// parse (the keys come from the frozen vocabulary, never from input).
fn frozen_capability(key: &'static str) -> CapabilityId {
    match CapabilityId::parse(key) {
        Ok(value) => value,
        Err(error) => panic!("frozen capability key {key:?} must always parse: {error}"),
    }
}

/// The live surface of a local environment: the user's own machine,
/// wrapping a subset of the F1 surface topology. Stateless over its
/// durable [`EnvironmentDescriptor`], so the canonical `env_` identity
/// lives entirely in the descriptor and survives serialization.
///
/// Satisfies the SAME frozen [`Environment`] contract as every other
/// environment — local or remote, from any provider (kernel §9).
#[derive(Debug, Clone)]
pub struct LocalEnvironment {
    descriptor: EnvironmentDescriptor,
}

impl LocalEnvironment {
    /// Builds a local environment wrapping the given F1 surface facets.
    /// The capability advertisement is the sorted, deduplicated union of
    /// the facets' capabilities; identity is the caller-supplied
    /// canonical `env_` ID.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the name is empty or
    /// oversized, or the facet list is empty.
    pub fn new(
        id: EnvironmentId,
        name: &str,
        surfaces: &[LocalSurface],
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        if surfaces.is_empty() {
            return Err(ExecError::invalid(
                "a local environment wraps at least one F1 surface facet",
            ));
        }
        let mut facets = surfaces.to_vec();
        facets.sort();
        facets.dedup();
        let mut capabilities: Vec<CapabilityId> = facets
            .iter()
            .flat_map(|surface| surface.capabilities())
            .collect();
        capabilities.sort();
        capabilities.dedup();
        let descriptor = EnvironmentDescriptor::new(
            id,
            name,
            Locality::Local,
            Some(LOCAL_PROVIDER_KIND),
            capabilities,
            EnvironmentStatus::Available,
            None,
            created_by,
            created_at,
        )?;
        Ok(Self { descriptor })
    }

    /// Reconstructs the live local surface from a durable descriptor
    /// (the reload path after a serialization round-trip): the canonical
    /// `env_` identity is the descriptor's, unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`ExecError::Invalid`] when the descriptor is not a local
    /// F1-surface environment: wrong locality, a provider kind other
    /// than `local` (or absent), or capabilities outside the F1 topology
    /// — each named in the error.
    pub fn from_descriptor(descriptor: EnvironmentDescriptor) -> Result<Self, ExecError> {
        descriptor.validate()?;
        if descriptor.locality != Locality::Local {
            return Err(ExecError::invalid(format!(
                "a local F1-surface environment must have locality `local`, found `{}`",
                match descriptor.locality {
                    Locality::Local => "local",
                    Locality::Remote => "remote",
                }
            )));
        }
        match descriptor.provider_kind.as_deref() {
            Some(LOCAL_PROVIDER_KIND) | None => {}
            Some(other) => {
                return Err(ExecError::invalid(format!(
                    "a local F1-surface environment is sourced by provider kind `local`, \
                     found {other:?}"
                )));
            }
        }
        let topology = local_surface_capabilities();
        let foreign: Vec<String> = descriptor
            .capabilities
            .iter()
            .filter(|capability| !topology.contains(capability))
            .map(|capability| capability.as_str().to_owned())
            .collect();
        if !foreign.is_empty() {
            return Err(ExecError::invalid(format!(
                "the local F1 surface topology does not offer: {}",
                foreign.join(", ")
            )));
        }
        Ok(Self { descriptor })
    }

    /// The F1 surface facets this environment wraps (topology metadata):
    /// a facet is wrapped exactly when all of its capabilities are
    /// offered. In canonical facet order.
    #[must_use]
    pub fn surfaces(&self) -> Vec<LocalSurface> {
        LOCAL_SURFACES
            .iter()
            .copied()
            .filter(|surface| {
                surface
                    .capabilities()
                    .iter()
                    .all(|capability| self.descriptor.capabilities.contains(capability))
            })
            .collect()
    }
}

impl Environment for LocalEnvironment {
    fn environment_id(&self) -> &EnvironmentId {
        &self.descriptor.id
    }

    fn locality(&self) -> Locality {
        self.descriptor.locality
    }

    fn capabilities(&self) -> &[CapabilityId] {
        &self.descriptor.capabilities
    }

    fn status(&self) -> EnvironmentStatus {
        self.descriptor.status
    }

    fn descriptor(&self) -> EnvironmentDescriptor {
        self.descriptor.clone()
    }
}

/// The execution provider that wraps the user's own machine: sources
/// [`LocalEnvironment`]s from the four F1 surface facets behind the
/// frozen [`ExecutionProvider`] contract. Deterministic, in-memory, no
/// I/O — the wrapper family later local-machine integrations extend.
///
/// - `provider_kind` is [`LOCAL_PROVIDER_KIND`] (`local`).
/// - Only [`Locality::Local`] can be sourced; a remote spec is a named
///   contract error.
/// - No [`ProviderConnection`](crate::ProviderConnection): the local
///   machine needs no connected account and carries no credential state
///   at all (kernel §7 — references only, and here not even a reference).
#[derive(Debug, Clone, Default)]
pub struct LocalEnvironmentProvider {
    environments: BTreeMap<EnvironmentId, EnvironmentDescriptor>,
}

impl LocalEnvironmentProvider {
    /// An empty local provider that has sourced no environments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The live [`LocalEnvironment`] for a sourced canonical `env_` ID,
    /// if this provider sourced it.
    #[must_use]
    pub fn environment(&self, id: &EnvironmentId) -> Option<LocalEnvironment> {
        self.environments
            .get(id)
            .cloned()
            .and_then(|descriptor| LocalEnvironment::from_descriptor(descriptor).ok())
    }
}

impl ExecutionProvider for LocalEnvironmentProvider {
    fn provider_kind(&self) -> &str {
        LOCAL_PROVIDER_KIND
    }

    fn supported_localities(&self) -> &[Locality] {
        &[Locality::Local]
    }

    fn connection_id(&self) -> Option<&crate::ProviderConnectionId> {
        None
    }

    fn source_environment(
        &mut self,
        spec: &EnvironmentSpec,
    ) -> Result<EnvironmentDescriptor, ExecError> {
        spec.validate()?;
        if !self.supported_localities().contains(&spec.locality) {
            return Err(ExecError::invalid(format!(
                "provider `{}` cannot source {:?} environments",
                self.provider_kind(),
                spec.locality
            )));
        }
        // Honest sourcing (addendum §4): the F1 topology is everything a
        // local environment can offer — gaps are named, never silent.
        let topology = local_surface_capabilities();
        let missing = crate::runtime::missing_capabilities(&topology, &spec.capabilities);
        if !missing.is_empty() {
            let keys: Vec<String> = missing.iter().map(|key| key.as_str().to_owned()).collect();
            return Err(ExecError::invalid(format!(
                "the local F1 surface topology does not offer: {}",
                keys.join(", ")
            )));
        }
        if self.environments.len() >= MAX_ENVIRONMENTS_PER_PROVIDER {
            return Err(ExecError::invalid(format!(
                "an execution provider exposes at most {MAX_ENVIRONMENTS_PER_PROVIDER} environments"
            )));
        }
        // ID generation is the crate's only entropy source (kernel §7).
        let descriptor = EnvironmentDescriptor::new(
            EnvironmentId::generate(),
            &spec.name,
            spec.locality,
            Some(self.provider_kind()),
            spec.capabilities.clone(),
            EnvironmentStatus::Available,
            None,
            spec.requested_by.clone(),
            spec.requested_at,
        )?;
        self.environments
            .insert(descriptor.id.clone(), descriptor.clone());
        Ok(descriptor)
    }

    fn environments(&self) -> Vec<EnvironmentDescriptor> {
        self.environments.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{EntityKind, validate};

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    /// Unwraps the expected error side of a contract result (the crate's
    /// no-expect/no-unwrap test discipline).
    fn err<T, E: std::fmt::Display>(result: Result<T, E>, what: &str) -> E {
        match result {
            Err(error) => error,
            Ok(_) => panic!("{what}"),
        }
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::parse("2026-09-21T13:45:00Z"))
    }

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn parse_capability(key: &str) -> CapabilityId {
        ok(CapabilityId::parse(key))
    }

    fn local_spec(name: &str, capabilities: Vec<CapabilityId>) -> EnvironmentSpec {
        ok(EnvironmentSpec::new(
            name,
            Locality::Local,
            capabilities,
            test_actor(),
            test_timestamp(),
        ))
    }

    #[test]
    fn local_surface_topology_wraps_the_four_f1_surfaces() {
        let labels: Vec<&str> = LOCAL_SURFACES
            .iter()
            .map(|surface| surface.label())
            .collect();
        assert_eq!(labels, ["terminal", "browser", "computer-use", "git"]);
        // Facet labels serialize as kebab-case F1 surface names.
        assert_eq!(
            ok(serde_json::to_string(&LocalSurface::ComputerUse)),
            "\"computer-use\""
        );

        for surface in LOCAL_SURFACES {
            let capabilities = surface.capabilities();
            assert!(
                !capabilities.is_empty(),
                "{} must advertise",
                surface.label()
            );
            for capability in &capabilities {
                ok(capability.validate());
            }
            assert!(
                capabilities.windows(2).all(|pair| pair[0] < pair[1]),
                "{} capabilities must be sorted and deduplicated",
                surface.label()
            );
        }

        // The topology union: sorted, deduplicated, exactly the F1 keys.
        let topology = local_surface_capabilities();
        let expected = [
            "browser",
            "browser.input",
            "browser.navigation",
            "computer.screen",
            "desktop.gui",
            "filesystem.read",
            "filesystem.write",
            "git",
            "keyboard",
            "mouse",
            "terminal",
            "window",
        ];
        let keys: Vec<&str> = topology.iter().map(|key| key.as_str()).collect();
        assert_eq!(keys, expected);
        // The Computer Use compound capability is present (constitution).
        assert!(topology.contains(&parse_capability("desktop.gui")));
    }

    #[test]
    fn local_provider_sources_local_environments_only() {
        let mut provider = LocalEnvironmentProvider::new();
        assert_eq!(provider.provider_kind(), "local");
        assert_eq!(provider.supported_localities(), &[Locality::Local]);
        assert_eq!(provider.connection_id(), None);
        assert!(provider.environments().is_empty());

        // Remote is a named contract error, not a sourced environment.
        let remote_spec = ok(EnvironmentSpec::new(
            "Sandbox",
            Locality::Remote,
            local_surface_capabilities(),
            test_actor(),
            test_timestamp(),
        ));
        let error = err(
            provider.source_environment(&remote_spec),
            "remote locality must be rejected",
        );
        assert!(error.to_string().contains("cannot source"));

        // Sourcing the full F1 topology: descriptor at version 1, local,
        // provider kind `local`, no connection, spec attribution.
        let descriptor = ok(provider.source_environment(&local_spec(
            "Local workstation",
            local_surface_capabilities(),
        )));
        assert_eq!(descriptor.version, 1);
        assert_eq!(descriptor.locality, Locality::Local);
        assert_eq!(descriptor.provider_kind.as_deref(), Some("local"));
        assert_eq!(descriptor.status, EnvironmentStatus::Available);
        assert_eq!(descriptor.connection_id, None);
        assert_eq!(descriptor.created_by, test_actor());
        assert_eq!(
            ok(validate(descriptor.id.as_str())),
            EntityKind::Environment
        );

        // The live surface is reconstructable and agrees with the record.
        let environment = provider
            .environment(&descriptor.id)
            .unwrap_or_else(|| panic!("the sourced environment must resolve"));
        assert_eq!(environment.descriptor(), descriptor);
        assert_eq!(environment.surfaces(), LOCAL_SURFACES.to_vec());
        assert_eq!(provider.environments(), vec![descriptor]);
    }

    #[test]
    fn local_provider_names_capabilities_outside_the_f1_topology() {
        let mut provider = LocalEnvironmentProvider::new();
        let spec = local_spec(
            "Impossible workstation",
            vec![parse_capability("gpu"), parse_capability("ssh")],
        );
        let error = err(
            provider.source_environment(&spec),
            "gaps must be named, never silently sourced",
        );
        let message = error.to_string();
        assert!(
            message.contains("gpu"),
            "the missing key `gpu` must be named"
        );
        assert!(
            message.contains("ssh"),
            "the missing key `ssh` must be named"
        );
        assert!(provider.environments().is_empty());
    }

    #[test]
    fn local_environment_builds_from_facets_and_round_trips() {
        let id = ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4"));
        let environment = ok(LocalEnvironment::new(
            id.clone(),
            "Local workstation",
            &[
                LocalSurface::Terminal,
                LocalSurface::Browser,
                LocalSurface::ComputerUse,
                LocalSurface::Git,
            ],
            test_actor(),
            test_timestamp(),
        ));
        assert_eq!(environment.environment_id(), &id);
        assert_eq!(environment.locality(), Locality::Local);
        assert_eq!(environment.surfaces(), LOCAL_SURFACES.to_vec());

        // Duplicated facets collapse; the advertisement stays canonical.
        let duplicated = ok(LocalEnvironment::new(
            ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE5")),
            "Local terminal",
            &[LocalSurface::Terminal, LocalSurface::Terminal],
            test_actor(),
            test_timestamp(),
        ));
        assert_eq!(duplicated.capabilities(), &[parse_capability("terminal")]);
        assert_eq!(duplicated.surfaces(), vec![LocalSurface::Terminal]);

        // A facet-less environment is representable but not constructible
        // through the facet path: an empty facet list is a contract error.
        assert!(
            LocalEnvironment::new(
                ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE5")),
                "Nothing wrapped",
                &[],
                test_actor(),
                test_timestamp(),
            )
            .is_err()
        );

        // The descriptor is canonical serialized state: it round-trips,
        // and the live surface reconstructs from the reloaded record with
        // the SAME canonical identity.
        let descriptor = environment.descriptor();
        let serialized = ok(serde_json::to_string(&descriptor));
        let reloaded: EnvironmentDescriptor = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, descriptor);
        let reconstructed = ok(LocalEnvironment::from_descriptor(reloaded));
        assert_eq!(reconstructed.environment_id(), &id);
        assert_eq!(reconstructed.descriptor(), descriptor);
    }

    #[test]
    fn local_environment_from_descriptor_rejects_foreign_state() {
        let base = ok(EnvironmentDescriptor::new(
            ok(EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")),
            "Foreign environment",
            Locality::Local,
            Some("local"),
            vec![parse_capability("terminal")],
            EnvironmentStatus::Available,
            None,
            test_actor(),
            test_timestamp(),
        ));

        // Wrong locality: a remote record is not a local F1 surface.
        let mut remote = base.clone();
        remote.locality = Locality::Remote;
        assert!(LocalEnvironment::from_descriptor(remote).is_err());

        // Sourced by a different provider: not this wrapper's environment.
        let mut foreign_provider = base.clone();
        foreign_provider.provider_kind = Some("e2b".to_owned());
        assert!(LocalEnvironment::from_descriptor(foreign_provider).is_err());

        // A capability outside the F1 topology is named in the error.
        let mut foreign_capability = base;
        foreign_capability.capabilities =
            vec![parse_capability("ports"), parse_capability("terminal")];
        let error = err(
            LocalEnvironment::from_descriptor(foreign_capability),
            "foreign capabilities must be rejected",
        );
        assert!(error.to_string().contains("ports"));
    }

    #[test]
    fn local_environment_fixtures_round_trip_canonical() {
        for case in ["typical.json", "minimal.json"] {
            let content = read_fixture(&format!("local-environment/{case}"));
            let descriptor: EnvironmentDescriptor = ok(serde_json::from_str(&content));
            ok(descriptor.validate());
            let serialized = ok(serde_json::to_string_pretty(&descriptor));
            assert_eq!(
                serialized, content,
                "fixture local-environment/{case} must round-trip byte-for-byte"
            );
            let environment = ok(LocalEnvironment::from_descriptor(descriptor));
            if case == "typical.json" {
                assert_eq!(environment.surfaces(), LOCAL_SURFACES.to_vec());
            } else {
                assert_eq!(environment.surfaces(), vec![LocalSurface::Terminal]);
            }
        }

        // The invalid fixture parses as a descriptor but is not a local
        // F1-surface environment: the foreign capability is named.
        let content = read_fixture("local-environment/invalid-foreign-capability.json");
        let descriptor: EnvironmentDescriptor = ok(serde_json::from_str(&content));
        let error = err(
            LocalEnvironment::from_descriptor(descriptor),
            "foreign capabilities must be rejected",
        );
        assert!(error.to_string().contains("gpu"));

        // Canonical reads are strict: unknown fields are rejected.
        let typical = read_fixture("local-environment/typical.json");
        let unknown_field = typical.replace("\"v\": 1,", "\"v\": 1, \"model_id\": null,");
        assert!(serde_json::from_str::<EnvironmentDescriptor>(&unknown_field).is_err());

        // The committed fixtures are data too: credential material never
        // appears (addendum §3).
        assert_fixture_directory_credential_free("local-environment");
    }

    fn fixture_path(relative: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/w2")
            .join(relative)
    }

    fn read_fixture(relative: &str) -> String {
        let content = std::fs::read_to_string(fixture_path(relative))
            .unwrap_or_else(|error| panic!("could not read fixture {relative}: {error}"));
        // Canonical fixtures are committed with LF endings; normalize so
        // the byte-for-byte round-trip law is tested against the
        // canonical form, not a platform's line-ending translation.
        content.replace("\r\n", "\n").trim_end().to_owned()
    }

    fn assert_fixture_directory_credential_free(directory: &str) {
        for entry in std::fs::read_dir(fixture_path(directory))
            .unwrap_or_else(|error| panic!("could not list fixture dir {directory}: {error}"))
        {
            let entry = ok(entry);
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
                for marker in [
                    "sk-",
                    "Bearer ",
                    "api_key",
                    "apikey",
                    "password",
                    "passwd",
                    "client_secret",
                    "access_token",
                    "refresh_token",
                    "PRIVATE KEY",
                    "BEGIN RSA",
                    "xoxb-",
                    "ghp_",
                ] {
                    assert!(
                        !content.contains(marker),
                        "fixture {} must never contain credential material ({marker:?})",
                        path.display()
                    );
                }
            }
        }
    }
}
