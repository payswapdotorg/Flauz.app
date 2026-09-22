//! Private ULID utility (kernel §2): validation and monotonic generation.
//!
//! This module is the crate's ONLY entropy source. Contract code elsewhere
//! never reads the clock or randomness; identifiers are either supplied by
//! callers or generated here. Generation is monotonic within the process:
//! a value that would not sort after the previous one is incremented instead.
//!
//! The ULID grammar is 26 characters of Crockford Base32 (uppercase, no `I`,
//! `L`, `O` or `U`) encoding 48 timestamp bits followed by 80 randomness
//! bits. No external ULID crate is used; the frozen kernel vectors pin the
//! behavior of this module across the F2 crates.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Length of the ULID string (excluding the entity-kind prefix).
pub(crate) const ULID_LEN: usize = 26;

/// The Crockford Base32 alphabet used by canonical identifiers.
pub(crate) const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

static GENERATION_COUNTER: AtomicU64 = AtomicU64::new(0);
static LAST_GENERATED: Mutex<Option<u128>> = Mutex::new(None);

/// Generates a fresh ULID string, strictly monotonic within this process.
pub(crate) fn generate_ulid() -> String {
    encode_ulid(monotonic_next())
}

fn encode_ulid(mut value: u128) -> String {
    let mut buffer = String::with_capacity(ULID_LEN);
    let mut digits = [0u8; ULID_LEN];
    for slot in digits.iter_mut().rev() {
        *slot = CROCKFORD_ALPHABET[(value % 32) as usize];
        value /= 32;
    }
    for byte in digits {
        // Every digit comes from the ASCII alphabet, so this cannot fail.
        buffer.push(byte as char);
    }
    buffer
}

fn monotonic_next() -> u128 {
    let mut last = LAST_GENERATED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut candidate = fresh_value();
    if let Some(previous) = *last
        && candidate <= previous
    {
        candidate = previous.saturating_add(1);
    }
    *last = Some(candidate);
    candidate
}

/// Builds a ULID value from the current millisecond timestamp (48 bits) and
/// 80 bits of mixed process entropy.
fn fresh_value() -> u128 {
    let millis = unix_epoch_millis();
    let entropy = mix_entropy();
    ((millis as u128) << 80) | (entropy & ((1u128 << 80) - 1))
}

fn unix_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

/// Mixes process-wide sources into 80 bits of uniqueness material. This is
/// not a cryptographic source; the process-local monotonic guard guarantees
/// distinctness even when the mix is poor.
fn mix_entropy() -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let counter = GENERATION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos() as u128)
        .unwrap_or(0);

    let mut thread_hasher = DefaultHasher::new();
    std::thread::current().id().hash(&mut thread_hasher);
    let thread_mix = thread_hasher.finish();

    let address_mix = &GENERATION_COUNTER as *const AtomicU64 as usize;

    let mut mixed: u128 = 0x2545_F491_4F6C_DD1D;
    mixed ^= u128::from(counter).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    mixed ^= nanos.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    mixed ^= u128::from(thread_mix).wrapping_mul(0x1656_67B1_9E37_79F9);
    mixed ^= u128::from(address_mix as u64).wrapping_mul(0x27D4_EB2F_A670_0C11);
    mixed ^= u128::from(std::process::id()).wrapping_mul(0x85EB_CA6B_43D1_0B69);
    mixed = mixed.rotate_left(31) ^ (mixed >> 29);
    mixed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_ulid(value: &str) -> bool {
        value.len() == ULID_LEN && value.bytes().all(|byte| CROCKFORD_ALPHABET.contains(&byte))
    }

    #[test]
    fn generated_ulids_are_valid_crockford_and_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..1000 {
            let value = generate_ulid();
            assert!(is_valid_ulid(&value));
            assert!(seen.insert(value), "generated a duplicate ULID");
        }
    }

    #[test]
    fn generated_ulids_are_monotonic() {
        let mut previous = generate_ulid();
        for _ in 0..1000 {
            let current = generate_ulid();
            assert!(current > previous, "ULID generation is not monotonic");
            previous = current;
        }
    }

    #[test]
    fn crockford_alphabet_excludes_ambiguous_characters() {
        assert_eq!(CROCKFORD_ALPHABET.len(), 32);
        for excluded in *b"ILOU" {
            assert!(!CROCKFORD_ALPHABET.contains(&excluded));
        }
        assert!(!is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPI0"));
        assert!(!is_valid_ulid("01j8zq5v8k3t2b7n6x4r9dqpa0"));
        assert!(!is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPA"));
        assert!(is_valid_ulid("01J8ZQ5V8K3T2B7N6X4R9DQPA0"));
    }
}
