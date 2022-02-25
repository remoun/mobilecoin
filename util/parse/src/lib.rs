// Copyright (c) 2018-2022 The MobileCoin Foundation
#![deny(missing_docs)]

//! Miscellaneous parsing and formatting utilities

use core::fmt::Display;
use itertools::Itertools;

pub use mc_sgx_css::Signature as CssSignature;

/// Parse a duration. This can be used with Clap.
/// Supports prefixes like '1m30s'.
/// Defaults to parsing a number of seconds.
/// See [duration_str::parse] for more details.
///
/// ```rust
/// use mc_util_parse::parse_duration;
/// use std::time::Duration;
///
/// let duration = parse_duration("1d").unwrap();
/// assert_eq!(duration, Duration::new(24 * 60 * 60, 0));
///
///
/// let duration = parse_duration("3m31s").unwrap();
/// assert_eq!(duration, Duration::new(211, 0));
///
/// let duration = parse_duration("3m + 31").unwrap(); // default unit is seconds
/// assert_eq!(duration, Duration::new(211, 0));
///
/// let duration = parse_duration("3m13s29ms").unwrap();
/// assert_eq!(duration, Duration::new(193, 29 * 1000 * 1000 + 0 + 0));
///
/// let duration = parse_duration("3m + 1s + 29ms +17µs").unwrap();
/// assert_eq!(duration, Duration::new(181, 29 * 1000 * 1000 + 17 * 1000 + 0)
/// );
///
/// let duration = parse_duration("1m*10").unwrap(); // default unit is seconds
/// assert_eq!(duration, Duration::new(600, 0));
///
/// let duration = parse_duration("1m*10ms").unwrap();
/// assert_eq!(duration, Duration::new(0, 600 * 1000 * 1000));
///
/// let duration = parse_duration("1m * 1m").unwrap();
/// assert_eq!(duration, Duration::new(3600, 0));
/// let duration = parse_duration("42µs").unwrap();
/// assert_eq!(duration,Duration::from_micros(42));
/// ```
pub use duration_str::parse as parse_duration;

/// Load a CSS file from disk. This represents a signature over an enclave,
/// and contains attestation parameters like MRENCLAVE and MRSIGNER as well
/// as other stuff.
pub fn load_css_file(filename: &str) -> Result<CssSignature, String> {
    let bytes = std::fs::read(filename)
        .map_err(|err| format!("Failed reading file '{}': {}", filename, err))?;
    let signature = CssSignature::try_from(&bytes[..])
        .map_err(|err| format!("Failed parsing CSS file '{}': {}", filename, err))?;
    Ok(signature)
}

/// Helper to format a sequence as a comma-separated list
/// (This is used with lists of Ingest peer uris in logs,
/// because the debug logging of that object is harder to read)
///
/// To use this, wrap the value in SeqDisplay( ) then format it
pub struct SeqDisplay<T: Display, I: Iterator<Item = T> + Clone>(pub I);

impl<T: Display, I: Iterator<Item = T> + Clone> Display for SeqDisplay<T, I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "[{}]", self.0.clone().format(", "))
    }
}
