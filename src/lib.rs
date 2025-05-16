//! # dCBOR Diagnostic Parser and Composer
//!
//! This crate provides tools for parsing and composing the [CBOR diagnostic
//! notation](https://datatracker.ietf.org/doc/html/rfc8949#name-diagnostic-notation)
//! into [dCBOR (deterministic
//! CBOR)](https://datatracker.ietf.org/doc/draft-mcnally-deterministic-cbor/)
//! data items.
//!
//! It is intended for use in testing, debugging, the `dcbor` command line tool,
//! and other scenarios where a human-readable representation of dCBOR is
//! useful. It is not optimized for performance and should not be used in
//! production environments where binary dCBOR is expected.
//!
//! The primary functions provided are:
//! - `parse_dcbor_item`: Parses a string in CBOR diagnostic notation into a
//!   `CBOR` object.
//! - `compose_dcbor_array`: Composes a `CBOR` array from a slice of strings
//!   representing dCBOR items in diagnostic notation.
//! - `compose_dcbor_map`: Composes a `CBOR` map from a slice of strings
//!   representing the key-value pairs in dCBOR diagnostic notation.

mod parse;
pub use parse::parse_dcbor_item;

mod compose;
pub use compose::{ compose_dcbor_array, compose_dcbor_map };

mod error;
pub use error::{ Error, Result };
