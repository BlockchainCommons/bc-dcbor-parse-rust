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
//!
//! - `parse_dcbor_item`: Parses a string in CBOR diagnostic notation into a
//!   `CBOR` object.
//! - `compose_dcbor_array`: Composes a `CBOR` array from a slice of strings
//!   representing dCBOR items in diagnostic notation.
//! - `compose_dcbor_map`: Composes a `CBOR` map from a slice of strings
//!   representing the key-value pairs in dCBOR diagnostic notation.
//!
//! | Type                | Example(s)                                                  |
//! | ------------------- | ----------------------------------------------------------- |
//! | Boolean             | `true`<br>`false`                                           |
//! | Null                | `null`                                                      |
//! | Integers            | `0`<br>`1`<br>`-1`<br>`42`                                  |
//! | Floats              | `3.14`<br>`-2.5`<br>`Infinity`<br>`-Infinity`<br>`NAN`      |
//! | Strings             | `"hello"`<br>`"🌎"`                                      |
//! | Hex Byte Strings    | `h'68656c6c6f'`                                             |
//! | Base64 Byte Strings | `b64'AQIDBAUGBwgJCg=='`                                     |
//! | Tagged Values       | `1234("hello")`<br>`5678(3.14)`                             |
//! | Name-Tagged Values  | `tag-name("hello")`<br>`tag-name(3.14)`                     |
//! | Known Values        | `'1'`<br>`'isA'`                                            |
//! | URs                 | `ur:date/cyisdadmlasgtapttl`                                |
//! | Arrays              | `[1, 2, 3]`<br>`["hello", "world"]`<br>`[1, [2, 3]]`        |
//! | Maps                | `{1: 2, 3: 4}`<br>`{"key": "value"}`<br>`{1: [2, 3], 4: 5}` |
//!
//! ## Parsing Named Tags and Uniform Resources (URs)
//!
//! A [Uniform Resource
//! (UR)](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md)
//! is a URI representation of tagged dCBOR, where the tag is represented as a
//! text type component. The last component of the UR is the untagged CBOR
//! encoded as ByteWords, including a CRC-32 checksum in the last eight letters.
//!
//! To parse named tags and URs, the correspondence between the tag name (UR
//! type) and the integer CBOR tag value must be known. This is done by using
//! the `with_tags!` macro to access the global tags registry. Clients wishing
//! to parse named tags and URs must register the CBOR tag value and its
//! corresponding name in the global tags registry. The
//! [`dcbor`](https://crates.io/crates/dcbor) crate only registers one tag and
//! name for `date` (tag 1). The [`bc-tags`](https://crates.io/crates/bc-tags)
//! crate registers many more. See the `register_tags` functions in these crates
//! for examples of how to register your own tags.

mod parse;
pub use parse::{ parse_dcbor_item, Error as ParseError, Result as ParseResult };

mod compose;
pub use compose::{
    compose_dcbor_array,
    compose_dcbor_map,
    Error as ComposeError,
    Result as ComposeResult,
};
