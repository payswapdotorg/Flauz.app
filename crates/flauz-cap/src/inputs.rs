//! The resolver inputs: capability advertisements and the permission +
//! policy decisions, taken as **data** (addendum §4).
//!
//! The crate never imports `flauz-world` or `flauz-exec`: the model's
//! capability metadata, the runtime advertisement and the environment
//! surfaces cross the seam as frozen capability-key strings, and the
//! permission/policy state arrives as call-side decisions. There is no
//! permission system and no policy engine here — those are future waves;
//! this crate only resolves over the decisions it is given.
//!
//! The environment input distinguishes "no environment attached" (`None`)
//! from "environments attached, none offering the surface" (`Some(keys)`),
//! because the named gap and the honest unlock path differ: the first asks
//! for an environment, the second for one that offers the capability.

use serde::{Deserialize, Serialize};

use crate::dimension::Dimension;
use crate::key::CapabilityKey;
use crate::{
    CapError, CapVersion, MAX_EXPLANATION_BYTES, ensure_capability_list, ensure_non_empty,
    ensure_str_bound,
};

/// The call-side permission decision for one capability on one task. This
/// is an INPUT, not a permission system: whatever grants or denies
/// permissions in a future wave produces this decision and hands it to the
/// resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum PermissionDecision {
    /// Permission to use the capability has been granted for the task.
    Granted,
    /// Permission to use the capability has been denied or was never
    /// requested. `detail` is an optional bounded, human-readable
    /// explanation from the deciding surface.
    Denied {
        /// Optional explanation from the deciding surface.
        detail: Option<String>,
    },
}

impl PermissionDecision {
    /// Whether the decision admits the capability.
    #[must_use]
    pub const fn admits(&self) -> bool {
        matches!(self, Self::Granted)
    }

    /// Validates the decision's bounds.
    pub fn validate(&self) -> Result<(), CapError> {
        if let Self::Denied {
            detail: Some(detail),
        } = self
        {
            ensure_non_empty("permission detail", detail)?;
            ensure_str_bound("permission detail", detail, MAX_EXPLANATION_BYTES)?;
        }
        Ok(())
    }
}

/// The call-side workspace-policy decision for one capability on one task.
/// This is an INPUT, not a policy engine: future policy surfaces produce
/// this decision and hand it to the resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum PolicyDecision {
    /// Workspace policy allows the capability for the task.
    Allowed,
    /// Workspace policy does not allow the capability for the task.
    /// `detail` is an optional bounded, human-readable explanation from
    /// the deciding surface.
    Restricted {
        /// Optional explanation from the deciding surface.
        detail: Option<String>,
    },
}

impl PolicyDecision {
    /// Whether the decision admits the capability.
    #[must_use]
    pub const fn admits(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Validates the decision's bounds.
    pub fn validate(&self) -> Result<(), CapError> {
        if let Self::Restricted {
            detail: Some(detail),
        } = self
        {
            ensure_non_empty("policy detail", detail)?;
            ensure_str_bound("policy detail", detail, MAX_EXPLANATION_BYTES)?;
        }
        Ok(())
    }
}

/// The inputs to one capability resolution (addendum §4): the frozen
/// interfaces as data. Every field is call-side state — the resolver
/// computes over them and never reaches beyond them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionInputs {
    /// Contract schema version (`"v": 1`).
    pub v: CapVersion,
    /// The capabilities the model in use offers (model capability
    /// metadata; sorted, deduplicated).
    pub model_advertised: Vec<CapabilityKey>,
    /// The capabilities the runtime in use advertises (sorted,
    /// deduplicated).
    pub runtime_advertised: Vec<CapabilityKey>,
    /// The capabilities offered by the environments attached to the task —
    /// the union of their surfaces (sorted, deduplicated). `None` means no
    /// environment is attached, which is a different named gap from an
    /// attached environment that lacks the surface.
    pub environment_advertised: Option<Vec<CapabilityKey>>,
    /// The permission decision for the task.
    pub permission: PermissionDecision,
    /// The workspace-policy decision for the task.
    pub policy: PolicyDecision,
}

impl ResolutionInputs {
    /// Builds resolver inputs, validating every advertisement list and
    /// decision against the canonical rules.
    pub fn new(
        model_advertised: Vec<CapabilityKey>,
        runtime_advertised: Vec<CapabilityKey>,
        environment_advertised: Option<Vec<CapabilityKey>>,
        permission: PermissionDecision,
        policy: PolicyDecision,
    ) -> Result<Self, CapError> {
        ensure_capability_list("model advertisement", &model_advertised)?;
        ensure_capability_list("runtime advertisement", &runtime_advertised)?;
        if let Some(environment) = environment_advertised.as_deref() {
            ensure_capability_list("environment advertisement", environment)?;
        }
        permission.validate()?;
        policy.validate()?;
        Ok(Self {
            v: CapVersion,
            model_advertised,
            runtime_advertised,
            environment_advertised,
            permission,
            policy,
        })
    }

