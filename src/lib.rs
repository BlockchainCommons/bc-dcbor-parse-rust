use logos::{ Lexer, Logos, Span };
use dcbor::prelude::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Error(String, Span);
impl Error {
    pub fn new(msg: impl Into<String>, span: Span) -> Self {
        Self(msg.into(), span)
    }

    pub fn message(&self) -> &str {
        &self.0
    }

    pub fn span(&self) -> &Span {
        &self.1
    }
}

type Result<T> = std::result::Result<T, Error>;

/// All meaningful JSON tokens.
#[derive(Debug, Logos)]
#[rustfmt::skip]
#[logos(error = Error)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token("(")]
    ParenthesisOpen,

    #[token(")")]
    ParenthesisClose,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("null")]
    Null,

    #[token("NaN")]
    NaN,

    #[token("Infinity")]
    Infinity,

    #[token("-Infinity")]
    NegInfinity,

    /// Binary string in hex format.
    #[regex(r"h'[0-9a-fA-F]{2,}'", |lex| {
        let hex = lex.slice();
        hex::decode(hex[2..hex.len() - 1].as_bytes())
            .map_err(|_|
                Error(format!("Invalid hex string: {}", &hex[2..hex.len() - 1]), lex.span())
            )
    })]
    ByteString(Result<Vec<u8>>),

    /// JavaScript-style number.
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex|
        lex.slice()
            .parse::<f64>()
            .map_err(|_|
                Error(format!("Invalid number: {}", lex.slice()), lex.span())
            )
    )]
    Number(Result<f64>),

    /// JavaScript-style string.
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex|
        lex.slice().to_owned()
    )]
    String(String),

    /// Integer followed immediately by an opening parenthesis.
    #[regex(r#"[1-9][0-9]*\("#, |lex|
        lex.slice()
            .strip_suffix('(')
            .ok_or_else(||
                Error(format!("Invalid tag number: {}", lex.slice()), lex.span())
            )
            .and_then(|s| s.parse::<TagValue>().map_err(|_|
                Error(format!("Invalid tag number: {}", s), lex.span())
            ))
    )]
    TagNumber(Result<TagValue>),

    /// Tag name followed immediately by an opening parenthesis.
    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_-]*\("#, |lex|
        lex.slice()
            .strip_suffix('(')
            .ok_or_else(||
                Error(format!("Invalid tag name: {}", lex.slice()), lex.span())
            )
            .map(|s| s.to_string())
    )]
    #[allow(dead_code)]
    TagName(Result<String>),
}

fn expect_token(lexer: &mut Lexer<'_, Token>) -> Result<Token> {
    if let Some(token) = lexer.next() {
        return token;
    } else {
        return Err(Error::new("Unexpected end of input", lexer.span()));
    }
}

fn parse_tagged_value(tag_value: TagValue, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let item = parse_item(lexer)?;
    match lexer.next() {
        Some(Ok(Token::ParenthesisClose)) => { Ok(CBOR::to_tagged_value(tag_value, item)) }
        Some(token) => {
            Err(
                Error(
                    format!("Unexpected token while parsing tagged value: {:?}", token),
                    lexer.span()
                )
            )
        }
        None => Err(Error::new("End of tokens while parsing tagged value", lexer.span())),
    }
}

fn parse_item_token(token: &Token, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    match token {
        Token::Bool(b) => Ok((*b).into()),
        Token::Null => Ok(CBOR::null()),
        Token::ByteString(Ok(bytes)) => Ok(CBOR::to_byte_string(bytes)),
        Token::Number(Ok(num)) => Ok((*num).into()),
        Token::NaN => Ok(f64::NAN.into()),
        Token::Infinity => Ok(f64::INFINITY.into()),
        Token::NegInfinity => Ok(f64::NEG_INFINITY.into()),
        Token::String(s) => {
            // Remove quotes
            let s = s[1..s.len() - 1].to_string();
            Ok(s.into())
        }
        Token::TagNumber(Ok(tag_value)) => parse_tagged_value(*tag_value, lexer),
        Token::BracketOpen => parse_array(lexer),
        Token::BraceOpen => parse_map(lexer),
        _ => Err(Error::new(format!("Unexpected token: {:?}", token), lexer.span())),
    }
}

fn parse_item(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let token = expect_token(lexer)?;
    parse_item_token(&token, lexer)
}

