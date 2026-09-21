//! Canonical entity identifiers and the frozen kind-prefix registry
//! (kernel §2).
//!
//! Every durable entity has a canonical ID `<kind>_<ULID>` where `<kind>` is
//! one of the frozen registry prefixes and `<ULID>` is 26 characters of
//! Crockford Base32. IDs are opaque: they never encode provider, surface,
//! location, model or client, and equality is string equality.
//!
//! This crate owns the typed identifiers for the exec entities
//! ([`EnvironmentId`], [`ModelId`], [`AgentId`],
//! [`ProviderConnectionId`]). Foreign crates' entities (for example `ws_`,
//! `task_` or `ctxsnap_`) are referenced through [`EntityKind`] +
//! [`validate`] without defining their types, per the kernel ownership
//! rules.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ulid;

/// Failure reasons for canonical identifier parsing. The frozen kernel
/// vectors (§2) pin one reason per invalid vector; the reasons are
/// identical across the F2 crates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdError {
    /// The identifier is the empty string.
    Empty,
    /// No separator is present and the string is not a known kind.
    MissingKind,
    /// No separator is present but the string is a known kind prefix.
    NoSeparator,
    /// A separator is present but the text before it is not a known kind.
    BadSeparator,
    /// The kind is a case variant of a known kind; kinds are lowercase.
    WrongKindCase,
    /// The ULID part does not have the canonical 26-character length.
    WrongLength {
        /// The observed character length.
        length: usize,
    },
    /// The ULID part contains a character outside the Crockford Base32
    /// alphabet (uppercase; no `I`, `L`, `O`, `U`; no lowercase letters).
    InvalidCharacter {
        /// The offending character.
        character: char,
        /// Its character index within the ULID part.
        position: usize,
    },
    /// The ULID part is valid but its kind prefix does not match the
    /// expected entity kind.
    KindMismatch {
        /// The prefix the caller expected.
        expected: &'static str,
        /// The prefix that was found.
        found: &'static str,
    },
}

impl fmt::Display for IdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("identifier must not be empty"),
            Self::MissingKind => formatter.write_str("identifier is missing its kind prefix"),
            Self::NoSeparator => formatter.write_str("identifier is missing the `_` separator"),
            Self::BadSeparator => {
                formatter.write_str("identifier does not start with a known kind prefix")
            }
            Self::WrongKindCase => formatter.write_str("identifier kind must be lowercase"),
            Self::WrongLength { length } => write!(
                formatter,
                "identifier ULID part must be 26 characters, found {length}"
            ),
            Self::InvalidCharacter {
                character,
                position,
            } => write!(
                formatter,
                "identifier ULID part contains {character:?} at position {position}, \
                 outside the Crockford Base32 alphabet"
            ),
            Self::KindMismatch { expected, found } => write!(
                formatter,
                "identifier kind mismatch: expected {expected:?}, found {found:?}"
            ),
        }
    }
}

impl Error for IdError {}

/// The entity kinds of the frozen kernel registry (§2), serialized as their
/// ID prefixes. This enum references foreign crates' entities by their
/// registered prefixes only; it does not define their types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EntityKind {
    /// Workspace (`ws`), owned by flauz-world.
    Workspace,
    /// Session (`sess`), owned by flauz-world.
    Session,
    /// Task (`task`), owned by flauz-world.
    Task,
    /// Artifact (`art`), owned by flauz-world.
    Artifact,
    /// Resource (`res`), owned by flauz-world.
    Resource,
    /// Event (`ev`), owned by flauz-world.
    Event,
    /// Observation (`obs`), owned by flauz-world.
    Observation,
    /// Claim (`claim`), owned by flauz-world.
    Claim,
    /// Evidence (`evd`), owned by flauz-world.
    Evidence,
    /// ResourceLease (`lease`), owned by flauz-world.
    ResourceLease,
    /// Procedure (`proc`), owned by flauz-world.
    Procedure,
    /// Environment (`env`), owned by flauz-exec.
    Environment,
    /// Model (`model`), owned by flauz-exec.
    Model,
    /// Agent (`agent`), owned by flauz-exec.
    Agent,
    /// ProviderConnection (`conn`), owned by flauz-exec.
    ProviderConnection,
    /// ContextSnapshot (`ctxsnap`), owned by flauz-context.
    ContextSnapshot,
    /// MemoryItem (`mem`), owned by flauz-context.
    MemoryItem,
}

impl EntityKind {
    /// The frozen ID prefix for this entity kind.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Workspace => "ws",
            Self::Session => "sess",
            Self::Task => "task",
            Self::Artifact => "art",
            Self::Resource => "res",
            Self::Event => "ev",
            Self::Observation => "obs",
            Self::Claim => "claim",
            Self::Evidence => "evd",
            Self::ResourceLease => "lease",
            Self::Procedure => "proc",
            Self::Environment => "env",
            Self::Model => "model",
            Self::Agent => "agent",
            Self::ProviderConnection => "conn",
            Self::ContextSnapshot => "ctxsnap",
            Self::MemoryItem => "mem",
        }
    }

    /// Looks up a kind by its frozen ID prefix (exact, case-sensitive).
    #[must_use]
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "ws" => Some(Self::Workspace),
            "sess" => Some(Self::Session),
            "task" => Some(Self::Task),
            "art" => Some(Self::Artifact),
            "res" => Some(Self::Resource),
            "ev" => Some(Self::Event),
            "obs" => Some(Self::Observation),
            "claim" => Some(Self::Claim),
            "evd" => Some(Self::Evidence),
            "lease" => Some(Self::ResourceLease),
            "proc" => Some(Self::Procedure),
            "env" => Some(Self::Environment),
            "model" => Some(Self::Model),
            "agent" => Some(Self::Agent),
            "conn" => Some(Self::ProviderConnection),
            "ctxsnap" => Some(Self::ContextSnapshot),
            "mem" => Some(Self::MemoryItem),
            _ => None,
        }
    }
}

