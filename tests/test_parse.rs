use std::collections::HashMap;

use base64::Engine as _;
use bc_ur::prelude::*;
use dcbor_parse::{ParseError, parse_dcbor_item, parse_dcbor_item_partial};
use indoc::indoc;

fn roundtrip<T: Into<CBOR>>(value: T) {
    let cbor = value.into();
    // println!("=== Original ===\n{}", cbor.diagnostic());
    let src = cbor.diagnostic();
    match parse_dcbor_item(&src) {
        Ok(result) => {
            // println!("{}", result.diagnostic());
            if result != cbor {
                panic!("=== Expected ===\n{}\n\n=== Got ===\n{}", cbor, result);
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[test]
fn test_basic_types() {
    roundtrip(true);
    roundtrip(false);
    roundtrip(CBOR::null());
    roundtrip(10);
    roundtrip(3.28);
    roundtrip(f64::INFINITY);
    roundtrip(f64::NEG_INFINITY);
    roundtrip("Hello, world!");
}

fn hex_diagnostic(bytes: &[u8]) -> String {
    let hex = hex::encode(bytes);
    format!("h'{}'", hex)
}

fn base64_diagnostic(bytes: &[u8]) -> String {
    format!(
        "b64'{}'",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

#[test]
fn test_byte_string() {
    let bytes =
        vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
    let cbor = CBOR::to_byte_string(bytes.clone());
    roundtrip(cbor.clone());

    let hex = hex_diagnostic(&bytes);
    assert_eq!(hex, "h'0102030405060708090a'");
    let cbor2 = parse_dcbor_item(&hex).unwrap();
    assert_eq!(cbor2, cbor);

    let base64 = base64_diagnostic(&bytes);
    assert_eq!(base64, "b64'AQIDBAUGBwgJCg=='");
    let cbor3 = parse_dcbor_item(&base64).unwrap();
    assert_eq!(cbor3, cbor);
}

#[test]
fn test_nan() {
    // NaN is a special case because it doesn't equal itself
    let cbor = CBOR::from(f64::NAN);
    let src = cbor.diagnostic();
    assert_eq!(src, "NaN");
    let cbor2 = parse_dcbor_item(&src).unwrap();
    assert!(f64::try_from(cbor2).unwrap().is_nan());
}

#[test]
fn test_tagged() {
    roundtrip(CBOR::to_tagged_value(
        1234,
        CBOR::to_byte_string(vec![0x01, 0x02, 0x03]),
    ));
    roundtrip(CBOR::to_tagged_value(5678, "Hello, world!"));
    roundtrip(CBOR::to_tagged_value(9012, true));
}

#[test]
fn test_array() {
    let v: Vec<i32> = vec![];
    roundtrip(v);

    roundtrip(vec![1, 2, 3]);
    roundtrip(vec![true.to_cbor(), false.to_cbor(), CBOR::null()]);
    roundtrip(vec![
        CBOR::to_byte_string(vec![0x01, 0x02]).to_cbor(),
        "Hello".to_cbor(),
    ]);
    roundtrip(vec![vec![1, 2], vec![3, 4]]);
}

#[test]
fn test_map() {
    let m1: HashMap<String, i32> = HashMap::new();
    roundtrip(m1);

    let mut m2 = HashMap::new();
    m2.insert("key1", 1);
    m2.insert("key2", 2);
    m2.insert("key3", 3);
    roundtrip(m2);

    let mut m3 = HashMap::new();
    m3.insert(1, "value1");
    m3.insert(2, "value2");
    m3.insert(3, "value3");
    roundtrip(m3.clone());

    let mut m4 = HashMap::new();
    m4.insert("key1", CBOR::to_byte_string(vec![0x01, 0x02]));
    m4.insert("key2", "value2".to_cbor());
    m4.insert("key3", m3.to_cbor());
    roundtrip(m4);
}

#[test]
fn test_nested() {
    let nested = vec![
        CBOR::to_tagged_value(
            1234,
            CBOR::to_byte_string(vec![0x01, 0x02, 0x03]),
        ),
        vec![1, 2, 3].to_cbor(),
        HashMap::from([
            ("key1", "value1".to_cbor()),
            ("key2", vec![4, 5, 6].to_cbor()),
        ])
        .to_cbor(),
    ];
    roundtrip(nested);
}

#[test]
fn test_ur() {
    dcbor::register_tags();
    let date = dcbor::Date::from_ymd(2025, 5, 15);
    let ur = date.ur_string();
    assert_eq!(ur, "ur:date/cyisdadmlasgtapttl");
    let date2 = dcbor::Date::from_ur_string(&ur).unwrap();
    assert_eq!(date2, date);
    let date_cbor = parse_dcbor_item(&ur).unwrap();
    assert_eq!(date_cbor, date.to_cbor());
}

#[test]
fn test_named_tag() {
    dcbor::register_tags();
    let date_cbor = dcbor::Date::from_ymd(2025, 5, 15).to_cbor();
    // Replace '1(` with `date(`:
    let date_diag = date_cbor.diagnostic().to_string().replace("1(", "date(");
    let date_cbor2 = parse_dcbor_item(&date_diag).unwrap();
    assert_eq!(date_cbor2, date_cbor);
}

#[test]
fn test_known_value() {
    let v = known_values::IS_A;
    let cbor = v.to_cbor();
    let src = cbor.diagnostic();
    assert_eq!(src, "40000(1)");
    let cbor2 = parse_dcbor_item(&src).unwrap();
    assert_eq!(cbor2, cbor);
    let src2 = "'1'";
    let cbor3 = parse_dcbor_item(src2).unwrap();
    assert_eq!(cbor3, cbor);
    let src3 = "'isA'";
    let cbor4 = parse_dcbor_item(src3).unwrap();
    assert_eq!(cbor4, cbor);
}

#[test]
fn test_unit_known_value() {
    let v = known_values::UNIT;
    let cbor = v.to_cbor();
    let src = cbor.diagnostic();
    assert_eq!(src, "40000(0)");
    let cbor2 = parse_dcbor_item(&src).unwrap();
    assert_eq!(cbor2, cbor);
    let src2 = "'0'";
    let cbor3 = parse_dcbor_item(src2).unwrap();
    assert_eq!(cbor3, cbor);
    let src3 = "''";
    let cbor4 = parse_dcbor_item(src3).unwrap();
    assert_eq!(cbor4, cbor);
    let src4 = "Unit";
    let cbor5 = parse_dcbor_item(src4).unwrap();
    assert_eq!(cbor5, cbor);
}

#[test]
fn test_errors() {
    dcbor::register_tags();

    fn check_error<F>(source: &str, expected: F)
    where
        F: Fn(&ParseError) -> bool,
    {
        let result = parse_dcbor_item(source);
        let err = result.unwrap_err();
        // println!("{}", err.full_message(source));
        assert!(
            expected(&err),
            "Unexpected error for source `{}`: {:?}",
            source,
            err
        );
    }

    check_error("", |e| matches!(e, ParseError::EmptyInput));
    check_error("[1, 2", |e| matches!(e, ParseError::UnexpectedEndOfInput));
    check_error("[1, 2,\n3, 4,", |e| {
        matches!(e, ParseError::UnexpectedEndOfInput)
    });
    check_error("1 1", |e| matches!(e, ParseError::ExtraData(_)));
    check_error("(", |e| matches!(e, ParseError::UnexpectedToken(_, _)));
    check_error("q", |e| matches!(e, ParseError::UnrecognizedToken(_)));
    check_error("[1 2 3]", |e| matches!(e, ParseError::ExpectedComma(_)));
    check_error("{1: 2, 3}", |e| matches!(e, ParseError::ExpectedColon(_)));
    check_error("{1: 2 3: 4}", |e| matches!(e, ParseError::ExpectedComma(_)));
    check_error("1([1, 2, 3]", |e| {
        matches!(e, ParseError::UnmatchedParentheses(_))
    });
    check_error("{1: 2, 3: 4", |e| {
        matches!(e, ParseError::UnmatchedBraces(_))
    });
    check_error("{1: 2, 3:}", |e| matches!(e, ParseError::ExpectedMapKey(_)));
    check_error("20000000000000000000(1)", |e| {
        matches!(e, ParseError::InvalidTagValue(_, _))
    });
    check_error("foobar(1)", |e| {
        matches!(e, ParseError::UnknownTagName(_, _))
    });
    check_error("h'01020'", |e| matches!(e, ParseError::InvalidHexString(_)));
    check_error("b64'AQIDBAUGBwgJCg'", |e| {
        matches!(e, ParseError::InvalidBase64String(_))
    });
    check_error("ur:foobar/cyisdadmlasgtapttl", |e| {
        matches!(e, ParseError::UnknownUrType(_, _))
    });
    check_error("ur:date/cyisdadmlasgtapttx", |e| {
        matches!(e, ParseError::InvalidUr(_, _))
    });
    check_error("'20000000000000000000'", |e| {
        matches!(e, ParseError::InvalidKnownValue(_, _))
    });
    check_error("'foobar'", |e| {
        matches!(e, ParseError::UnknownKnownValueName(_, _))
    });

    // Test invalid date literals
    check_error("2023-13-01", |e| {
        matches!(e, ParseError::InvalidDateString(_, _))
    });
    check_error("2023-02-30", |e| {
        matches!(e, ParseError::InvalidDateString(_, _))
    });
}

#[test]
fn test_whitespace() {
    let src = indoc! {r#"
        {
            "Hello":
                "World"
        }
    "#}
    .trim();
    let result = parse_dcbor_item(src).unwrap();
    println!("{}", result.diagnostic());
}

#[test]
fn test_whitespace_2() {
    let src = indoc! {r#"
        {"Hello":
        "World"}
    "#}
    .trim();
    let result = parse_dcbor_item(src).unwrap();
    println!("{}", result.diagnostic());
}

#[test]
fn test_inline_comments() {
    let src = "/this is a comment/ [1, /ignore me/ 2, 3]";
    let result = parse_dcbor_item(src).unwrap();
    assert_eq!(result, vec![1, 2, 3].into());
}

#[test]
fn test_end_of_line_comments() {
    let src = "[1, 2, 3] # this should be ignored";
    let result = parse_dcbor_item(src).unwrap();
    assert_eq!(result, vec![1, 2, 3].into());
}

#[test]
fn test_parse_partial_basic() {
    let (cbor, used) = parse_dcbor_item_partial("true )").unwrap();
    assert_eq!(cbor, CBOR::from(true));
    assert_eq!(used, 5);
}

#[test]
fn test_parse_partial_trailing_ws() {
    let src = "false  # comment\n";
    let (cbor, used) = parse_dcbor_item_partial(src).unwrap();
    assert_eq!(cbor, CBOR::from(false));
    assert_eq!(used, src.len());
}

#[test]
fn test_date_literals() {
    dcbor::register_tags();

    // Test parsing a simple date
    let date_cbor = parse_dcbor_item("2023-02-08").unwrap();
    let expected_date = dcbor::Date::from_ymd(2023, 2, 8);
    assert_eq!(date_cbor, expected_date.to_cbor());

    // Test parsing a date-time
    let datetime_cbor = parse_dcbor_item("2023-02-08T15:30:45Z").unwrap();
    let expected_datetime = dcbor::Date::from_ymd_hms(2023, 2, 8, 15, 30, 45);
    assert_eq!(datetime_cbor, expected_datetime.to_cbor());

    // Test parsing an array with dates (the main goal)
    let array_cbor =
        parse_dcbor_item("[1965-05-15, 2000-07-25, 2004-10-30]").unwrap();
    let expected_array = vec![
        dcbor::Date::from_ymd(1965, 5, 15).to_cbor(),
        dcbor::Date::from_ymd(2000, 7, 25).to_cbor(),
        dcbor::Date::from_ymd(2004, 10, 30).to_cbor(),
    ];
    assert_eq!(array_cbor, expected_array.to_cbor());

    // Test that the diagnostic output doesn't have quotes (they're not strings)
    let diagnostic = array_cbor.diagnostic();
    assert!(
        !diagnostic.contains('"'),
        "Date literals should not be quoted in diagnostic output"
    );
}

#[test]
fn test_date_literals_extended() {
    dcbor::register_tags();

    // Test date with time including seconds and timezone
    let datetime_with_tz =
        parse_dcbor_item("2023-02-08T15:30:45+01:00").unwrap();
    println!(
        "Parsed datetime with timezone: {}",
        datetime_with_tz.diagnostic()
    );

    // Test date in a map
    let map_with_dates =
        parse_dcbor_item(r#"{"start": 2023-01-01, "end": 2023-12-31}"#)
            .unwrap();
    println!("Parsed map with dates: {}", map_with_dates.diagnostic());

    // Test nested structure with dates
    let nested = parse_dcbor_item(r#"{"events": [2023-01-01T00:00:00Z, 2023-06-15T12:30:00Z], "metadata": {"created": 2023-02-08}}"#).unwrap();
    println!("Parsed nested structure: {}", nested.diagnostic());
}

#[test]
fn test_date_literal_errors() {
    dcbor::register_tags();

    // Test invalid date format
    let result = parse_dcbor_item("2023-13-01"); // Invalid month
    match result {
        Err(dcbor_parse::ParseError::InvalidDateString(_, _)) => {
            // Expected error
        }
        _ => panic!("Expected InvalidDateString error for invalid date"),
    }

    // Test incomplete date
    let result = parse_dcbor_item("2023-02"); // Incomplete date
    match result {
        Err(_) => {
            // Expected some kind of error
        }
        Ok(_) => panic!("Expected error for incomplete date"),
    }
}

#[test]
fn test_user_requested_example() {
    dcbor::register_tags();

    // Test the exact example from the user's request
    let array_result = parse_dcbor_item("[1965-05-15, 2000-07-25, 2004-10-30]");
    assert!(
        array_result.is_ok(),
        "Should parse array of date literals successfully"
    );

    let cbor = array_result.unwrap();
    let diagnostic = cbor.diagnostic();

    // Verify the dates are parsed as Date objects (tag 1) not strings
    assert!(
        diagnostic.contains("1("),
        "Should contain CBOR tag 1 for dates"
    );
    assert!(
        !diagnostic.contains('"'),
        "Should not contain quotes (dates are not strings)"
    );

    // Verify this is equivalent to manually creating the same dates
    let expected = vec![
        dcbor::Date::from_ymd(1965, 5, 15).to_cbor(),
        dcbor::Date::from_ymd(2000, 7, 25).to_cbor(),
        dcbor::Date::from_ymd(2004, 10, 30).to_cbor(),
    ];
    assert_eq!(cbor, expected.to_cbor());

    println!("Successfully parsed: {}", diagnostic);
}

#[test]
fn test_date_literals_with_milliseconds() {
    dcbor::register_tags();

    // Test date with milliseconds
    let datetime_with_ms =
        parse_dcbor_item("2023-02-08T15:30:45.123Z").unwrap();
    println!(
        "Parsed datetime with milliseconds: {}",
        datetime_with_ms.diagnostic()
    );

    // Test that it's a valid date object
    let expected =
        dcbor::Date::from_string("2023-02-08T15:30:45.123Z").unwrap();
    assert_eq!(datetime_with_ms, expected.to_cbor());
}

#[test]
fn test_date_vs_number_precedence() {
    dcbor::register_tags();

    // Test that pure numbers still work
    let number_result = parse_dcbor_item("2023").unwrap();
    assert_eq!(number_result, CBOR::from(2023));

    // Test that date format is recognized as date, not number
    let date_result = parse_dcbor_item("2023-01-01").unwrap();
    let expected_date = dcbor::Date::from_ymd(2023, 1, 1);
    assert_eq!(date_result, expected_date.to_cbor());

    // Ensure they produce different results
    assert_ne!(number_result, date_result);
}

#[test]
fn test_duplicate_map_keys() {
    // Test string key duplicates
    let result = parse_dcbor_item(r#"{"key1": 1, "key2": 2, "key1": 3}"#);
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::DuplicateMapKey(_) => {} // Expected
        e => panic!("Expected DuplicateMapKey error, got: {:?}", e),
    }

    // Test integer key duplicates
    let result =
        parse_dcbor_item("{1: \"value1\", 2: \"value2\", 1: \"value3\"}");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::DuplicateMapKey(_) => {} // Expected
        e => panic!("Expected DuplicateMapKey error, got: {:?}", e),
    }

    // Test mixed type duplicates - integers and floats with same numeric value
    // are considered duplicates
    let result =
        parse_dcbor_item("{1: \"value1\", 2: \"value2\", 1.0: \"value3\"}");
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::DuplicateMapKey(_) => {} /* Expected - 1 and 1.0 are
                                               * semantically the same key */
        e => panic!("Expected DuplicateMapKey error, got: {:?}", e),
    }

    // Test that non-duplicate keys work fine
    let result = parse_dcbor_item(r#"{"key1": 1, "key2": 2, "key3": 3}"#);
    assert!(result.is_ok());

    let result =
        parse_dcbor_item("{1: \"value1\", 2: \"value2\", 3: \"value3\"}");
    assert!(result.is_ok());

    // Test that integer and float with different values are allowed
    let result = parse_dcbor_item("{1: \"value1\", 2.0: \"value2\"}");
    assert!(result.is_ok());
}

#[test]
fn test_duplicate_key_error_location() {
    let input = r#"{"key1": 1, "key2": 2, "key1": 3}"#;
    let result = parse_dcbor_item(input);
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::DuplicateMapKey(span) => {
            // The error should point to the second occurrence of "key1"
            assert_eq!(span.start, 23); // Position of the duplicate "key1"
            assert_eq!(span.end, 29); // End of the duplicate "key1"

            // Test error message formatting
            let error = ParseError::DuplicateMapKey(span);
            let full_message = error.full_message(input);
            assert!(full_message.contains("Duplicate map key"));
            assert!(full_message.contains("^")); // Should show caret pointing to the error
        }
        e => panic!("Expected DuplicateMapKey error, got: {:?}", e),
    }
}
