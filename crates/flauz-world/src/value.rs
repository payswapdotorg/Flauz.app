//! Canonical JSON values and event payloads (kernel §4).
//!
//! Canonical state contains no floats and no null payload values: only
//! integers, bools, strings, arrays and string-keyed maps. [`CanonicalValue`]
//! makes those rules unrepresentable-as-well-as-rejected: a float or null
//! fails to deserialize, and payload construction validates bounds.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{MAX_PAYLOAD_FIELDS, MAX_PAYLOAD_ITEMS, MAX_PAYLOAD_STRING_BYTES, WorldError};

/// A canonical JSON value: bool, integer, string, array or string-keyed map.
/// Floats and nulls are not part of canonical state and are rejected on
/// deserialize.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum CanonicalValue {
    /// A boolean.
    Bool(bool),
    /// A 64-bit signed integer. Canonical state has no floats.
    Int(i64),
    /// A string.
    Str(String),
    /// An array of canonical values.
    Array(Vec<CanonicalValue>),
    /// A string-keyed map of canonical values.
    Object(BTreeMap<String, CanonicalValue>),
}

impl CanonicalValue {
    /// Returns the boolean value when this is a [`CanonicalValue::Bool`].
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the integer value when this is a [`CanonicalValue::Int`].
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the string value when this is a [`CanonicalValue::Str`].
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    /// Validates the canonical bounds (string lengths, container sizes)
    /// recursively.
    pub fn validate_bounds(&self) -> Result<(), WorldError> {
        match self {
            Self::Bool(_) | Self::Int(_) => Ok(()),
            Self::Str(value) => check_payload_string("payload string", value),
            Self::Array(items) => {
                check_payload_items("payload array", items.len())?;
                for item in items {
                    item.validate_bounds()?;
                }
                Ok(())
            }
            Self::Object(fields) => {
                check_payload_items("payload object", fields.len())?;
                for (key, value) in fields {
                    check_payload_string("payload key", key)?;
                    value.validate_bounds()?;
                }
                Ok(())
            }
        }
    }
}

fn check_payload_string(field: &'static str, value: &str) -> Result<(), WorldError> {
    if value.len() > MAX_PAYLOAD_STRING_BYTES {
        Err(WorldError::invalid(format!(
            "{field} exceeds {MAX_PAYLOAD_STRING_BYTES} bytes"
        )))
    } else {
        Ok(())
    }
}

fn check_payload_items(field: &'static str, count: usize) -> Result<(), WorldError> {
    if count > MAX_PAYLOAD_ITEMS {
        Err(WorldError::invalid(format!(
            "{field} exceeds {MAX_PAYLOAD_ITEMS} items"
        )))
    } else {
        Ok(())
    }
}

impl From<bool> for CanonicalValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for CanonicalValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for CanonicalValue {
    fn from(value: i32) -> Self {
        Self::Int(i64::from(value))
    }
}

impl From<&str> for CanonicalValue {
    fn from(value: &str) -> Self {
        Self::Str(value.to_owned())
    }
}

impl From<String> for CanonicalValue {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl fmt::Display for CanonicalValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(formatter, "{value}"),
            Self::Int(value) => write!(formatter, "{value}"),
            Self::Str(value) => write!(formatter, "{value:?}"),
            Self::Array(items) => {
                formatter.write_str("[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{item}")?;
                }
                formatter.write_str("]")
            }
            Self::Object(fields) => {
                formatter.write_str("{")?;
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{key:?}: {value}")?;
                }
                formatter.write_str("}")
            }
        }
    }
}

struct CanonicalValueVisitor;

impl<'de> Visitor<'de> for CanonicalValueVisitor {
    type Value = CanonicalValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "a canonical JSON value: bool, integer, string, array, or string-keyed object \
             (floats and nulls are not canonical)",
        )
    }

    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(CanonicalValue::Bool(value))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(CanonicalValue::Int(value))
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        i64::try_from(value)
            .map(CanonicalValue::Int)
            .map_err(|_| E::custom("integer out of canonical i64 range"))
    }

    fn visit_f64<E: serde::de::Error>(self, _value: f64) -> Result<Self::Value, E> {
        Err(E::custom("floats are not canonical state"))
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Err(E::custom("null is not a canonical payload value"))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        check_payload_string("payload string", value)
            .map_err(|error| E::custom(error.to_string()))?;
        Ok(CanonicalValue::Str(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element::<CanonicalValue>()? {
            if items.len() == MAX_PAYLOAD_ITEMS {
                return Err(serde::de::Error::custom(format!(
                    "payload array exceeds {MAX_PAYLOAD_ITEMS} items"
                )));
            }
            items.push(item);
        }
        Ok(CanonicalValue::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut fields = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, CanonicalValue>()? {
            if fields.len() == MAX_PAYLOAD_ITEMS {
                return Err(serde::de::Error::custom(format!(
                    "payload object exceeds {MAX_PAYLOAD_ITEMS} items"
                )));
            }
            check_payload_string("payload key", &key)
                .map_err(|error| serde::de::Error::custom(error.to_string()))?;
            fields.insert(key, value);
        }
        Ok(CanonicalValue::Object(fields))
    }
}

impl<'de> Deserialize<'de> for CanonicalValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CanonicalValueVisitor)
    }
}

