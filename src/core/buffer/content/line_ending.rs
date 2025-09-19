/// line_ending.rs
/// File for handling line endings in text buffers.
/// This module provides functionality to detect, convert, & manage line endings.
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;
use crate::core::buffer::content::encoding::{Encoding, EncodingHandler, EncodingError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnding {
    LF,
    CRLF,
    CR,
    NEL,
    LS,
    PS,
    Unknown,
}

impl Default for LineEnding {
    fn default() -> Self {
        #[cfg(windows)]
        return LineEnding::CRLF;
        
        #[cfg(not(windows))]
        return LineEnding::LF;
    }
}

impl fmt::Display for LineEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LineEnding::LF => "LF (\\n)",
            LineEnding::CRLF => "CRLF (\\r\\n)",
            LineEnding::CR => "CR (\\r)",
            LineEnding::NEL => "NEL (U+0085)",
            LineEnding::LS => "LS (U+2028)",
            LineEnding::PS => "PS (U+2029)",
            LineEnding::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for LineEnding {
    type Err = LineEndingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LF" | "\\N" | "\n" => Ok(LineEnding::LF),
            "CRLF" | "\\R\\N" | "\r\n" => Ok(LineEnding::CRLF),
            "CR" | "\\R" | "\r" => Ok(LineEnding::CR),
            "NEL" | "U+0085" => Ok(LineEnding::NEL),
            "LS" | "U+2028" => Ok(LineEnding::LS),
            "PS" | "U+2029" => Ok(LineEnding::PS),
            _ => Err(LineEndingParseError::InvalidLineEnding(s.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum LineEndingParseError {
    #[error("Invalid line ending: {0}")]
    InvalidLineEnding(String),
    #[error("Encoding error: {0}")]
    EncodingError(#[from] EncodingError),
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
            LineEnding::CR => "\r",
            LineEnding::NEL => "\u{0085}",
            LineEnding::LS => "\u{2028}",
            LineEnding::PS => "\u{2029}",
            LineEnding::Unknown => "",
        }
    }

    pub fn as_bytes(&self, encoding: &Encoding) -> Vec<u8> {
        let handler = EncodingHandler::new(encoding.clone());
        handler.encode(self.as_str()).unwrap_or_default()
    }

    pub fn len_bytes(&self, encoding: &Encoding) -> usize {
        self.as_bytes(encoding).len()
    }

    pub fn len_chars(&self) -> usize {
        self.as_str().chars().count()
    }

    pub fn from_bytes(bytes: &[u8], encoding: &Encoding) -> Result<Self, LineEndingParseError> {
        let handler = EncodingHandler::new(encoding.clone());
        let text = handler.decode(bytes)?;
        Self::detect(&text)
    }

    pub fn detect(text: &str) -> Result<Self, LineEndingParseError> {
        let mut crlf_count = 0;
        let mut lf_count = 0;
        let mut cr_count = 0;
        let mut nel_count = 0;
        let mut ls_count = 0;
        let mut ps_count = 0;

        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                        crlf_count += 1;
                    } else {
                        cr_count += 1;
                    }
                }
                '\n' => lf_count += 1,
                '\u{0085}' => nel_count += 1,
                '\u{2028}' => ls_count += 1,
                '\u{2029}' => ps_count += 1,
                _ => {}
            }
        }

        let total = crlf_count + lf_count + cr_count + nel_count + ls_count + ps_count;
        if total == 0 {
            return Ok(LineEnding::default());
        }

        let max_count = [
            (crlf_count, LineEnding::CRLF),
            (lf_count, LineEnding::LF),
            (cr_count, LineEnding::CR),
            (nel_count, LineEnding::NEL),
            (ls_count, LineEnding::LS),
            (ps_count, LineEnding::PS),
        ].into_iter().max_by_key(|(count, _)| *count);

        match max_count {
            Some((0, _)) => Ok(LineEnding::default()),
            Some((_, line_ending)) => Ok(line_ending),
            None => Ok(LineEnding::Unknown),
        }
    }

    pub fn convert_text(&self, text: &str, target: LineEnding) -> String {
        if *self == target {
            return text.to_string();
        }

        let normalized = self.normalize_to_lf(text);
        match target {
            LineEnding::LF => normalized,
            LineEnding::CRLF => normalized.replace('\n', "\r\n"),
            LineEnding::CR => normalized.replace('\n', "\r"),
            LineEnding::NEL => normalized.replace('\n', "\u{0085}"),
            LineEnding::LS => normalized.replace('\n', "\u{2028}"),
            LineEnding::PS => normalized.replace('\n', "\u{2029}"),
            LineEnding::Unknown => normalized,
        }
    }

    pub fn normalize_to_lf(&self, text: &str) -> String {
        match self {
            LineEnding::LF => text.to_string(),
            LineEnding::CRLF => text.replace("\r\n", "\n"),
            LineEnding::CR => text.replace('\r', "\n"),
            LineEnding::NEL => text.replace('\u{0085}', "\n"),
            LineEnding::LS => text.replace('\u{2028}', "\n"),
            LineEnding::PS => text.replace('\u{2029}', "\n"),
            LineEnding::Unknown => text.to_string(),
        }
    }

    pub fn count_lines(&self, text: &str) -> usize {
        if text.is_empty() {
            return 1;
        }

        match self {
            LineEnding::CRLF => text.matches("\r\n").count() + 1,
            LineEnding::LF => text.matches('\n').count() + 1,
            LineEnding::CR => text.matches('\r').count() + 1,
            LineEnding::NEL => text.matches('\u{0085}').count() + 1,
            LineEnding::LS => text.matches('\u{2028}').count() + 1,
            LineEnding::PS => text.matches('\u{2029}').count() + 1,
            LineEnding::Unknown => {
                let normalized = text.replace("\r\n", "\n")
                    .replace('\r', "\n")
                    .replace('\u{0085}', "\n")
                    .replace('\u{2028}', "\n")
                    .replace('\u{2029}', "\n");
                normalized.matches('\n').count() + 1
            }
        }
    }

    pub fn split_lines<'a>(&self, text: &'a str) -> Vec<&'a str> {
        match self {
            LineEnding::CRLF => text.split("\r\n").collect(),
            LineEnding::LF => text.split('\n').collect(),
            LineEnding::CR => text.split('\r').collect(),
            LineEnding::NEL => text.split('\u{0085}').collect(),
            LineEnding::LS => text.split('\u{2028}').collect(),
            LineEnding::PS => text.split('\u{2029}').collect(),
            LineEnding::Unknown => {
                UnicodeSegmentation::split_word_bounds(text)
                    .filter(|s| !s.trim().is_empty())
                    .collect()
            }
        }
    }

    pub fn all() -> Vec<LineEnding> {
        vec![
            LineEnding::LF,
            LineEnding::CRLF,
            LineEnding::CR,
            LineEnding::NEL,
            LineEnding::LS,
            LineEnding::PS,
        ]
    }

    pub fn is_unicode(&self) -> bool {
        matches!(self, LineEnding::NEL | LineEnding::LS | LineEnding::PS)
    }

    pub fn platform_default() -> Self {
        #[cfg(windows)]
        return LineEnding::CRLF;
        
        #[cfg(target_os = "macos")]
        return LineEnding::LF;
        
        #[cfg(unix)]
        return LineEnding::LF;
        
        #[cfg(not(any(windows, unix, target_os = "macos")))]
        return LineEnding::LF;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_ending_detection() {
        assert_eq!(LineEnding::detect("hello\nworld").unwrap(), LineEnding::LF);
        assert_eq!(LineEnding::detect("hello\r\nworld").unwrap(), LineEnding::CRLF);
        assert_eq!(LineEnding::detect("hello\rworld").unwrap(), LineEnding::CR);
    }

    #[test]
    fn test_line_ending_conversion() {
        let text = "line1\nline2\nline3";
        let converted = LineEnding::LF.convert_text(text, LineEnding::CRLF);
        assert_eq!(converted, "line1\r\nline2\r\nline3");
    }

    #[test]
    fn test_line_counting() {
        let text = "line1\nline2\nline3";
        assert_eq!(LineEnding::LF.count_lines(text), 3);
        
        let text_crlf = "line1\r\nline2\r\nline3";
        assert_eq!(LineEnding::CRLF.count_lines(text_crlf), 3);
    }

    #[test] 
        let text = "line1\r\nline2\rline3\nline4";
        let normalized = LineEnding::CRLF.normalize_to_lf(text);
        assert!(normalized.contains('\n'));
        assert!(!normalized.contains('\r'));
    }

    #[test]
    fn test_unicode_line_endings() {
        assert!(LineEnding::NEL.is_unicode());
        assert!(LineEnding::LS.is_unicode());
        assert!(LineEnding::PS.is_unicode());
        assert!(!LineEnding::LF.is_unicode());
        assert!(!LineEnding::CRLF.is_unicode());
    }

    #[test]
    fn test_from_str() {
        assert_eq!("LF".parse::<LineEnding>().unwrap(), LineEnding::LF);
        assert_eq!("CRLF".parse::<LineEnding>().unwrap(), LineEnding::CRLF);
        assert!("invalid".parse::<LineEnding>().is_err());
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(LineEnding::LF.count_lines(""), 1);
        assert_eq!(LineEnding::detect("").unwrap(), LineEnding::default());
}