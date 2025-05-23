use base64::Engine as _;
use bc_ur::prelude::*;
use known_values::KnownValue;
use logos::{ Lexer, Logos, Span };
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Error {
    #[error("Empty input")]
    EmptyInput,
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Extra data at end of input")]
    ExtraData(Span),
    #[error("Unexpected token {0:?}")]
    UnexpectedToken(Box<Token>, Span),
    #[error("Unrecognized token")]
    UnrecognizedToken(Span),
    #[error("Expected comma")]
    ExpectedComma(Span),
    #[error("Expected colon")]
    ExpectedColon(Span),
    #[error("Unmatched parentheses")]
    UnmatchedParentheses(Span),
    #[error("Unmatched braces")]
    UnmatchedBraces(Span),
    #[error("Expected map key")]
    ExpectedMapKey(Span),
    #[error("Invalid tag value '{0}'")]
    InvalidTagValue(String, Span),
    #[error("Unknown tag name '{0}'")]
    UnknownTagName(String, Span),
    #[error("Invalid hex string")]
    InvalidHexString(Span),
    #[error("Invalid base64 string")]
    InvalidBase64String(Span),
    #[error("Unknown UR type '{0}'")]
    UnknownUrType(String, Span),
    #[error("Invalid UR '{0}'")]
    InvalidUr(String, Span),
    #[error("Invalid known value '{0}'")]
    InvalidKnownValue(String, Span),
    #[error("Unknown known value name '{0}'")]
    UnknownKnownValueName(String, Span),
}

impl Error {
    pub fn is_default(&self) -> bool {
        matches!(self, Error::UnrecognizedToken(_))
    }

    fn format_message(message: &dyn ToString, source: &str, range: &Span) -> String {
        let message = message.to_string();
        let start = range.start;
        let end = range.end;
        // Walk through the bytes up to `start` to find line number and line start offset
        let mut line_number = 1;
        let mut line_start = 0;
        for (idx, ch) in source.char_indices() {
            if idx >= start {
                break;
            }
            if ch == '\n' {
                line_number += 1;
                line_start = idx + 1;
            }
        }
        // Grab the exact line text (or empty if out of bounds)
        let line = source
            .lines()
            .nth(line_number - 1)
            .unwrap_or("");
        // Column is byte-offset into that line
        let column = start.saturating_sub(line_start);
        // Underline at least one caret, even for zero-width spans
        let underline_len = end.saturating_sub(start).max(1);
        let caret = " ".repeat(column) + &"^".repeat(underline_len);
        format!("line {line_number}: {message}\n{line}\n{caret}")
    }

    #[rustfmt::skip]
    pub fn full_message(&self, source: &str) -> String {
        match self {
            Error::EmptyInput => Self::format_message(self, source, &Span::default()),
            Error::UnexpectedEndOfInput => Self::format_message(self, source, &(source.len()..source.len())),
            Error::ExtraData(range) => Self::format_message(self, source, range),
            Error::UnexpectedToken(_, range) => Self::format_message(self, source, range),
            Error::UnrecognizedToken(range) => Self::format_message(self, source, range),
            Error::UnknownUrType(_, range) => Self::format_message(self, source, range),
            Error::UnmatchedParentheses(range) => Self::format_message(self, source, range),
            Error::ExpectedComma(range) => Self::format_message(self, source, range),
            Error::ExpectedColon(range) => Self::format_message(self, source, range),
            Error::ExpectedMapKey(range) => Self::format_message(self, source, range),
            Error::UnmatchedBraces(range) => Self::format_message(self, source, range),
            Error::UnknownTagName(_, range) => Self::format_message(self, source, range),
            Error::InvalidHexString(range) => Self::format_message(self, source, range),
            Error::InvalidBase64String(range) => Self::format_message(self, source, range),
            Error::InvalidTagValue(_, range) => Self::format_message(self, source, range),
            Error::InvalidUr(_, range) => Self::format_message(self, source, range),
            Error::InvalidKnownValue(_, range) => Self::format_message(self, source, range),
            Error::UnknownKnownValueName(_, range) => Self::format_message(self, source, range),
        }
    }
}

