//! The [`ProviderConnection`] entity and [`SecretRef`] — credential
//! references only (kernel §7).
//!
//! Users can connect their own provider accounts. A connection carries a
//! provider kind, an account label and an **opaque** `flausec_...` secret
//! reference — and nothing else. Credential **material** never appears in
//! any contract type, fixture, log line or serialized state of this crate:
//! secrets live behind the secret-store boundary that mints `flausec_`
//! references.
//!
//! The connection type is deliberately minimal: no tokens, no endpoints, no
//! scopes, no raw connection strings. Provider quota belongs to the
//! user-owned connection where supported.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ids::ProviderConnectionId;
use crate::refs::ActorRef;
use crate::time::Timestamp;
use crate::{
    ContractVersion, ExecError, MAX_NAME_BYTES, MAX_REFERENCE_BYTES, ensure_kind_label,
    ensure_non_empty, ensure_str_bound,
};

/// The marker list of credential-material shapes that must never appear in
/// or behind a secret reference (shared by parsing and the conformance
/// scans).
pub(crate) const CREDENTIAL_MARKERS: &[&str] = &[
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
];

/// An opaque reference to a secret held by the secret store: a
/// `flausec_...` string (kernel §7). The reference is a bounded, printable
/// token; it is never credential material itself, and references that look
/// like raw credentials are rejected outright so the type cannot smuggle
/// material past the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SecretRef(String);

impl SecretRef {
    /// The mandatory prefix of every secret reference.
    pub const PREFIX: &'static str = "flausec_";

    /// Parses and validates a secret reference: mandatory `flausec_`
    /// prefix, non-empty bounded remainder of printable ASCII tokens, and
    /// no credential-material markers anywhere in the reference.
    pub fn parse(value: &str) -> Result<Self, ExecError> {
        let Some(rest) = value.strip_prefix(Self::PREFIX) else {
            return Err(ExecError::invalid(format!(
                "secret reference must start with {:?}, found {value:?}",
                Self::PREFIX
            )));
        };
        ensure_non_empty("secret reference", rest)?;
        ensure_str_bound("secret reference", value, MAX_REFERENCE_BYTES)?;
        if !rest.chars().all(|character| character.is_ascii_graphic()) {
            return Err(ExecError::invalid(
                "secret reference must be a printable token without whitespace",
            ));
        }
        for marker in CREDENTIAL_MARKERS {
            if value.contains(marker) {
                return Err(ExecError::invalid(format!(
                    "secret reference must be an opaque token, not credential material \
                     (contains {marker:?})"
                )));
            }
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the secret reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the secret reference.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::parse(&self.0)?;
        Ok(())
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SecretRef {
    type Err = ExecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for SecretRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SecretRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A user-owned connection to a provider account: provider kind, account
/// label and an opaque [`SecretRef`] **only** (kernel §7). Credential
/// material never appears here — or anywhere else in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderConnection {
    /// Contract schema version (`"v": 1`).
    pub v: ContractVersion,
    /// Canonical connection ID (`conn_<ULID>`).
    pub id: ProviderConnectionId,
    /// Durable entity version, starting at 1, +1 per durable mutation.
    pub version: u64,
    /// The neutral kind label of the provider this account connects to
    /// (for example `openai`, `e2b`).
    pub provider_kind: String,
    /// A user-facing label for the connected account (for example "Work
    /// account").
    pub account_label: String,
    /// The opaque secret reference (`flausec_...`). Credential material
    /// lives behind the secret store, never in this type.
    pub secret_ref: SecretRef,
    /// The actor that connected the account.
    pub created_by: ActorRef,
    /// Connection timestamp (caller-supplied).
    pub created_at: Timestamp,
}

impl ProviderConnection {
    /// Builds a new provider connection at version 1. Accepts exactly a
    /// provider kind, an account label and a secret reference — there is
    /// no way to construct a connection carrying credential material.
    pub fn new(
        id: ProviderConnectionId,
        provider_kind: &str,
        account_label: &str,
        secret_ref: SecretRef,
        created_by: ActorRef,
        created_at: Timestamp,
    ) -> Result<Self, ExecError> {
        ensure_kind_label("connection provider kind", provider_kind)?;
        ensure_non_empty("connection account label", account_label)?;
        ensure_str_bound("connection account label", account_label, MAX_NAME_BYTES)?;
        secret_ref.validate()?;
        Ok(Self {
            v: ContractVersion,
            id,
            version: 1,
            provider_kind: provider_kind.to_owned(),
            account_label: account_label.to_owned(),
            secret_ref,
            created_by,
            created_at,
        })
    }

    /// Validates the connection record.
    pub fn validate(&self) -> Result<(), ExecError> {
        Self::new(
            self.id.clone(),
            &self.provider_kind,
            &self.account_label,
            self.secret_ref.clone(),
            self.created_by.clone(),
            self.created_at,
        )?;
        if self.version == 0 {
            return Err(ExecError::invalid(
                "provider connection version must be at least 1",
            ));
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

    fn test_actor() -> ActorRef {
        ok(ActorRef::user("alice"))
    }

    fn test_timestamp() -> Timestamp {
        ok(Timestamp::from_unix_seconds(1_789_998_300))
    }

    #[test]
    fn secret_refs_are_opaque_flausec_tokens() {
        assert!(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34").is_ok());
        assert!(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34:work").is_ok());
        assert!(SecretRef::parse("flausec_").is_err());
        assert!(SecretRef::parse("sk-proj-abcdefgh1234").is_err());
        assert!(SecretRef::parse("flausec_sk-proj-abcdefgh1234").is_err());
        assert!(SecretRef::parse("flausec_token with space").is_err());
        assert!(SecretRef::parse("flausec_ghp_0123456789abcdef").is_err());
        assert!(SecretRef::parse("").is_err());
        let reference = ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34"));
        let serialized = ok(serde_json::to_string(&reference));
        assert_eq!(serialized, "\"flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34\"");
        let parsed: SecretRef = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, reference);
    }

    #[test]
    fn connection_serializes_references_only() {
        let connection = ok(ProviderConnection::new(
            ok(ProviderConnectionId::parse(
                "conn_01J8ZQ5V8K3T2B7N6X4R9DQP34",
            )),
            "e2b",
            "Work sandbox account",
            ok(SecretRef::parse("flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34")),
            test_actor(),
            test_timestamp(),
        ));
        let serialized = ok(serde_json::to_string(&connection));
        assert_eq!(
            serialized,
            concat!(
                "{\"v\":1,\"id\":\"conn_01J8ZQ5V8K3T2B7N6X4R9DQP34\",\"version\":1,",
                "\"provider_kind\":\"e2b\",\"account_label\":\"Work sandbox account\",",
                "\"secret_ref\":\"flausec_01J8ZQ5V8K3T2B7N6X4R9DQP34\",",
                "\"created_by\":{\"kind\":\"user\",\"id\":\"alice\"},",
                "\"created_at\":\"2026-09-21T13:45:00Z\"}"
            )
        );
        let parsed: ProviderConnection = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, connection);
        ok(connection.validate());
        assert!(
            serde_json::from_str::<ProviderConnection>(&serialized.replace("\"v\":1,", ""))
                .is_err()
        );
    }
}
