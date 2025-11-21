use bc_ur::prelude::*;
use known_values::KnownValue;
use logos::{Lexer, Logos, Span};

use crate::{
    Token,
    error::{Error, Result},
};

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
    let first_token = expect_token(&mut lexer);
    match first_token {
        Ok(token) => parse_item_token(&token, &mut lexer).and_then(|cbor| {
            if lexer.next().is_some() {
                Err(Error::ExtraData(lexer.span()))
            } else {
                Ok(cbor)
            }
        }),
        Err(e) => {
            if e == Error::UnexpectedEndOfInput {
                return Err(Error::EmptyInput);
            }
            Err(e)
        }
    }
}

/// Parses a dCBOR item from the beginning of a string and returns the parsed
/// [`CBOR`] along with the number of bytes consumed.
///
/// Unlike [`parse_dcbor_item`], this function succeeds even if additional
/// characters follow the first item. The returned index points to the first
/// unparsed character after skipping any trailing whitespace or comments.
///
/// # Example
///
/// ```rust
/// # use dcbor_parse::parse_dcbor_item_partial;
/// # use dcbor::prelude::*;
/// let (cbor, used) = parse_dcbor_item_partial("true )").unwrap();
/// assert_eq!(cbor, CBOR::from(true));
/// assert_eq!(used, 5);
/// ```
pub fn parse_dcbor_item_partial(src: &str) -> Result<(CBOR, usize)> {
    let mut lexer = Token::lexer(src);
    let first_token = expect_token(&mut lexer);
    match first_token {
        Ok(token) => parse_item_token(&token, &mut lexer).map(|cbor| {
            let consumed = match lexer.next() {
                Some(_) => lexer.span().start,
                None => src.len(),
            };
            (cbor, consumed)
        }),
        Err(e) => {
            if e == Error::UnexpectedEndOfInput {
                Err(Error::EmptyInput)
            } else {
                Err(e)
            }
        }
    }
}

//
// === Private Functions ===
//

fn parse_item(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let token = expect_token(lexer)?;
    parse_item_token(&token, lexer)
}

fn expect_token(lexer: &mut Lexer<'_, Token>) -> Result<Token> {
    let span = lexer.span();
    match lexer.next() {
        Some(token_or_err) => match token_or_err {
            Ok(token) => Ok(token),
            Err(e) => {
                if e.is_default() {
                    Err(Error::UnrecognizedToken(span))
                } else {
                    Err(e)
                }
            }
        },
        None => Err(Error::UnexpectedEndOfInput),
    }
}

