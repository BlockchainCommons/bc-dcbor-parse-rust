use base64::Engine as _;
use bc_ur::prelude::*;
use dcbor::Date;
use logos::Logos;

use crate::error::{Error, Result};

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
        let raw_hex = &hex.as_bytes()[2..hex.len() - 1];
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

    /// ISO-8601 date literal (date-only or date-time).
    #[regex(r"\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)?", |lex| {
        let date_str = lex.slice();
        Date::from_string(date_str).map_err(|_| {
            Error::InvalidDateString(date_str.to_string(), lex.span())
        })
    })]
    DateLiteral(Result<Date>),

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
