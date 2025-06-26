use logos::Span;
use thiserror::Error;

use crate::Token;

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
    #[error("Invalid date string '{0}'")]
    InvalidDateString(String, Span),
    #[error("Duplicate map key")]
    DuplicateMapKey(Span),
}

impl Error {
    pub fn is_default(&self) -> bool {
        matches!(self, Error::UnrecognizedToken(_))
    }

    fn format_message(
        message: &dyn ToString,
        source: &str,
        range: &Span,
    ) -> String {
        let message = message.to_string();
        let start = range.start;
        let end = range.end;
        // Walk through the bytes up to `start` to find line number and line
        // start offset
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
        let line = source.lines().nth(line_number - 1).unwrap_or("");
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
            Error::InvalidDateString(_, range) => Self::format_message(self, source, range),
            Error::DuplicateMapKey(range) => Self::format_message(self, source, range),
        }
    }
}

impl Default for Error {
    fn default() -> Self { Error::UnrecognizedToken(Span::default()) }
}

pub type Result<T> = std::result::Result<T, Error>;
