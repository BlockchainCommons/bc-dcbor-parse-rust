use dcbor::prelude::*;
use crate::{parse_dcbor_item, ParseError};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Error {
    #[error("Invalid odd map length")]
    InvalidOddMapLength,
    #[error("Invalid CBOR item: {0}")]
    ParseError(#[from] ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Composes a dCBOR array from a slice of string slices, and returns a CBOR
/// object representing the array.
///
/// Each string slice is parsed as a dCBOR item.
///
/// # Example
///
/// ```rust
/// # use dcbor_parse::compose_dcbor_array;
/// let cbor = compose_dcbor_array(&["1", "2", "3"]).unwrap();
/// assert_eq!(cbor.diagnostic(), "[1, 2, 3]");
/// ```
pub fn compose_dcbor_array(array: &[&str]) -> Result<CBOR> {
    let mut result = Vec::new();
    for item in array {
        let cbor = parse_dcbor_item(item)?;
        result.push(cbor);
    }
    Ok(result.into())
}

/// Composes a dCBOR map from a slice of string slices, and returns a CBOR
/// object representing the map.
///
/// The length of the slice must be even, as each key must have a corresponding
/// value.
///
/// Each string slice is parsed as a dCBOR item.
///
/// # Example
///
/// ```rust
/// # use dcbor_parse::compose_dcbor_map;
/// let cbor = compose_dcbor_map(&["1", "2", "3", "4"]).unwrap();
/// assert_eq!(cbor.diagnostic(), "{1: 2, 3: 4}");
/// ```
pub fn compose_dcbor_map(array: &[&str]) -> Result<CBOR> {
    if array.len() % 2 != 0 {
        return Err(Error::InvalidOddMapLength);
    }

    let mut map = Map::new();
    for i in (0..array.len()).step_by(2) {
        let key = parse_dcbor_item(array[i])?;
        let value = parse_dcbor_item(array[i + 1])?;
        map.insert(key, value);
    }

    Ok(map.into())
}
