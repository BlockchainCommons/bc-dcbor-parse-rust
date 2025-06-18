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
