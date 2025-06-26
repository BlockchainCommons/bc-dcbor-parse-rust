use dcbor_parse::{parse_dcbor_item, ParseError};

fn main() {
    println!("=== dCBOR Parse - Duplicate Key Detection ===\n");

    // Valid map - should parse successfully
    let valid_map = r#"{"key1": "value1", "key2": "value2", "key3": "value3"}"#;
    println!("Valid map: {}", valid_map);
    match parse_dcbor_item(valid_map) {
        Ok(cbor) => println!("✓ Parsed successfully: {}\n", cbor.diagnostic()),
        Err(e) => println!("✗ Parse error: {:?}\n", e),
    }

    // Invalid map with duplicate string keys - should fail
    let invalid_map1 = r#"{"key1": "value1", "key2": "value2", "key1": "value3"}"#;
    println!("Invalid map (duplicate string keys): {}", invalid_map1);
    match parse_dcbor_item(invalid_map1) {
        Ok(cbor) => println!("✗ Unexpectedly parsed: {}\n", cbor.diagnostic()),
        Err(ParseError::DuplicateMapKey(span)) => {
            println!("✓ Correctly detected duplicate key at position {}..{}\n", span.start, span.end);
        },
        Err(e) => println!("✗ Unexpected error: {:?}\n", e),
    }

    // Invalid map with duplicate integer keys - should fail
    let invalid_map2 = "{1: \"value1\", 2: \"value2\", 1: \"value3\"}";
    println!("Invalid map (duplicate integer keys): {}", invalid_map2);
    match parse_dcbor_item(invalid_map2) {
        Ok(cbor) => println!("✗ Unexpectedly parsed: {}\n", cbor.diagnostic()),
        Err(ParseError::DuplicateMapKey(span)) => {
            println!("✓ Correctly detected duplicate key at position {}..{}\n", span.start, span.end);
        },
        Err(e) => println!("✗ Unexpected error: {:?}\n", e),
    }

    // Invalid map with numeric equivalence (1 and 1.0) - should fail
    let invalid_map3 = "{1: \"value1\", 2: \"value2\", 1.0: \"value3\"}";
    println!("Invalid map (numeric equivalent keys 1 and 1.0): {}", invalid_map3);
    match parse_dcbor_item(invalid_map3) {
        Ok(cbor) => println!("✗ Unexpectedly parsed: {}\n", cbor.diagnostic()),
        Err(ParseError::DuplicateMapKey(span)) => {
            println!("✓ Correctly detected duplicate key at position {}..{}\n", span.start, span.end);
        },
        Err(e) => println!("✗ Unexpected error: {:?}\n", e),
    }

    println!("All duplicate key detection tests completed!");
}