fn parse_item_token(
    token: &Token,
    lexer: &mut Lexer<'_, Token>,
) -> Result<CBOR> {
    // Handle embedded lexing errors in token payloads
    if let Token::ByteStringHex(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::ByteStringBase64(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::DateLiteral(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::TagValue(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::UR(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::KnownValueNumber(Err(e)) = token {
        return Err(e.clone());
    }

    match token {
        Token::Bool(b) => Ok((*b).into()),
        Token::Null => Ok(CBOR::null()),
        Token::ByteStringHex(Ok(bytes)) => Ok(CBOR::to_byte_string(bytes)),
        Token::ByteStringBase64(Ok(bytes)) => Ok(CBOR::to_byte_string(bytes)),
        Token::DateLiteral(Ok(date)) => Ok((*date).into()),
        Token::Number(num) => Ok((*num).into()),
        Token::NaN => Ok(f64::NAN.into()),
        Token::Infinity => Ok(f64::INFINITY.into()),
        Token::NegInfinity => Ok(f64::NEG_INFINITY.into()),
        Token::String(s) => parse_string(s, lexer.span()),
        Token::UR(Ok(ur)) => parse_ur(ur, lexer.span()),
        Token::TagValue(Ok(tag_value)) => parse_number_tag(*tag_value, lexer),
        Token::TagName(name) => parse_name_tag(name, lexer),
        Token::KnownValueNumber(Ok(value)) => {
            Ok(KnownValue::new(*value).into())
        }
        Token::KnownValueName(name) => {
            if let Some(known_value) = known_value_for_name(name) {
                Ok(known_value.into())
            } else {
                let span = lexer.span().start + 1..lexer.span().end - 1;
                Err(Error::UnknownKnownValueName(name.clone(), span))
            }
        }
        Token::Unit => Ok(KnownValue::new(0).into()),
        Token::BracketOpen => parse_array(lexer),
        Token::BraceOpen => parse_map(lexer),
        _ => Err(Error::UnexpectedToken(
            Box::new(token.clone()),
            lexer.span(),
        )),
    }
}

fn parse_string(s: &str, span: Span) -> Result<CBOR> {
    if s.starts_with('"') && s.ends_with('"') {
        let s = &s[1..s.len() - 1];
        Ok(s.into())
    } else {
        Err(Error::UnrecognizedToken(span))
    }
}

fn tag_for_name(name: &str) -> Option<Tag> {
    with_tags!(|tags: &TagsStore| tags.tag_for_name(name))
}

fn known_value_for_name(name: &str) -> Option<KnownValue> {
    let binding = known_values::KNOWN_VALUES.get();
    let known_values = binding.as_ref().unwrap();
    known_values.known_value_named(name).cloned()
}

fn parse_ur(ur: &UR, span: Span) -> Result<CBOR> {
    let ur_type = ur.ur_type_str();
    if let Some(tag) = tag_for_name(ur_type) {
        Ok(CBOR::to_tagged_value(tag, ur.cbor()))
    } else {
        Err(Error::UnknownUrType(
            ur_type.to_string(),
            span.start + 3..span.start + 3 + ur_type.len(),
        ))
    }
}

fn parse_number_tag(
    tag_value: TagValue,
    lexer: &mut Lexer<'_, Token>,
) -> Result<CBOR> {
    let item = parse_item(lexer)?;
    match expect_token(lexer) {
        Ok(Token::ParenthesisClose) => {
            Ok(CBOR::to_tagged_value(tag_value, item))
        }
        Ok(_) => Err(Error::UnmatchedParentheses(lexer.span())),
        Err(e) => {
            if e == Error::UnexpectedEndOfInput {
                return Err(Error::UnmatchedParentheses(lexer.span()));
            }
            Err(e)
        }
    }
}

fn parse_name_tag(name: &str, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let span = lexer.span().start..lexer.span().end - 1;
    let item = parse_item(lexer)?;
    match expect_token(lexer)? {
        Token::ParenthesisClose => {
            if let Some(tag) = tag_for_name(name) {
                Ok(CBOR::to_tagged_value(tag, item))
            } else {
                Err(Error::UnknownTagName(name.to_string(), span))
            }
        }
        _ => Err(Error::UnmatchedParentheses(lexer.span())),
    }
}

fn parse_array(lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let mut items = Vec::new();
    let mut awaits_comma = false;
    let mut awaits_item = false;

    loop {
        match expect_token(lexer)? {
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
            Token::DateLiteral(Ok(date)) if !awaits_comma => {
                items.push(date.into());
                awaits_item = false;
            }
            Token::Number(num) if !awaits_comma => {
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
            Token::TagValue(Ok(tag_value)) if !awaits_comma => {
                items.push(parse_number_tag(tag_value, lexer)?);
                awaits_item = false;
            }
            Token::TagName(name) if !awaits_comma => {
                items.push(parse_name_tag(&name, lexer)?);
                awaits_item = false;
            }
            Token::KnownValueNumber(Ok(value)) if !awaits_comma => {
                items.push(KnownValue::new(value).into());
                awaits_item = false;
            }
            Token::KnownValueName(name) if !awaits_comma => {
                if let Some(known_value) = known_value_for_name(&name) {
                    items.push(known_value.into());
                } else {
                    return Err(Error::UnknownKnownValueName(
                        name,
                        lexer.span(),
                    ));
                }
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
            token => {
                if awaits_comma {
                    return Err(Error::ExpectedComma(lexer.span()));
                }
                return Err(Error::UnexpectedToken(
                    Box::new(token),
                    lexer.span(),
                ));
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
        let token = match expect_token(lexer) {
            Ok(tok) => tok,
            Err(Error::UnexpectedEndOfInput) => {
                return Err(Error::UnmatchedBraces(lexer.span()));
            }
            Err(e) => {
                return Err(e);
            }
        };
        match token {
            Token::BraceClose if !awaits_key => {
                return Ok(map.into());
            }
            Token::Comma if awaits_comma => {
                awaits_key = true;
            }
            _ => {
                if awaits_comma {
                    return Err(Error::ExpectedComma(lexer.span()));
                }
                let key = parse_item_token(&token, lexer)?;
                let key_span = lexer.span();

                // Check for duplicate key
                if map.contains_key(key.clone()) {
                    return Err(Error::DuplicateMapKey(key_span));
                }

                if let Ok(Token::Colon) = expect_token(lexer) {
                    let value = match parse_item(lexer) {
                        Err(Error::UnexpectedToken(token, span))
                            if *token == Token::BraceClose =>
                        {
                            return Err(Error::ExpectedMapKey(span));
                        }
                        other => other?,
                    };
                    map.insert(key, value);
                    awaits_key = false;
                } else {
                    return Err(Error::ExpectedColon(lexer.span()));
                }
            }
        }
        awaits_comma = !awaits_key;
    }
}
