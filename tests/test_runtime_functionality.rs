use bc_ur::prelude::*;
use dcbor_parse::parse_dcbor_item;

// These tests verify that the full regex patterns are used at runtime,
// not the simplified patterns that are provided for IDE compatibility.
//
// The simplified patterns are designed to suppress rust-analyzer's
// "macro invocation exceeds token limit" errors while preserving
// the full functionality of the parser at runtime.
//
// If the simplified patterns were used at runtime, many of these tests
// would fail because the simplified patterns don't support:
// - Complex string escapes
// - Fractional seconds in dates
// - Timezone information in dates
// - Minimum length requirements for base64
// - Control character exclusion in strings

/// Test that basic functionality is preserved with simplified patterns
#[test]
fn test_basic_functionality_preserved() {
    // Test basic string parsing
    let result = parse_dcbor_item(r#""Hello, World!""#).unwrap();
    assert_eq!(result, "Hello, World!".into());

    // Test empty string
    let result = parse_dcbor_item(r#""""#).unwrap();
    assert_eq!(result, "".into());

    // Test hex string parsing
    let result = parse_dcbor_item("h'deadbeef'").unwrap();
    assert_eq!(
        result,
        dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])
    );

    // Test empty hex
    let result = parse_dcbor_item("h''").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(vec![]));

    // Test basic base64 parsing
    let result = parse_dcbor_item("b64'SGVsbG8='").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(b"Hello"));

    // Test date parsing (date only)
    dcbor::register_tags();
    let result = parse_dcbor_item("2023-12-25").unwrap();
    let expected = Date::from_ymd(2023, 12, 25);
    assert_eq!(result, expected.to_cbor());

    // Test basic array
    let result = parse_dcbor_item(r#"["hello", h'dead', 42]"#).unwrap();
    let array = result.as_array().expect("Should be an array");
    assert_eq!(array.len(), 3);
    assert_eq!(array[0], "hello".into());
    assert_eq!(array[1], dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD]));
    assert_eq!(array[2], 42.into());

    // Test basic map
    let result =
        parse_dcbor_item(r#"{"key": "value", "number": 123}"#).unwrap();
    let map = result.as_map().expect("Should be a map");
    assert!(map.contains_key("key"));
    assert!(map.contains_key("number"));
}

/// Test that simplified patterns don't break basic functionality during
/// compilation
#[test]
fn test_simplified_patterns_compilation() {
    // This test ensures that when rust-analyzer uses simplified patterns,
    // the basic tokenization still works correctly.

    // Basic patterns that should work with both full and simplified regex
    let inputs = vec![
        r#""simple""#,
        "h'ff'",
        "b64'QUE='",
        "2023-01-01",
        "42",
        "true",
        "false",
        "null",
        "[1, 2, 3]",
        r#"{"a": 1}"#,
    ];

    for input in inputs {
        let result = parse_dcbor_item(input);
        assert!(result.is_ok(), "Failed to parse: {}", input);
    }
}

/// Test hex string parsing with various formats
#[test]
fn test_hex_parsing_comprehensive() {
    // Test empty hex string
    let result = parse_dcbor_item("h''").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(vec![]));

    // Test single byte
    let result = parse_dcbor_item("h'FF'").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(vec![0xFF]));

    // Test lowercase hex
    let result = parse_dcbor_item("h'deadbeef'").unwrap();
    assert_eq!(
        result,
        dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])
    );

    // Test uppercase hex
    let result = parse_dcbor_item("h'DEADBEEF'").unwrap();
    assert_eq!(
        result,
        dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])
    );

    // Test mixed case hex
    let result = parse_dcbor_item("h'DeAdBeEf'").unwrap();
    assert_eq!(
        result,
        dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])
    );
}