fn parse_array(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let mut items = Vec::new();
    let mut awaits_comma = false;
    let mut awaits_item = false;

    loop {
        let token = expect_token(lexer)?;
        match token {
            Token::Bool(b) if !awaits_comma => {
                items.push(b.into());
                awaits_item = false;
            }
            Token::Null if !awaits_comma => {
                items.push(CBOR::null());
                awaits_item = false;
            }
            Token::ByteString(Ok(bytes)) if !awaits_comma => {
                items.push(CBOR::to_byte_string(bytes));
                awaits_item = false;
            }
            Token::Number(Ok(num)) if !awaits_comma => {
                items.push(num.into());
                awaits_item = false;
            }
            Token::NaN if !awaits_comma => {
                items.push(f64::NAN.into());
                awaits_item = false;
            }
            Token::Infinity if !awaits_comma => {
                items.push(f64::INFINITY.into());
                awaits_item = false;
            }
            Token::NegInfinity if !awaits_comma => {
                items.push(f64::NEG_INFINITY.into());
                awaits_item = false;
            }
            Token::String(s) if !awaits_comma => {
                // Remove quotes
                let s = s[1..s.len() - 1].to_string();
                items.push(s.into());
                awaits_item = false;
            }
            Token::TagNumber(Ok(tag_value)) if !awaits_comma => {
                items.push(parse_tagged_value(tag_value, lexer)?);
                awaits_item = false;
            }
            Token::BracketOpen if !awaits_comma => {
                items.push(parse_array(lexer)?);
                awaits_item = false;
            }
            Token::BraceOpen if !awaits_comma => {
                items.push(parse_map(lexer)?);
                awaits_item = false;
            }
            Token::Comma if awaits_comma => {
                awaits_item = true;
            }
            Token::BracketClose if !awaits_item => {
                return Ok(items.into());
            }
            _ => {
                return Err(
                    Error::new(
                        format!("Unexpected token while parsing array: {:?}", token),
                        lexer.span()
                    )
                );
            }
        }
        awaits_comma = !awaits_item;
    }
}

fn parse_map(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let mut map = Map::new();
    let mut awaits_comma = false;
    let mut awaits_key = false;

    loop {
        let token = expect_token(lexer)?;
        match token {
            Token::BraceClose if !awaits_key => {
                return Ok(map.into());
            }
            Token::Comma if awaits_comma => {
                awaits_key = true;
            }
            _ => {
                let key = parse_item_token(&token, lexer)?;
                if let Some(Token::Colon) = expect_token(lexer).ok() {
                    let value = parse_item(lexer)?;
                    map.insert(key, value);
                    awaits_key = false;
                } else {
                    return Err(
                        Error::new(format!("Expected colon after key: {:?}", token), lexer.span())
                    );
                }
            }
        }
        awaits_comma = !awaits_key;
    }
}

pub fn parse_diagnostic(src: &str) -> Result<CBOR> {
    let mut lexer = Token::lexer(src);
    parse_item(&mut lexer).and_then(|cbor| {
        if lexer.next().is_some() {
            Err(Error::new("Extra tokens found", lexer.span()))
        } else {
            Ok(cbor)
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn roundtrip<T: Into<CBOR>>(value: T) {
        let cbor = value.into();
        // println!("=== Original ===\n{}", cbor.diagnostic());
        let src = cbor.diagnostic();
        match parse_diagnostic(&src) {
            Ok(result) => {
                println!("{}", result.diagnostic());
                if result != cbor {
                    panic!("=== Expected ===\n{}\n\n=== Got ===\n{}", cbor, result);
                }
            }
            Err(e) => panic!("{:?}: Failed to parse: {}", e.span(), e.message()),
        }
    }

    #[test]
    fn test_parse_atomic_types() {
        roundtrip(true);
        roundtrip(false);
        roundtrip(CBOR::null());
        roundtrip(CBOR::to_byte_string(vec![0x01, 0x02, 0x03]));
        roundtrip(10);
        roundtrip(3.14);
        roundtrip(f64::INFINITY);
        roundtrip(f64::NEG_INFINITY);
        roundtrip("Hello, world!");
    }

    #[test]
    fn test_nan() {
        // NaN is a special case because it doesn't equal itself
        let cbor = CBOR::from(f64::NAN);
        let src = cbor.diagnostic();
        assert_eq!(src, "NaN");
        let cbor2 = parse_diagnostic(&src).unwrap();
        assert!(f64::try_from(cbor2).unwrap().is_nan());
    }

    #[test]
    fn test_tagged() {
        roundtrip(CBOR::to_tagged_value(1234, CBOR::to_byte_string(vec![0x01, 0x02, 0x03])));
        roundtrip(CBOR::to_tagged_value(5678, "Hello, world!"));
        roundtrip(CBOR::to_tagged_value(9012, true));
    }

    #[test]
    fn test_array() {
        let v: Vec<i32> = vec![];
        roundtrip(v);

        roundtrip(vec![1, 2, 3]);
        roundtrip(vec![true.to_cbor(), false.to_cbor(), CBOR::null()]);
        roundtrip(vec![CBOR::to_byte_string(vec![0x01, 0x02]).to_cbor(), "Hello".to_cbor()]);
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
            CBOR::to_tagged_value(1234, CBOR::to_byte_string(vec![0x01, 0x02, 0x03])),
            vec![1, 2, 3].to_cbor(),
            HashMap::from([
                ("key1", "value1".to_cbor()),
                ("key2", vec![4, 5, 6].to_cbor()),
            ]).to_cbor()
        ];
        roundtrip(nested);
    }
}
