//! Frozen reference formats re-validated locally (the flauz-cap pattern):
//! capability keys, identifier grammars, chords and frame fingerprints cross
//! the seam as frozen format strings, never as cargo dependencies.
//!
//! The capability-key grammar is owned by `flauz-exec`'s `CapabilityId`
//! (kernel §2); this crate re-validates the same frozen grammar locally so a
//! key serialized by either crate has byte-identical meaning on the wire.
//! The identifier grammars here are owned by this crate: named kebab-case
//! identifiers for journeys, steps, probes, moments, anchors, surfaces, runs
//! and findings; product-journey references (`J-01`..); canonical key
//! chords; and the frame fingerprint.

use std::fmt;
use std::str::FromStr;

use serde::{Deserializer, Serializer};

use crate::{LabError, MAX_NAME_BYTES, ensure_name, ensure_str_bound};

/// Length of a frame fingerprint (FNV-1a 64-bit, 16 lowercase hex chars).
const DIGEST_LEN: usize = 16;

/// The hexadecimal alphabet of frame fingerprints.
const HEX_ALPHABET: &[u8; 16] = b"0123456789abcdef";

/// The named keys of the canonical chord grammar (the F1 binding vocabulary
/// the lab scenes exercise; single printable characters parse as
/// themselves).
pub const NAMED_KEYS: [&str; 14] = [
    "backspace",
    "delete",
    "down",
    "end",
    "escape",
    "home",
    "left",
    "page_down",
    "page_up",
    "return",
    "right",
    "space",
    "tab",
    "up",
];

/// The canonical modifier order of the chord grammar (the F1 binding
/// convention, not alphabetical): a chord's modifiers appear exactly once
/// each, in this order, before the key — `ctrl+alt+shift+6`.
pub const MODIFIERS: [&str; 4] = ["ctrl", "alt", "shift", "cmd"];

/// Validates a kebab-case identifier: the first segment is
/// `[a-z][a-z0-9]*`; subsequent segments are `[a-z0-9][a-z0-9]*` (so
/// ordinal segments like `01` and `0001` parse — `step-01`,
/// `frame-0001`), joined by single hyphens.
fn validate_kebab(field: &'static str, value: &str) -> Result<(), LabError> {
    ensure_name(field, value)?;
    for (index, segment) in value.split('-').enumerate() {
        let mut characters = segment.chars();
        let first = characters.next();
        let first_valid = if index == 0 {
            first.is_some_and(|c| c.is_ascii_lowercase())
        } else {
            first.is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        };
        let rest_valid = characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        if !first_valid || !rest_valid {
            return Err(LabError::invalid(format!(
                "{field} {value:?} segments must be kebab-case (first segment starts with a \
                 letter; later segments may start with a digit)"
            )));
        }
    }
    Ok(())
}

macro_rules! kebab_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Parses and validates the identifier against the kebab-case
            /// grammar.
            ///
            /// # Errors
            ///
            /// Returns [`LabError`] when the value is empty, oversized or
            /// outside the grammar.
            pub fn parse(value: &str) -> Result<Self, LabError> {
                validate_kebab(stringify!($name), value)?;
                Ok(Self(value.to_owned()))
            }

            /// Returns the identifier string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Validates the identifier against the grammar.
            ///
            /// # Errors
            ///
            /// Returns [`LabError`] when the value is outside the grammar.
            pub fn validate(&self) -> Result<(), LabError> {
                validate_kebab(stringify!($name), &self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::parse(&text).map_err(serde::de::Error::custom)
            }
        }
    };
}

kebab_id!(
    /// A lab journey's identifier (for example `shell-discovery`).
    JourneyId
);
kebab_id!(
    /// One journey step's identifier (for example `open-picker`).
    StepId
);
kebab_id!(
    /// One journey probe's identifier (for example `assert-picker-open`).
    ProbeId
);
kebab_id!(
    /// A named capture moment (for example `picker-open`) — frames are
    /// captured at named moments, never at wall-clock times.
    FrameMoment
);
kebab_id!(
    /// A named UI anchor (for example `model-picker-panel`) — clicks target
    /// NAMED anchors; adapters resolve them provider-side.
    AnchorId
);
kebab_id!(
    /// A named UI surface (for example `command-palette`).
    SurfaceId
);
kebab_id!(
    /// An evidence run's identifier (for example
    /// `flauz-lab-local-shell-discovery`).
    RunId
);
kebab_id!(
    /// A named finding's identifier (for example `pty-focus-transfer`) —
    /// records reference findings by id, never by prose.
    FindingId
);
kebab_id!(
    /// A frame's run-scoped reference (for example `frame-0001`) — frames
    /// are referenced, never inlined as pixels.
    FrameRef
);
kebab_id!(
    /// A named anchor state (for example `open-empty`, `chat-two`).
    StateKey
);

