//! The admission dimensions of the frozen intersection (addendum §4).
//!
//! A capability is available iff every one of these dimensions admits it:
//!
//! ```text
//! model ∩ runtime ∩ environment ∩ permissions ∩ workspace policy
//! ```
//!
//! The five dimensions are the J-04 user-facing vocabulary ("model lacks
//! it / runtime doesn't advertise it / environment has no such surface /
//! permission denied / workspace policy"), so the enum carries both the
//! canonical serialized name and the user-facing label.

use serde::{Deserialize, Serialize};

/// One admission dimension of the capability intersection (addendum §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// The model in use offers the capability (model capability metadata).
    Model,
    /// The runtime in use advertises the capability (runtime
    /// advertisement).
    Runtime,
    /// An attached environment offers the capability (environment
    /// surfaces).
    Environment,
    /// Permission to use the capability has been granted for the task.
    Permissions,
    /// Workspace policy allows the capability for the task.
    WorkspacePolicy,
}

impl Dimension {
    /// Every admission dimension, in canonical order. Resolution records
    /// list their per-dimension admissions — and their unlock paths — in
    /// exactly this order.
    pub const ALL: [Self; crate::DIMENSIONS] = [
        Self::Model,
        Self::Runtime,
        Self::Environment,
        Self::Permissions,
        Self::WorkspacePolicy,
    ];

    /// The canonical serialized name (snake_case, kernel §4).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Runtime => "runtime",
            Self::Environment => "environment",
            Self::Permissions => "permissions",
            Self::WorkspacePolicy => "workspace_policy",
        }
    }

    /// The user-facing dimension label (J-04 vocabulary). UI copy uses
    /// these words; the serialized names are canonical state, not display
    /// copy.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Runtime => "Runtime",
            Self::Environment => "Environment",
            Self::Permissions => "Permissions",
            Self::WorkspacePolicy => "Workspace policy",
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

    #[test]
    fn dimensions_serialize_to_their_snake_case_names() {
        for dimension in Dimension::ALL {
            let serialized = ok(serde_json::to_string(&dimension));
            assert_eq!(serialized, format!("\"{}\"", dimension.name()));
            let reloaded: Dimension = ok(serde_json::from_str(&serialized));
            assert_eq!(reloaded, dimension);
        }
        assert_eq!(Dimension::WorkspacePolicy.name(), "workspace_policy");
        assert!(
            serde_json::from_str::<Dimension>("\"workspace policy\"").is_err(),
            "unknown names are rejected"
        );
    }

    #[test]
    fn the_canonical_order_is_the_intersection_order() {
        let names: Vec<&str> = Dimension::ALL.iter().map(|d| d.name()).collect();
        assert_eq!(
            names,
            vec![
                "model",
                "runtime",
                "environment",
                "permissions",
                "workspace_policy"
            ]
        );
    }
}