/// A canonical event payload: a bounded, string-keyed map of
/// [`CanonicalValue`]s. Byte payloads belong in referenced artifacts, never
/// inlined here.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(transparent)]
pub struct Payload(BTreeMap<String, CanonicalValue>);

impl Payload {
    /// The empty payload.
    #[must_use]
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    /// Builds a payload from a map, enforcing the canonical bounds.
    pub fn new(map: BTreeMap<String, CanonicalValue>) -> Result<Self, WorldError> {
        let payload = Self(map);
        payload.validate()?;
        Ok(payload)
    }

    /// Adds a field to the payload, enforcing the canonical bounds.
    pub fn with(mut self, key: &str, value: CanonicalValue) -> Result<Self, WorldError> {
        check_payload_string("payload key", key)?;
        self.0.insert(key.to_owned(), value);
        self.validate()?;
        Ok(self)
    }

    /// Looks up a payload field.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&CanonicalValue> {
        self.0.get(key)
    }

    /// Iterates the payload fields in deterministic key order.
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, String, CanonicalValue> {
        self.0.iter()
    }

    /// Returns the number of payload fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` when the payload has no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Validates the canonical bounds of this payload.
    pub fn validate(&self) -> Result<(), WorldError> {
        if self.0.len() > MAX_PAYLOAD_FIELDS {
            return Err(WorldError::invalid(format!(
                "payload exceeds {MAX_PAYLOAD_FIELDS} fields"
            )));
        }
        for (key, value) in &self.0 {
            check_payload_string("payload key", key)?;
            value.validate_bounds()?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = BTreeMap::<String, CanonicalValue>::deserialize(deserializer)?;
        let payload = Self(map);
        payload
            .validate()
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        Ok(payload)
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("{")?;
        for (index, (key, value)) in self.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{key:?}: {value}")?;
        }
        formatter.write_str("}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn floats_and_nulls_fail_to_deserialize() {
        assert!(serde_json::from_str::<CanonicalValue>("1.5").is_err());
        assert!(serde_json::from_str::<CanonicalValue>("1.0").is_err());
        assert!(serde_json::from_str::<CanonicalValue>("1e3").is_err());
        assert!(serde_json::from_str::<CanonicalValue>("null").is_err());
        assert!(serde_json::from_str::<CanonicalValue>("true").is_ok());
        assert!(serde_json::from_str::<CanonicalValue>("42").is_ok());
        assert!(serde_json::from_str::<CanonicalValue>("\"x\"").is_ok());
        assert!(serde_json::from_str::<CanonicalValue>("[1, true, \"x\"]").is_ok());
        assert!(serde_json::from_str::<CanonicalValue>("{\"a\": 1}").is_ok());
        assert!(serde_json::from_str::<CanonicalValue>("{\"a\": null}").is_err());
        assert!(serde_json::from_str::<CanonicalValue>("[1.5]").is_err());
    }

    #[test]
    fn payload_round_trip_and_bounds() {
        let payload = ok(Payload::new(BTreeMap::from([
            (
                "model".to_owned(),
                CanonicalValue::from("model_01J8ZQ5V8K3T2B7N6X4R9DQPE4"),
            ),
            ("attempt".to_owned(), CanonicalValue::from(2i64)),
        ])));
        let serialized = ok(serde_json::to_string(&payload));
        assert_eq!(
            serialized,
            "{\"attempt\":2,\"model\":\"model_01J8ZQ5V8K3T2B7N6X4R9DQPE4\"}"
        );
        let parsed: Payload = ok(serde_json::from_str(&serialized));
        assert_eq!(parsed, payload);

        let oversized = "x".repeat(MAX_PAYLOAD_STRING_BYTES + 1);
        assert!(
            Payload::empty()
                .with("big", CanonicalValue::from(oversized))
                .is_err()
        );
        let mut many = BTreeMap::new();
        for index in 0..=MAX_PAYLOAD_FIELDS {
            many.insert(format!("k{index}"), CanonicalValue::from(1i64));
        }
        assert!(Payload::new(many).is_err());
    }
}