impl Default for Error {
    fn default() -> Self {
        Error::UnrecognizedToken(Span::default())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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
        Ok(token) => {
            parse_item_token(&token, &mut lexer).and_then(|cbor| {
                if lexer.next().is_some() { Err(Error::ExtraData(lexer.span())) } else { Ok(cbor) }
            })
        }
        Err(e) => {
            if e == Error::UnexpectedEndOfInput {
                return Err(Error::EmptyInput);
            }
            return Err(e);
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
        Some(token_or_err) => {
            match token_or_err {
                Ok(token) => { Ok(token) }
                Err(e) => {
                    if e.is_default() { Err(Error::UnrecognizedToken(span)) } else { Err(e) }
                }
            }
        }
        None => Err(Error::UnexpectedEndOfInput),
    }
}

fn parse_item_token(token: &Token, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    // Handle embedded lexing errors in token payloads
    if let Token::ByteStringHex(Err(e)) = token {
        return Err(e.clone());
    }
    if let Token::ByteStringBase64(Err(e)) = token {
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
        Token::Number(num) => Ok((*num).into()),
        Token::NaN => Ok(f64::NAN.into()),
        Token::Infinity => Ok(f64::INFINITY.into()),
        Token::NegInfinity => Ok(f64::NEG_INFINITY.into()),
        Token::String(s) => parse_string(s, lexer.span()),
        Token::UR(Ok(ur)) => parse_ur(ur, lexer.span()),
        Token::TagValue(Ok(tag_value)) => parse_number_tag(*tag_value, lexer),
        Token::TagName(name) => parse_name_tag(&name, lexer),
        Token::KnownValueNumber(Ok(value)) => Ok(KnownValue::new(*value).into()),
        Token::KnownValueName(name) => {
            if let Some(known_value) = known_value_for_name(&name) {
                Ok(known_value.into())
            } else {
                let span = lexer.span().start + 1..lexer.span().end - 1;
                Err(Error::UnknownKnownValueName(name.clone(), span))
            }
        }
        Token::Unit => Ok(KnownValue::new(0).into()),
        Token::BracketOpen => parse_array(lexer),
        Token::BraceOpen => parse_map(lexer),
        _ => Err(Error::UnexpectedToken(Box::new(token.clone()), lexer.span())),
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
        Err(
            Error::UnknownUrType(
                ur_type.to_string(),
                span.start + 3..span.start + 3 + ur_type.len()
            )
        )
    }
}

fn parse_number_tag(tag_value: TagValue, lexer: &mut Lexer<'_, Token>) -> Result<CBOR> {
    let item = parse_item(lexer)?;
    match expect_token(lexer) {
        Ok(Token::ParenthesisClose) => Ok(CBOR::to_tagged_value(tag_value, item)),
        Ok(_) => Err(Error::UnmatchedParentheses(lexer.span())),
        Err(e) => {
            if e == Error::UnexpectedEndOfInput {
                return Err(Error::UnmatchedParentheses(lexer.span()));
            }
            return Err(e);
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
        _ => { Err(Error::UnmatchedParentheses(lexer.span())) }
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
                    return Err(Error::UnknownKnownValueName(name, lexer.span()));
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
                return Err(Error::UnexpectedToken(Box::new(token), lexer.span()));
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
            Err(e) if e == Error::UnexpectedEndOfInput => {
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
                if let Some(Token::Colon) = expect_token(lexer).ok() {
                    let value = match parse_item(lexer) {
                        Err(Error::UnexpectedToken(token, span)) if *token == Token::BraceClose => {
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

#[derive(Debug, Clone, Logos, PartialEq)]
#[rustfmt::skip]
#[logos(error = Error)]
#[logos(skip r"(?:[ \t\r\n\f]|/[^/]*/|#[^\n]*)+")]
pub enum Token {
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
    #[regex(r"h'[0-9a-fA-F]*'", |lex| {
        let hex = lex.slice();
        let raw_hex = hex[2..hex.len() - 1].as_bytes();
        if raw_hex.len() % 2 != 0 {
            return Err(Error::InvalidHexString(lex.span()));
        }
        hex::decode(raw_hex)
            .map_err(|_|
                Error::InvalidHexString(lex.span())
            )
    })]
    ByteStringHex(Result<Vec<u8>>),

    /// Binary string in base64 format.
    #[regex(r"b64'([A-Za-z0-9+/=]{2,})'", |lex| {
        let base64 = lex.slice();
        let s = &base64[4..base64.len() - 1];
        base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|_| Error::InvalidBase64String(lex.span()))
    })]
    ByteStringBase64(Result<Vec<u8>>),

    /// JavaScript-style number.
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex|
        lex.slice().parse::<f64>().unwrap()
    )]
    Number(f64),

    /// JavaScript-style string.
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex|
        lex.slice().to_owned()
    )]
    String(String),

    /// Integer followed immediately by an opening parenthesis.
    #[regex(r#"0\(|[1-9][0-9]*\("#, |lex|
        let span = (lex.span().start)..(lex.span().end - 1);
        let stripped = lex.slice().strip_suffix('(').unwrap();
        stripped.parse::<TagValue>().map_err(|_|
                Error::InvalidTagValue(stripped.to_string(), span)
            )
    )]
    TagValue(Result<TagValue>),

    /// Tag name followed immediately by an opening parenthesis.
    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_-]*\("#, |lex|
        // safe to drop the trailing '('
        lex.slice()[..lex.slice().len()-1].to_string()
    )]
    TagName(String),

    /// Integer (same regex as TagValue) enclosed in single quotes.
    #[regex(r#"'0'|'[1-9][0-9]*'"#, |lex|
        let span = (lex.span().start + 1)..(lex.span().end - 1);
        let slice = lex.slice();
        let stripped = slice[1..slice.len() - 1].to_string();
        stripped.parse::<TagValue>().map_err(|_|
                Error::InvalidKnownValue(stripped, span)
            )
    )]
    KnownValueNumber(Result<u64>),

    /// Single-quoted empty string (i.e., `''`) (Unit) or Identifier (same regex
    /// as for tag names) enclosed in single quotes.
    #[regex(r#"''|'[a-zA-Z_][a-zA-Z0-9_-]*'"#, |lex|
        lex.slice()[1..lex.slice().len()-1].to_string()
    )]
    KnownValueName(String),

    /// The _unit_ known value `40000(0)`.
    #[token("Unit")]
    Unit,

    #[regex(r#"ur:([a-zA-Z0-9][a-zA-Z0-9-]*)/([a-zA-Z]{8,})"#, |lex|
        let s = lex.slice();
        let ur = UR::from_ur_string(s);
        ur.map_err(|e| {
            Error::InvalidUr(e.to_string(), lex.span())
        })
    )]
    UR(Result<UR>),
}