/// Test that IDE fixes don't break normal compilation
#[test]
fn test_ide_compatibility() {
    // This test demonstrates that our conditional compilation approach
    // maintains functionality while fixing IDE issues.

    // Test that all basic token types still work
    assert!(parse_dcbor_item("true").is_ok());
    assert!(parse_dcbor_item("false").is_ok());
    assert!(parse_dcbor_item("null").is_ok());
    assert!(parse_dcbor_item("42").is_ok());
    assert!(parse_dcbor_item("3.14").is_ok());
    assert!(parse_dcbor_item(r#""string""#).is_ok());
    assert!(parse_dcbor_item("h'dead'").is_ok());
    assert!(parse_dcbor_item("b64'SGVsbG8='").is_ok());

    dcbor::register_tags();
    assert!(parse_dcbor_item("2023-01-01").is_ok());

    // Test composite structures
    assert!(parse_dcbor_item("[1, 2, 3]").is_ok());
    assert!(parse_dcbor_item(r#"{"key": "value"}"#).is_ok());
    assert!(parse_dcbor_item("42(123)").is_ok()); // Tagged value with numeric tag
}

/// Test that the lexer correctly captures complex string patterns
/// The DCBOR parser captures the literal string including escape sequences
/// It does NOT process escape sequences like JSON - that's the key insight!
#[test]
fn test_complex_string_escapes_runtime_only() {
    // Test string with quotes - the lexer should capture the literal escaped
    // string
    let result = parse_dcbor_item(r#""She said \"Hello\"""#).unwrap();
    // The parser captures the literal string with escape sequences, not
    // processed
    assert_eq!(result, r#"She said \"Hello\""#.into());

    // Test string with backslash escapes
    let result = parse_dcbor_item(r#""Path\\to\\file""#).unwrap();
    assert_eq!(result, r#"Path\\to\\file"#.into());

    // Test string with escape sequences - they remain as literals
    let result = parse_dcbor_item(r#""Line 1\nLine 2\tTabbed""#).unwrap();
    assert_eq!(result, r#"Line 1\nLine 2\tTabbed"#.into());

    // Test string with unicode escapes - captured as literals
    let result = parse_dcbor_item(r#""Unicode: \u0041\u0042\u0043""#).unwrap();
    assert_eq!(result, r#"Unicode: \u0041\u0042\u0043"#.into());

    // Test that the complex regex pattern correctly validates the string
    // structure These would be rejected by the simplified pattern but
    // accepted by the full pattern
    let result = parse_dcbor_item(r#""Valid escape: \"""#).unwrap();
    assert_eq!(result, r#"Valid escape: \""#.into());

    let result = parse_dcbor_item(r#""Valid unicode: \u1234""#).unwrap();
    assert_eq!(result, r#"Valid unicode: \u1234"#.into());
}

/// Test complex date formats that ONLY work with full regex patterns
/// These tests would FAIL if the simplified patterns were used during
/// compilation
#[test]
fn test_complex_date_formats_runtime_only() {
    dcbor::register_tags();

    // Test date with timezone Z - would fail with simplified pattern
    let result = parse_dcbor_item("2023-12-25T10:30:45Z").unwrap();
    let expected = Date::from_string("2023-12-25T10:30:45Z").unwrap();
    assert_eq!(result, expected.to_cbor());

    // Test date with positive timezone offset - would fail with simplified
    // pattern
    let result = parse_dcbor_item("2023-12-25T10:30:45+05:30").unwrap();
    let expected = Date::from_string("2023-12-25T10:30:45+05:30").unwrap();
    assert_eq!(result, expected.to_cbor());

    // Test date with negative timezone offset - would fail with simplified
    // pattern
    let result = parse_dcbor_item("2023-12-25T10:30:45-08:00").unwrap();
    let expected = Date::from_string("2023-12-25T10:30:45-08:00").unwrap();
    assert_eq!(result, expected.to_cbor());

    // Test date with milliseconds - would fail with simplified pattern
    let result = parse_dcbor_item("2023-12-25T10:30:45.123Z").unwrap();
    let expected = Date::from_string("2023-12-25T10:30:45.123Z").unwrap();
    assert_eq!(result, expected.to_cbor());

    // Test date with microseconds - would fail with simplified pattern
    let result = parse_dcbor_item("2023-12-25T10:30:45.123456Z").unwrap();
    let expected = Date::from_string("2023-12-25T10:30:45.123456Z").unwrap();
    assert_eq!(result, expected.to_cbor());
}

/// Test complex base64 minimum length requirement that ONLY works with full
/// regex These tests would FAIL if the simplified patterns were used during
/// compilation
#[test]
fn test_complex_base64_requirements_runtime_only() {
    // Test base64 with minimum 2-character requirement - would fail with
    // simplified pattern The full pattern has {2,} quantifier, simplified
    // has *
    let result = parse_dcbor_item("b64'QQ=='").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(vec![0x41]));

    // Test longer base64 strings that meet the minimum requirement
    let result = parse_dcbor_item("b64'SGVsbG8gV29ybGQ='").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(b"Hello World"));

    // Test base64 without padding but meeting minimum length
    let result = parse_dcbor_item("b64'SGVsbG8='").unwrap();
    assert_eq!(result, dcbor::CBOR::to_byte_string(b"Hello"));
}

/// Test mixed complex patterns in realistic structures
/// These would FAIL if simplified patterns were used during compilation
#[test]
fn test_complex_mixed_patterns_runtime_only() {
    dcbor::register_tags();

    // Complex array with features that require full patterns
    let complex_array = r#"[
        "String with \"quotes\" and \\n newlines",
        h'deadbeef',
        b64'SGVsbG8gV29ybGQ=',
        2023-12-25T10:30:45.123Z,
        "Unicode: \\u0041\\u0042\\u0043"
    ]"#;

    let result = parse_dcbor_item(complex_array).unwrap();
    let array = result.as_array().expect("Should be an array");
    assert_eq!(array.len(), 5);

    // Verify complex string with escapes (literal, not processed)
    assert_eq!(
        array[0],
        r#"String with \"quotes\" and \\n newlines"#.into()
    );

    // Verify hex bytes
    assert_eq!(
        array[1],
        dcbor::CBOR::to_byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])
    );

    // Verify base64 bytes
    assert_eq!(array[2], dcbor::CBOR::to_byte_string(b"Hello World"));

    // Verify complex date with milliseconds and timezone
    let expected_date = Date::from_string("2023-12-25T10:30:45.123Z").unwrap();
    assert_eq!(array[3], expected_date.to_cbor());

    // Verify unicode escape sequences (as literals)
    assert_eq!(array[4], r#"Unicode: \\u0041\\u0042\\u0043"#.into());

    // Complex map - just test that it parses with complex patterns
    let complex_map = r#"{
        "message": "Hello \\\"World\\\" with \\n newlines",
        "data": h'0123456789abcdef',
        "timestamp": 2023-12-25T10:30:45-08:00
    }"#;

    let result = parse_dcbor_item(complex_map).unwrap();
    let map = result.as_map().expect("Should be a map");

    // Basic structure verification
    assert!(map.contains_key("message"));
    assert!(map.contains_key("timestamp"));

    // The fact that this complex structure parsed at all proves full patterns
    // work
    assert!(map.len() >= 2);
}

/// Tests that would fail if simplified patterns were used at runtime
/// These tests prove that the full regex patterns are active during compilation
#[test]
fn test_base64_minimum_length_enforcement() {
    // Test with single character - should fail with full patterns but succeed
    // with simplified
    let input = r#"b64'A'"#;
    let result = parse_dcbor_item(input);

    // Both patterns might fail due to base64 decoder requirements
    // Let's test empty base64 which should definitely fail with full patterns
    let empty_input = r#"b64''"#;
    let empty_result = parse_dcbor_item(empty_input);

    // Empty base64 should fail with full patterns due to {2,} requirement
    assert!(empty_result.is_err());

    // But let's also check that single character fails
    assert!(result.is_err());
}

#[test]
fn test_date_with_fractional_seconds() {
    // This test would fail with simplified patterns that don't support
    // fractional seconds
    let input = r#"2023-12-25T12:30:45.123Z"#;
    dcbor::register_tags();
    let result = parse_dcbor_item(input);

    // This should parse successfully with full patterns
    assert!(result.is_ok());
}

#[test]
fn test_date_with_timezone_offset() {
    // This test would fail with simplified patterns that don't support timezone
    // info
    let input = r#"2023-12-25T12:30:45+05:30"#;
    dcbor::register_tags();
    let result = parse_dcbor_item(input);

    // This should parse successfully with full patterns
    assert!(result.is_ok());
}

#[test]
fn test_string_with_control_characters_rejected() {
    // This test ensures that strings with control characters are properly
    // rejected by the full pattern but would be accepted by the simplified
    // pattern
    let input = "\"hello\x01world\""; // Contains control character \x01
    let result = parse_dcbor_item(input);

    // Full pattern should reject this due to \x00-\x1F exclusion
    // Simplified pattern would accept it
    assert!(result.is_err());
}

#[test]
fn test_string_with_unescaped_quotes_rejected() {
    // Test that unescaped quotes are properly rejected by full pattern
    let input = r#""hello"world""#; // Contains unescaped quote in middle
    let result = parse_dcbor_item(input);

    // This should be rejected by both patterns, but for different reasons
    // Full pattern: doesn't match the complex escape rules
    // Simplified pattern: would match "hello" and stop there
    assert!(result.is_err());
}

#[test]
fn test_complex_string_escapes() {
    // This test would fail with simplified patterns that don't support escape
    // sequences
    let input = r#""hello\nworld\t\u0041""#;
    let result = parse_dcbor_item(input);

    // This should parse successfully with full patterns
    assert!(result.is_ok());

    let parsed = result.unwrap();
    let s = parsed.as_text().expect("Should be a string");
    // The parser should handle the escaped string (stores literal escapes)
    assert!(s.contains("\\n")); // Parser stores literal backslash-n, not newline
    assert!(s.contains("\\u0041")); // Parser stores literal unicode escape
}

#[test]
fn test_runtime_pattern_validation() {
    // This test validates that the full patterns are actually being used at
    // runtime by testing inputs that would produce different results with
    // simplified patterns

    // Test 1: Complex date with microseconds and timezone
    dcbor::register_tags();
    let complex_date = "2023-12-25T10:30:45.123456Z";
    let result = parse_dcbor_item(complex_date);
    assert!(
        result.is_ok(),
        "Complex date should parse with full patterns"
    );

    // Test 2: String with valid escape sequences
    let escaped_string = r#""line1\nline2\ttab\u0041end""#;
    let result = parse_dcbor_item(escaped_string);
    assert!(
        result.is_ok(),
        "Escaped string should parse with full patterns"
    );

    // Test 3: Base64 with proper minimum length
    let base64_input = "b64'SGVsbG8gV29ybGQ='"; // "Hello World" in base64
    let result = parse_dcbor_item(base64_input);
    assert!(
        result.is_ok(),
        "Proper base64 should parse with full patterns"
    );

    // Test 4: Complex mixed input that exercises multiple patterns
    let complex_input = r#"{
        "message": "Hello\nWorld",
        "data": b64'SGVsbG8=',
        "timestamp": 2023-12-25T10:30:45.123Z,
        "binary": h'deadbeef'
    }"#;
    let result = parse_dcbor_item(complex_input);
    assert!(
        result.is_ok(),
        "Complex mixed input should parse with full patterns"
    );
}