/// A product-journey reference from the frozen PRODUCT-UX-JOURNEYS
/// registry: `J-` followed by two digits (for example `J-01`). The lab
/// journey maps to product journeys as data; the registry itself is owned
/// by the journeys document.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JourneyRef(String);

impl JourneyRef {
    /// Parses and validates a product-journey reference.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is not `J-` + two digits.
    pub fn parse(value: &str) -> Result<Self, LabError> {
        let valid = value.len() == 4
            && value.starts_with("J-")
            && value.as_bytes()[2].is_ascii_digit()
            && value.as_bytes()[3].is_ascii_digit();
        if !valid {
            return Err(LabError::invalid(format!(
                "product-journey reference {value:?} must be `J-` + two digits (e.g. `J-01`)"
            )));
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the product-journey reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the product-journey reference.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is outside the grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::parse(&self.0).map(|_| ())
    }
}

impl fmt::Display for JourneyRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl serde::Serialize for JourneyRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for JourneyRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A namespaced capability key (kernel §2): segments of
/// `[a-z][a-z0-9_]*` separated by dots, for example `terminal` or
/// `computer.screen`. The grammar is owned by `flauz-exec`'s
/// `CapabilityId`; this crate re-validates the same frozen format locally
/// (the flauz-cap pattern) so a key serialized by either crate has
/// byte-identical meaning on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityKey(String);

impl CapabilityKey {
    /// Parses and validates a capability key against the frozen grammar.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when a segment is outside
    /// `[a-z][a-z0-9_]*` or the key is empty or oversized.
    pub fn parse(value: &str) -> Result<Self, LabError> {
        if value.is_empty() {
            return Err(LabError::invalid("capability key must not be empty"));
        }
        for segment in value.split('.') {
            let mut characters = segment.chars();
            let valid = characters
                .next()
                .is_some_and(|first| first.is_ascii_lowercase())
                && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if !valid {
                return Err(LabError::invalid(format!(
                    "capability key {value:?} segments must be `[a-z][a-z0-9_]*`"
                )));
            }
        }
        ensure_str_bound("capability key", value, MAX_NAME_BYTES)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the capability key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the capability key.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is outside the grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::parse(&self.0).map(|_| ())
    }
}

impl fmt::Display for CapabilityKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for CapabilityKey {
    type Err = LabError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl serde::Serialize for CapabilityKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for CapabilityKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A canonical key chord (the F1 binding vocabulary as data): zero or more
/// modifiers from [`MODIFIERS`] in canonical order, then one key — a single
/// ASCII printable character (`6`, `^`, `]`) or one of [`NAMED_KEYS`]
/// (`escape`, `return`, `tab`, …). Serialized as the lowercase `+`-joined
/// form, for example `ctrl+alt+shift+6`, `alt+^`, `ctrl+page_down`,
/// `escape`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Chord(String);

impl Chord {
    /// Parses and validates a chord against the canonical grammar.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when a segment is unknown, a modifier repeats,
    /// the modifiers are out of canonical order, or the key is neither a
    /// single printable character nor a named key.
    pub fn parse(value: &str) -> Result<Self, LabError> {
        ensure_str_bound("chord", value, MAX_NAME_BYTES)?;
        let segments: Vec<&str> = value.split('+').collect();
        let (modifiers, key) = segments.split_at(segments.len() - 1);
        let key = key[0];
        if key.is_empty() {
            return Err(LabError::invalid("chord key must not be empty"));
        }
        let named = NAMED_KEYS.contains(&key);
        let printable = key.len() == 1 && key.as_bytes()[0].is_ascii_graphic();
        if !named && !printable {
            return Err(LabError::invalid(format!(
                "chord key {key:?} must be a named key or a single printable character"
            )));
        }
        if !modifiers.is_empty() {
            let mut seen: Vec<&str> = Vec::with_capacity(modifiers.len());
            for modifier in modifiers {
                if !MODIFIERS.contains(modifier) {
                    return Err(LabError::invalid(format!(
                        "chord modifier {modifier:?} is not one of ctrl/alt/shift/cmd"
                    )));
                }
                if seen.contains(modifier) {
                    return Err(LabError::invalid(format!(
                        "chord modifier {modifier:?} repeats"
                    )));
                }
                seen.push(modifier);
            }
            let canonical: Vec<&str> = MODIFIERS
                .iter()
                .filter(|modifier| seen.contains(modifier))
                .copied()
                .collect();
            if canonical != seen {
                return Err(LabError::invalid(format!(
                    "chord modifiers must appear in canonical order ({value:?})"
                )));
            }
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the canonical chord string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the chord against the grammar.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is outside the grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::parse(&self.0).map(|_| ())
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl serde::Serialize for Chord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Chord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A provider-local frame fingerprint: 16 lowercase hex characters (FNV-1a
/// 64-bit over the normalized frame content). NOT cryptographic and NOT
/// cross-provider comparable — it pins renderer determinism within one
/// provider's runs (the lab's md5 frame-diff reasoning); the comparator
/// ignores it by design.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameDigest(String);

impl FrameDigest {
    /// Formats a fingerprint from a 64-bit FNV-1a hash.
    #[must_use]
    pub fn from_fnv1a(hash: u64) -> Self {
        Self(format!("{hash:016x}"))
    }

    /// Parses and validates a frame fingerprint (16 lowercase hex chars).
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is not 16 lowercase hex
    /// characters.
    pub fn parse(value: &str) -> Result<Self, LabError> {
        if value.len() != DIGEST_LEN {
            return Err(LabError::invalid(format!(
                "frame digest must be {DIGEST_LEN} lowercase hex characters, found {}",
                value.len()
            )));
        }
        if !value.bytes().all(|byte| HEX_ALPHABET.contains(&byte)) {
            return Err(LabError::invalid(format!(
                "frame digest {value:?} contains characters outside lowercase hex"
            )));
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the fingerprint string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FrameDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl serde::Serialize for FrameDigest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for FrameDigest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A reference to an archived VLM read result (operator-side; for example
/// `vlm-frame-0001.json`). A bounded archive token — never the read
/// content, never credential material. The comparator never compares read
/// references: archive locations are provider-local.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VlmReadRef(String);

impl VlmReadRef {
    /// Parses and validates a read-result reference: non-empty, bounded,
    /// lowercase alphanumerics with hyphens and dots.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is empty, oversized or contains
    /// characters outside `[a-z0-9.-]`.
    pub fn parse(value: &str) -> Result<Self, LabError> {
        ensure_name("vlm read reference", value)?;
        if !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        }) {
            return Err(LabError::invalid(format!(
                "vlm read reference {value:?} must be lowercase alphanumerics, hyphens and dots"
            )));
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the read-result reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the read-result reference.
    ///
    /// # Errors
    ///
    /// Returns [`LabError`] when the value is outside the grammar.
    pub fn validate(&self) -> Result<(), LabError> {
        Self::parse(&self.0).map(|_| ())
    }
}

impl fmt::Display for VlmReadRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl serde::Serialize for VlmReadRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for VlmReadRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// The FNV-1a 64-bit hash (deterministic, dependency-free) used for frame
/// fingerprints: provider-local determinism pinning only, never a
/// cross-provider comparison and never a cryptographic claim.
#[must_use]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
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
    fn kebab_identifiers_follow_the_grammar() {
        for valid in [
            "shell-discovery",
            "a11y-chord-ladder",
            "step-01",
            "frame-0001",
            "picker-open",
            "model-picker-panel",
            "pty-focus-transfer",
            "f",
        ] {
            assert_eq!(ok(JourneyId::parse(valid)).as_str(), valid);
        }
        for invalid in [
            "",
            "Shell-Discovery",
            "-leading",
            "trailing-",
            "double--hyphen",
            "under_score",
            "space in id",
            "01-pure-digits-first",
        ] {
            assert!(
                JourneyId::parse(invalid).is_err(),
                "{invalid:?} must not parse"
            );
        }
        // Unknown fields are rejected on read (strict parse).
        assert!(
            serde_json::from_str::<AnchorId>("\"Model Picker\"").is_err(),
            "invalid anchors are rejected on read, not ignored"
        );
    }

    #[test]
    fn product_journey_references_follow_the_frozen_grammar() {
        for valid in ["J-01", "J-07", "J-13", "J-18"] {
            assert_eq!(ok(JourneyRef::parse(valid)).as_str(), valid);
        }
        for invalid in ["", "J-1", "J-001", "J-1a", "j-01", "J01", "J-01 ", "K-01"] {
            assert!(
                JourneyRef::parse(invalid).is_err(),
                "{invalid:?} must not parse"
            );
        }
        assert!(
            serde_json::from_str::<JourneyRef>("\"j-01\"").is_err(),
            "lowercase journey references are rejected on read"
        );
    }

    #[test]
    fn capability_keys_follow_the_frozen_grammar() {
        // The frozen grammar vectors of flauz-exec's CapabilityId,
        // re-pinned here: the formats are identical across the seam.
        for valid in [
            "terminal",
            "computer.screen",
            "computer.keyboard",
            "computer.mouse",
            "desktop.gui",
            "filesystem.read",
            "browser.input",
            "persistent_storage",
        ] {
            assert_eq!(ok(CapabilityKey::parse(valid)).as_str(), valid);
        }
        for invalid in [
            "",
            "Terminal",
            "computer..screen",
            ".screen",
            "screen.",
            "computer screen",
        ] {
            assert!(
                CapabilityKey::parse(invalid).is_err(),
                "{invalid:?} must not parse"
            );
        }
    }

    #[test]
    fn chords_follow_the_canonical_grammar() {
        // The F1 binding vocabulary the lab scenes exercise, as data.
        for valid in [
            "escape",
            "return",
            "tab",
            "ctrl+n",
            "ctrl+return",
            "ctrl+k",
            "ctrl+alt+shift+6",
            "ctrl+alt+shift+7",
            "ctrl+alt+shift+m",
            "ctrl+shift+]",
            "ctrl+shift+[",
            "ctrl+page_down",
            "ctrl+`",
            "alt+^",
        ] {
            assert_eq!(ok(Chord::parse(valid)).as_str(), valid);
        }
        for invalid in [
            "",
            "Ctrl+K",
            "ctrl++k",
            "ctrl+",
            "ctrl+escape+extra",
            "ctrl+ctrl+k",
            "alt+ctrl+k",
            "ctrl+kungfu",
            "ctrl+ page_down",
        ] {
            assert!(Chord::parse(invalid).is_err(), "{invalid:?} must not parse");
        }
        let chord = ok(Chord::parse("ctrl+alt+shift+6"));
        assert_eq!(ok(serde_json::to_string(&chord)), "\"ctrl+alt+shift+6\"");
        assert!(
            serde_json::from_str::<Chord>("\"CTRL+K\"").is_err(),
            "non-canonical chords are rejected on read"
        );
    }

    #[test]
    fn frame_digests_are_sixteen_lowercase_hex_characters() {
        let digest = FrameDigest::from_fnv1a(fnv1a64(b"picker-open"));
        assert_eq!(digest.as_str().len(), 16);
        assert!(
            digest
                .as_str()
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
        let reloaded = ok(FrameDigest::parse(digest.as_str()));
        assert_eq!(reloaded, digest);
        assert!(FrameDigest::parse("xyz").is_err());
        assert!(FrameDigest::parse("0123456789ABCDEF").is_err());
        assert!(
            serde_json::from_str::<FrameDigest>("\"0123456789abcdeZ\"").is_err(),
            "non-hex digests are rejected on read"
        );
    }

    #[test]
    fn vlm_read_references_are_bounded_archive_tokens() {
        let reference = ok(VlmReadRef::parse("vlm-frame-0001.json"));
        assert_eq!(reference.as_str(), "vlm-frame-0001.json");
        assert_eq!(
            ok(serde_json::to_string(&reference)),
            "\"vlm-frame-0001.json\""
        );
        assert!(VlmReadRef::parse("").is_err());
        assert!(VlmReadRef::parse("VLM-Frame").is_err());
        assert!(VlmReadRef::parse("read result").is_err());
        assert!(VlmReadRef::parse(&"a".repeat(MAX_NAME_BYTES + 1)).is_err());
    }

    #[test]
    fn fnv1a_is_deterministic_and_dependency_free() {
        assert_eq!(fnv1a64(b"picker-open"), fnv1a64(b"picker-open"));
        assert_ne!(fnv1a64(b"picker-open"), fnv1a64(b"picker-closed"));
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    }
}
