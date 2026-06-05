//! `Display` contract for [`VectorId`] (audit finding M9).
//!
//! Logs and error messages render `VectorId` through `Display`; the derived
//! `Debug` output for `Bytes` prints raw byte arrays, which is unreadable
//! for opaque-key customers (UUIDs, SHA-256, content hashes). The contract
//! locked here:
//!
//! - `VectorId::U64(n)` renders as the decimal integer.
//! - `VectorId::Bytes(b)` renders as lowercase hex, no prefix, no
//!   separators. Empty inputs cannot occur (`TryFrom<Vec<u8>>` rejects
//!   them with `IqdbError::InvalidConfig { reason: "VectorId::Bytes key must not be empty" }`).

#![allow(clippy::unwrap_used)]

use iqdb_types::VectorId;

#[test]
fn u64_display_is_decimal() {
    assert_eq!(VectorId::from(0u64).to_string(), "0");
    assert_eq!(VectorId::from(7u64).to_string(), "7");
    assert_eq!(VectorId::from(u64::MAX).to_string(), u64::MAX.to_string());
}

#[test]
fn bytes_display_is_lowercase_hex_no_prefix() {
    let id = VectorId::try_from(vec![0xde, 0xad, 0xbe, 0xef]).unwrap();
    assert_eq!(id.to_string(), "deadbeef");
}

#[test]
fn bytes_display_pads_each_byte_to_two_hex_chars() {
    // 0x00, 0x0f, 0xf0 must render as "00", "0f", "f0" — not "0", "f",
    // "f0" — so the rendered length is always 2 × byte count.
    let id = VectorId::try_from(vec![0x00, 0x0f, 0xf0]).unwrap();
    assert_eq!(id.to_string(), "000ff0");
}

#[test]
fn bytes_display_handles_long_keys() {
    // A 32-byte SHA-256-shaped id: still hex, still no separators.
    let bytes: Vec<u8> = (0..32).map(|i| i as u8).collect();
    let id = VectorId::try_from(bytes.clone()).unwrap();
    let rendered = id.to_string();
    assert_eq!(rendered.len(), 64);
    assert!(
        rendered
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
    assert!(rendered.starts_with("000102"));
    assert!(rendered.ends_with("1e1f"));
}

#[test]
fn debug_and_display_are_distinct() {
    // Debug stays informative ("Bytes([...])") for in-source troubleshooting;
    // Display is the operator-facing string. They MUST NOT collapse to the
    // same shape, so a log line written via `Display` does not regress to
    // raw byte arrays.
    let id = VectorId::try_from(vec![0x01, 0x02]).unwrap();
    assert_ne!(format!("{id}"), format!("{id:?}"));
    assert_eq!(format!("{id}"), "0102");
}