impl fmt::Display for EntityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.prefix())
    }
}

impl Serialize for EntityKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.prefix())
    }
}

impl<'de> Deserialize<'de> for EntityKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let prefix = String::deserialize(deserializer)?;
        Self::from_prefix(&prefix)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown entity kind {prefix:?}")))
    }
}

/// Validates a canonical identifier of any registered kind and returns its
/// kind. This is the generic entry point used for foreign entities (for
/// example `ws_` or `ctxsnap_` IDs referenced from exec state).
pub fn validate(id: &str) -> Result<EntityKind, IdError> {
    if id.is_empty() {
        return Err(IdError::Empty);
    }

    let Some((kind, rest)) = id.split_once('_') else {
        if EntityKind::from_prefix(id).is_some() {
            return Err(IdError::NoSeparator);
        }
        if EntityKind::from_prefix(&id.to_ascii_lowercase()).is_some() {
            return Err(IdError::WrongKindCase);
        }
        return Err(IdError::MissingKind);
    };

    let kind = match EntityKind::from_prefix(kind) {
        Some(kind) => kind,
        None => {
            if kind.chars().any(char::is_uppercase)
                && EntityKind::from_prefix(&kind.to_ascii_lowercase()).is_some()
            {
                return Err(IdError::WrongKindCase);
            }
            return Err(IdError::BadSeparator);
        }
    };

    if rest.len() != ulid::ULID_LEN {
        return Err(IdError::WrongLength { length: rest.len() });
    }

    for (position, character) in rest.char_indices() {
        if !character.is_ascii() || !ulid::CROCKFORD_ALPHABET.contains(&(character as u8)) {
            return Err(IdError::InvalidCharacter {
                character,
                position,
            });
        }
    }

    Ok(kind)
}

macro_rules! canonical_id {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// The frozen kind prefix of this identifier kind.
            pub const PREFIX: &'static str = $prefix;

            /// Returns the canonical identifier string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Parses and validates a canonical identifier of this kind.
            pub fn parse(id: &str) -> Result<Self, IdError> {
                let kind = validate(id)?;
                if kind.prefix() != $prefix {
                    return Err(IdError::KindMismatch {
                        expected: $prefix,
                        found: kind.prefix(),
                    });
                }
                Ok(Self(id.to_owned()))
            }

            /// Generates a fresh identifier of this kind. ID generation is
            /// the crate's only entropy source (kernel §7).
            #[must_use]
            pub fn generate() -> Self {
                Self(format!(concat!($prefix, "_{}"), ulid::generate_ulid()))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(id: &str) -> Result<Self, Self::Err> {
                Self::parse(id)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(serde::de::Error::custom)
            }
        }
    };
}

canonical_id!(
    /// Canonical Environment identifier (`env_<ULID>`). An Environment is
    /// where execution happens; its ID is opaque and never encodes
    /// provider or locality.
    EnvironmentId,
    "env"
);
canonical_id!(
    /// Canonical Model identifier (`model_<ULID>`). A Model is
    /// intelligence; its ID never encodes provider or runtime.
    ModelId,
    "model"
);
canonical_id!(
    /// Canonical Agent identifier (`agent_<ULID>`). An Agent is distinct
    /// from an Environment (`env_`) and from a Model (`model_`).
    AgentId,
    "agent"
);
canonical_id!(
    /// Canonical ProviderConnection identifier (`conn_<ULID>`).
    ProviderConnectionId,
    "conn"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_parse_rejects_valid_ulid_of_wrong_kind() {
        let error = ModelId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4");
        assert_eq!(
            error.err(),
            Some(IdError::KindMismatch {
                expected: "model",
                found: "env"
            })
        );
    }

    #[test]
    fn typed_parse_accepts_its_own_kind() {
        let id = EnvironmentId::parse("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4");
        assert_eq!(
            id.as_ref().map(|id| id.as_str()),
            Ok("env_01J8ZQ5V8K3T2B7N6X4R9DQPE4")
        );
    }

    #[test]
    fn generated_ids_carry_their_kind_prefix() {
        assert!(EnvironmentId::generate().as_str().starts_with("env_"));
        assert!(ModelId::generate().as_str().starts_with("model_"));
        assert!(AgentId::generate().as_str().starts_with("agent_"));
        assert!(
            ProviderConnectionId::generate()
                .as_str()
                .starts_with("conn_")
        );
    }

    #[test]
    fn entity_kind_prefixes_round_trip() {
        for kind in [
            EntityKind::Workspace,
            EntityKind::Session,
            EntityKind::Task,
            EntityKind::Artifact,
            EntityKind::Resource,
            EntityKind::Event,
            EntityKind::Observation,
            EntityKind::Claim,
            EntityKind::Evidence,
            EntityKind::ResourceLease,
            EntityKind::Procedure,
            EntityKind::Environment,
            EntityKind::Model,
            EntityKind::Agent,
            EntityKind::ProviderConnection,
            EntityKind::ContextSnapshot,
            EntityKind::MemoryItem,
        ] {
            assert_eq!(EntityKind::from_prefix(kind.prefix()), Some(kind));
        }
        assert_eq!(EntityKind::from_prefix("nope"), None);
    }
}
