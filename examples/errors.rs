//! The domain error: variants, the machine `kind()`, the human `caption()`,
//! and the full `Display` message.
//!
//! Run with: `cargo run --example errors`

use error_forge::ForgeError;
use iqdb_types::IqdbError;

fn main() {
    let errors = [
        IqdbError::DimensionMismatch {
            expected: 768,
            found: 384,
        },
        IqdbError::InvalidVector,
        IqdbError::InvalidConfig {
            reason: "dim must be greater than zero",
        },
        IqdbError::NotFound,
        IqdbError::Duplicate,
        IqdbError::InvalidMetric,
        IqdbError::InvalidFilter,
        IqdbError::ResourceLimitExceeded {
            kind: "metadata_keys",
            max: 64,
            found: 91,
        },
    ];

    // `kind()` is a stable machine identifier; `caption()` is a fixed human
    // summary; `Display` is the full operator-facing message (with details).
    for err in errors {
        println!("[{}] {} — {}", err.kind(), err.caption(), err);
    }

    // Branch on the exact cause instead of parsing a string.
    let err = IqdbError::DimensionMismatch {
        expected: 3,
        found: 2,
    };
    match err {
        IqdbError::DimensionMismatch { expected, found } => {
            println!("recover: re-embed the query at {expected} dims (got {found})");
        }
        // `IqdbError` is #[non_exhaustive], so a wildcard arm is required.
        other => println!("unhandled: {other}"),
    }
}
