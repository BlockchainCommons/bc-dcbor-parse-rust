use base64::Engine as _;
use bc_ur::prelude::*;
use logos::{ Lexer, Logos, Span };
use crate::{ Result, Error };

/// Parses a dCBOR item from a string input.
///
/// This function takes a string slice containing a dCBOR diagnostic notation
/// encoded value and attempts to parse it into a `CBOR` object. If the input
/// contains extra tokens after a valid item, an error is returned.
///
/// # Arguments
///
/// * `src` - A string slice containing the dCBOR-encoded data.
///
/// # Returns
///
/// * `Ok(CBOR)` if parsing is successful and the input contains exactly one
///   valid dCBOR item, which itself might be an atomic value like a number or
///   string, or a complex value like an array or map.
/// * `Err(Error)` if parsing fails or if extra tokens are found after the item.
///
/// # Errors
///
/// Returns an error if the input is invalid, contains extra tokens, or if any
/// token cannot be parsed as expected.
///
/// # Example
///
/// ```rust
/// # use dcbor_parse::parse_dcbor_item;
/// let cbor = parse_dcbor_item("[1, 2, 3]").unwrap();
/// assert_eq!(cbor.diagnostic(), "[1, 2, 3]");
/// ```
pub fn parse_dcbor_item(src: &str) -> Result<CBOR> {
    let mut lexer = Token::lexer(src);
    parse_item(&mut lexer).and_then(|cbor| {
        if lexer.next().is_some() {
            Err(Error::new("Extra tokens found", lexer.span()))
        } else {
            Ok(cbor)
        }
    })
}

//
// === Private Functions ===
//

fn parse_item(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let token = expect_token(lexer)?;
    parse_item_token(&token, lexer)
}

fn expect_token(lexer: &mut Lexer<'_, Token>) -> Result<Token> {
    if let Some(token) = lexer.next() {
        return token;
    } else {
        return Err(Error::new("Unexpected end of input", lexer.span()));
    }
}

fn parse_item_token(token: &Token, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    match token {
        Token::Bool(b) => Ok((*b).into()),
        Token::Null => Ok(CBOR::null()),
        Token::ByteStringHex(Ok(bytes)) => Ok(CBOR::to_byte_string(bytes)),
        Token::ByteStringBase64(Ok(bytes)) => Ok(CBOR::to_byte_string(bytes)),
        Token::Number(Ok(num)) => Ok((*num).into()),
        Token::NaN => Ok(f64::NAN.into()),
        Token::Infinity => Ok(f64::INFINITY.into()),
        Token::NegInfinity => Ok(f64::NEG_INFINITY.into()),
        Token::String(s) => parse_string(s, lexer.span()),
        Token::UR(Ok(ur)) => parse_ur(ur, lexer.span()),
        Token::TagNumber(Ok(tag_value)) => parse_number_tag(*tag_value, lexer),
        Token::TagName(Ok(name)) => parse_name_tag(name, lexer),
        Token::BracketOpen => parse_array(lexer),
        Token::BraceOpen => parse_map(lexer),
        _ => Err(Error::new(format!("Unexpected token: {:?}", token), lexer.span())),
    }
}

fn parse_string(s: &str, span: Span) -> Result<CBOR> {
    if s.starts_with('"') && s.ends_with('"') {
        let s = &s[1..s.len() - 1];
        Ok(s.into())
    } else {
        Err(Error::new(format!("Invalid string: {}", s), span))
    }
}

fn tag_for_name(name: &str) -> Option<Tag> {
    with_tags!(|tags: &TagsStore| tags.tag_for_name(name))
}

fn parse_ur(ur: &UR, span: Span) -> Result<CBOR> {
    let ur_type = ur.ur_type_str();
    if let Some(tag) = tag_for_name(ur_type) {
        Ok(CBOR::to_tagged_value(tag, ur.cbor()))
    } else {
        Err(Error::new(format!("Unknown UR type: {}", ur_type), span))
    }
}

fn parse_number_tag(tag_value: TagValue, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let item = parse_item(lexer)?;
    match lexer.next() {
        Some(Ok(Token::ParenthesisClose)) => { Ok(CBOR::to_tagged_value(tag_value, item)) }
        Some(token) => {
            Err(
                Error::new(
                    format!("Unexpected token while parsing tagged value: {:?}", token),
                    lexer.span()
                )
            )
        }
        None => Err(Error::new("End of tokens while parsing tagged value", lexer.span())),
    }
}

fn parse_name_tag(name: &str, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let item = parse_item(lexer)?;
    match lexer.next() {
        Some(Ok(Token::ParenthesisClose)) => {
            if let Some(tag) = tag_for_name(name) {
                Ok(CBOR::to_tagged_value(tag, item))
            } else {
                Err(Error::new(format!("Unknown tag name: {}", name), lexer.span()))
            }
        }
        Some(token) => {
            Err(
                Error::new(
                    format!("Unexpected token while parsing tagged value: {:?}", token),
                    lexer.span()
                )
            )
        }
        None => Err(Error::new("End of tokens while parsing tagged value", lexer.span())),
    }
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
            Token::ByteStringHex(Ok(bytes)) if !awaits_comma => {
                items.push(CBOR::to_byte_string(bytes));
                awaits_item = false;
            }
            Token::ByteStringBase64(Ok(bytes)) if !awaits_comma => {
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
                items.push(parse_string(&s, lexer.span())?);
                awaits_item = false;
            }
            Token::UR(Ok(ur)) if !awaits_comma => {
                items.push(parse_ur(&ur, lexer.span())?);
                awaits_item = false;
            }
            Token::TagNumber(Ok(tag_value)) if !awaits_comma => {
                items.push(parse_number_tag(tag_value, lexer)?);
                awaits_item = false;
            }
            Token::TagName(Ok(name)) if !awaits_comma => {
                items.push(parse_name_tag(&name, lexer)?);
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
                Error::new(format!("Invalid hex string: {}", &hex[2..hex.len() - 1]), lex.span())
            )
    })]
    ByteStringHex(Result<Vec<u8>>),

    /// Binary string in base64 format.
    #[regex(r"b64'([A-Za-z0-9+/=]{2,})'", |lex| {
        let base64 = lex.slice();
        let s = &base64[4..base64.len() - 1];
        base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|_| Error::new(format!("Invalid base64 string: {}", s), lex.span()))
    })]
    ByteStringBase64(Result<Vec<u8>>),

    /// JavaScript-style number.
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex|
        lex.slice()
            .parse::<f64>()
            .map_err(|_|
                Error::new(format!("Invalid number: {}", lex.slice()), lex.span())
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
                Error::new(format!("Invalid tag number: {}", lex.slice()), lex.span())
            )
            .and_then(|s| s.parse::<TagValue>().map_err(|_|
                Error::new(format!("Invalid tag number: {}", s), lex.span())
            ))
    )]
    TagNumber(Result<TagValue>),

    /// Tag name followed immediately by an opening parenthesis.
    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_-]*\("#, |lex|
        lex.slice()
            .strip_suffix('(')
            .ok_or_else(||
                Error::new(format!("Invalid tag name: {}", lex.slice()), lex.span())
            )
            .map(|s| s.to_string())
    )]
    TagName(Result<String>),

    #[regex(r#"ur:([a-zA-Z0-9][a-zA-Z0-9-]*)/([a-zA-Z]{8,})"#, |lex|
        let s = lex.slice();
        let ur = UR::from_ur_string(s);
        ur.map_err(|e| {
            Error::new(format!("Invalid UR: {}", e), lex.span())
        })
    )]
    UR(Result<UR>),
}