    /// Validates the inputs against the canonical rules.
    pub fn validate(&self) -> Result<(), CapError> {
        Self::new(
            self.model_advertised.clone(),
            self.runtime_advertised.clone(),
            self.environment_advertised.clone(),
            self.permission.clone(),
            self.policy.clone(),
        )?;
        Ok(())
    }

    /// Whether `dimension` admits `capability` under these inputs.
    ///
    /// The environment dimension admits only when an environment is
    /// attached AND its surfaces include the capability; `None` never
    /// admits.
    #[must_use]
    pub fn admits(&self, dimension: Dimension, capability: &CapabilityKey) -> bool {
        match dimension {
            Dimension::Model => self.model_advertised.contains(capability),
            Dimension::Runtime => self.runtime_advertised.contains(capability),
            Dimension::Environment => self
                .environment_advertised
                .as_ref()
                .is_some_and(|advertised| advertised.contains(capability)),
            Dimension::Permissions => self.permission.admits(),
            Dimension::WorkspacePolicy => self.policy.admits(),
        }
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

    fn key(value: &str) -> CapabilityKey {
        ok(CapabilityKey::parse(value))
    }

    #[test]
    fn decisions_serialize_internally_tagged() {
        let granted = PermissionDecision::Granted;
        assert_eq!(
            ok(serde_json::to_string(&granted)),
            "{\"kind\":\"granted\"}"
        );
        let denied = PermissionDecision::Denied {
            detail: Some("no approval has been requested yet".to_owned()),
        };
        let serialized = ok(serde_json::to_string(&denied));
        assert_eq!(
            serialized,
            concat!(
                "{\"kind\":\"denied\",",
                "\"detail\":\"no approval has been requested yet\"}"
            )
        );
        let reloaded: PermissionDecision = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, denied);

        let allowed = PolicyDecision::Allowed;
        assert_eq!(
            ok(serde_json::to_string(&allowed)),
            "{\"kind\":\"allowed\"}"
        );
        let restricted = PolicyDecision::Restricted { detail: None };
        assert_eq!(
            ok(serde_json::to_string(&restricted)),
            "{\"kind\":\"restricted\",\"detail\":null}"
        );
        assert!(
            serde_json::from_str::<PolicyDecision>("{\"kind\":\"maybe\"}").is_err(),
            "unknown decision kinds are rejected"
        );
        assert!(
            serde_json::from_str::<PermissionDecision>(
                &serialized.replace("\"detail\":", "\"reason\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn inputs_reject_unsorted_and_duplicated_advertisements() {
        assert!(
            ResolutionInputs::new(
                vec![key("terminal"), key("browser.input")],
                vec![],
                None,
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            )
            .is_err(),
            "advertisements must be sorted"
        );
        assert!(
            ResolutionInputs::new(
                vec![],
                vec![],
                Some(vec![key("terminal"), key("terminal")]),
                PermissionDecision::Granted,
                PolicyDecision::Allowed,
            )
            .is_err(),
            "advertisements must be deduplicated"
        );
        assert!(
            ResolutionInputs::new(
                vec![],
                vec![],
                None,
                PermissionDecision::Denied {
                    detail: Some("".to_owned())
                },
                PolicyDecision::Allowed,
            )
            .is_err(),
            "decision details must not be empty"
        );
    }

    #[test]
    fn inputs_round_trip_with_the_environment_distinction() {
        let attached = ok(ResolutionInputs::new(
            vec![key("browser.input"), key("vision")],
            vec![key("browser.input"), key("terminal")],
            Some(vec![key("filesystem.read"), key("terminal")]),
            PermissionDecision::Granted,
            PolicyDecision::Allowed,
        ));
        let serialized = ok(serde_json::to_string(&attached));
        let reloaded: ResolutionInputs = ok(serde_json::from_str(&serialized));
        assert_eq!(reloaded, attached);
        assert!(serialized.contains("\"environment_advertised\":["));
        assert!(attached.admits(Dimension::Environment, &key("terminal")));
        assert!(!attached.admits(Dimension::Environment, &key("browser.input")));

        let detached = ok(ResolutionInputs::new(
            vec![],
            vec![],
            None,
            PermissionDecision::Granted,
            PolicyDecision::Allowed,
        ));
        let serialized = ok(serde_json::to_string(&detached));
        assert!(serialized.contains("\"environment_advertised\":null"));
        assert!(
            !detached.admits(Dimension::Environment, &key("terminal")),
            "no attached environment never admits"
        );
        assert!(
            serde_json::from_str::<ResolutionInputs>(
                &serialized.replace("\"permission\":", "\"permissions\":")
            )
            .is_err(),
            "unknown fields are rejected"
        );
    }
}
