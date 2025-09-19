/// validation.rs
/// File containing validation functions for buffer content.
/// This includes functions to validate UTF-8 encoding,
/// check for control characters, and ensure proper line endings.
use std::str;
use std::error::Error;
use std::fmt;
use std::result::Result;
use regex::Regex;
use lazy_static::lazy_static;
use crate::core::buffer::content::encoding::{Encoding, EncodingHandler, EncodingError};
use crate::core::buffer::content::line_ending::LineEnding;

#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidUtf8,
    ControlCharacterFound(char),
    InvalidLineEnding(String),
    EncodingError(EncodingError),
    InvalidEncoding(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidUtf8 => write!(f, "Invalid UTF-8 encoding"),
            ValidationError::ControlCharacterFound(c) => write!(f, "Control character found: {:?}", c),
            ValidationError::InvalidLineEnding(le) => write!(f, "Invalid line ending: {}", le),
            ValidationError::EncodingError(e) => write!(f, "Encoding error: {}", e),
            ValidationError::InvalidEncoding(enc) => write!(f, "Invalid encoding: {}", enc),
        }
    }
}

impl Error for ValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidationError::EncodingError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EncodingError> for ValidationError {
    fn from(error: EncodingError) -> Self {
        ValidationError::EncodingError(error)
    }
}

pub fn validate_utf8(content: &[u8]) -> Result<(), ValidationError> {
    if str::from_utf8(content).is_ok() {
        Ok(())
    } else {
        Err(ValidationError::InvalidUtf8)
    }
}

pub fn validate_control_characters(content: &str) -> Result<(), ValidationError> {
    for c in content.chars() {
        if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
            return Err(ValidationError::ControlCharacterFound(c));
        }
    }
    Ok(())
}

pub fn validate_line_endings(content: &str, expected: LineEnding) -> Result<(), ValidationError> {
    if expected == LineEnding::Unknown {
        return Ok(());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"\r\n|\r|\u{0085}|\u{2028}|\u{2029}|\n").unwrap();
    }

    for mat in RE.find_iter(content) {
        let le = mat.as_str();
        let valid = match expected {
            LineEnding::CRLF => le == "\r\n",
            LineEnding::LF => le == "\n",
            LineEnding::CR => le == "\r",
            LineEnding::NEL => le == "\u{0085}",
            LineEnding::LS => le == "\u{2028}",
            LineEnding::PS => le == "\u{2029}",
            LineEnding::Unknown => true,
        };
        if !valid {
            return Err(ValidationError::InvalidLineEnding(le.to_string()));
        }
    }
    Ok(())
}

pub fn validate_encoding_compatibility(content: &[u8], encoding: &Encoding) -> Result<(), ValidationError> {
    let handler = EncodingHandler::new(encoding.clone());
    handler.decode(content)?;
    Ok(())
}

pub fn validate_content(content: &[u8], encoding: &Encoding, line_ending: LineEnding) -> Result<(), ValidationError> {
    validate_encoding_compatibility(content, encoding)?;
    
    let handler = EncodingHandler::new(encoding.clone());
    let content_str = handler.decode(content)?;
    
    validate_control_characters(&content_str)?;
    validate_line_endings(&content_str, line_ending)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_utf8_valid() {
        let content = "Hello, 世界!".as_bytes();
        assert!(validate_utf8(content).is_ok());
    }

    #[test]
    fn test_validate_utf8_invalid() {
        let content = &[0xFF, 0xFE];
        assert!(validate_utf8(content).is_err());
    }

    #[test]
    fn test_validate_control_characters_valid() {
        let content = "Hello\nWorld\tTest\r";
        assert!(validate_control_characters(content).is_ok());
    }

    #[test]
    fn test_validate_control_characters_invalid() {
        let content = "Hello\x00World";
        assert!(validate_control_characters(content).is_err());
    }

    #[test]
    fn test_validate_line_endings_valid() {
        let content = "line1\r\nline2\r\nline3";
        assert!(validate_line_endings(content, LineEnding::CRLF).is_ok());
    }

    #[test]
    fn test_validate_line_endings_invalid() {
        let content = "line1\nline2\r\nline3";
        assert!(validate_line_endings(content, LineEnding::CRLF).is_err());
    }

    #[test]
    fn test_validate_content_valid() {
        let content = "Hello\nWorld".as_bytes();
        assert!(validate_content(content, &Encoding::UTF8, LineEnding::LF).is_ok());
    }

    #[test]
    fn test_validate_unicode_line_endings() {
        let content = "line1\u{0085}line2";
        assert!(validate_line_endings(content, LineEnding::NEL).is_ok());
        
        let content = "line1\u{2028}line2";
        assert!(validate_line_endings(content, LineEnding::LS).is_ok());
    }
}

// --Made by still-eau (Id discord: stilau_)