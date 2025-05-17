use dcbor_parse::*;

fn roundtrip_array(array: &[&str], expected_diag: &str) {
    let cbor = compose_dcbor_array(array).unwrap();
    let diag = cbor.diagnostic_flat();
    // println!("{}", diag);
    assert_eq!(diag, expected_diag);
    let cbor2 = parse_dcbor_item(&diag).unwrap();
    assert_eq!(cbor, cbor2);
}

fn roundtrip_map(array: &[&str], expected_diag: &str) {
    let cbor = compose_dcbor_map(array).unwrap();
    let diag = cbor.diagnostic_flat();
    // println!("{}", diag);
    assert_eq!(diag, expected_diag);
    let cbor2 = parse_dcbor_item(&diag).unwrap();
    assert_eq!(cbor, cbor2);
}

#[test]
fn test_compose_array() {
    // Empty array
    let array = vec![];
    let expected_diag = "[]";
    roundtrip_array(&array, expected_diag);

    // Integers
    let array = vec!["1", "2", "3"];
    let expected_diag = "[1, 2, 3]";
    roundtrip_array(&array, expected_diag);

    // Strings
    let array = vec![r#""hello""#, r#""world""#];
    let expected_diag = r#"["hello", "world"]"#;
    roundtrip_array(&array, expected_diag);

    // Mixed types
    let array = vec!["true", "false", "null", "3.14"];
    let expected_diag = "[true, false, null, 3.14]";
    roundtrip_array(&array, expected_diag);

    // Nested arrays
    let array = vec!["[1, 2]", "[3, 4]"];
    let expected_diag = "[[1, 2], [3, 4]]";
    roundtrip_array(&array, expected_diag);

    // Error: Empty item in array
    let array = vec!["1", "2", "", "4"];
    let err = compose_dcbor_array(&array).unwrap_err();
    assert!(matches!(err, ComposeError::ParseError(ParseError::EmptyInput)));
}

#[test]
fn test_compose_map() {
    // Empty map
    let array = vec![];
    let expected_diag = "{}";
    roundtrip_map(&array, expected_diag);

    // Integer keys and values
    let array = vec!["1", "2", "3", "4"];
    let expected_diag = "{1: 2, 3: 4}";
    roundtrip_map(&array, expected_diag);

    // Mixed keys and values
    let array = vec!["true", "false", "null", "null"];
    let expected_diag = "{true: false, null: null}";
    roundtrip_map(&array, expected_diag);

    // String keys and values
    let array = vec![r#""key1""#, r#""value1""#, r#""key2""#, r#""value2""#];
    let expected_diag = r#"{"key1": "value1", "key2": "value2"}"#;
    roundtrip_map(&array, expected_diag);

    // Nested maps
    let array = vec!["{1: 2}", "{3: 4}"];
    let expected_diag = "{{1: 2}: {3: 4}}";
    roundtrip_map(&array, expected_diag);

    // Sorted keys
    let array = vec!["3", "4", "1", "2"];
    let expected_diag = "{1: 2, 3: 4}";
    roundtrip_map(&array, expected_diag);

    // Duplicate keys (last wins)
    let array = vec!["1", "2", "1", "3"];
    let expected_diag = "{1: 3}";
    roundtrip_map(&array, expected_diag);

    // Error: Odd number of items in map
    let array = vec!["1", "2", "3"];
    let err = compose_dcbor_map(&array).unwrap_err();
    assert!(matches!(err, ComposeError::OddMapLength));

    // Error: Empty item in map
    let array = vec!["1", "2", "", "4"];
    let err = compose_dcbor_map(&array).unwrap_err();
    assert!(matches!(err, ComposeError::ParseError(ParseError::EmptyInput)));
}
