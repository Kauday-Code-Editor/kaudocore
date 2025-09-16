// If you read this code & you dont know what it does, please dont touch it & remember you are a dumbass

// This code is used to detect & handling the encoding for the IDE & the editor

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding {
    UTF8,
    UTF16,
    UTF16LE,
    UTF16BE,
    LATIN1,
    WINDOWS1252,
    ASCII,
}

impl Encoding {
    pub fn name(&self) -> &'static str {
        match self {
            Encoding::UTF8 => "UTF-8",
            Encoding::UTF16 => "UTF-16",
            Encoding::UTF16LE => "UTF-16LE",
            Encoding::UTF16BE => "UTF-16BE",
            Encoding::LATIN1 => "ISO-8859-1",
            Encoding::WINDOWS1252 => "WINDOWS-1252",
            Encoding::ASCII => "ASCII",
        }
    }

    pub fn all() -> Vec<Encoding> {
        vec![
            Encoding::UTF8,
            Encoding::UTF16,
            Encoding::UTF16LE,
            Encoding::UTF16BE,
            Encoding::LATIN1,
            Encoding::WINDOWS1252,
            Encoding::ASCII,
        ]
    }

    pub fn from_name(name: &str) -> Option<Encoding> {
        match name {
            "UTF-8" => Some(Encoding::UTF8),
            "UTF-16" => Some(Encoding::UTF16),
            "UTF-16LE" => Some(Encoding::UTF16LE),
            "UTF-16BE" => Some(Encoding::UTF16BE),
            "ISO-8859-1" => Some(Encoding::LATIN1),
            "WINDOWS-1252" => Some(Encoding::WINDOWS1252),
            "ASCII" => Some(Encoding::ASCII),
            _ => None,
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::UTF8
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone)]
pub enum EncodingError {
    InvalidSequence(String),
    InvalidByte(u8),
    UnsupportedEncoding(String),
    ConversionFailed(String),
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingError::InvalidSequence(sequence) => write!(f, "Invalid sequence: {}", sequence),
            EncodingError::InvalidByte(byte) => write!(f, "Invalid byte: {}", byte),
            EncodingError::UnsupportedEncoding(encoding) => write!(f, "Unsupported encoding: {}", encoding),
            EncodingError::ConversionFailed(encoding) => write!(f, "Conversion failed: {}", encoding),
        }
    }
}

impl Error for EncodingError {}

#[derive(Debug, Clone)]
pub struct EncodingHandler {
    encoding: Encoding,
    with_bom: bool,
    bom_bytes: Vec<u8>,
}

impl EncodingHandler {
    pub fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            with_bom: false,
            bom_bytes: Vec::new(),
        }
    }

    pub fn with_bom(mut self) -> Self {
        self.with_bom = true;
        self.bom_bytes = match self.encoding {
            Encoding::UTF8 => vec![0xEF, 0xBB, 0xBF],
            Encoding::UTF16LE => vec![0xFF, 0xFE],
            Encoding::UTF16BE => vec![0xFE, 0xFF],
            _ => Vec::new(),
        };
        self
    }

    pub fn encoding(&self) -> &Encoding {
        &self.encoding
    }

    pub fn has_bom(&self) -> bool {
        self.with_bom
    }

    pub fn bom_bytes(&self) -> &[u8] {
        &self.bom_bytes
    }

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding = encoding;
        if self.with_bom {
            self.bom_bytes = match encoding {
                Encoding::UTF8 => vec![0xEF, 0xBB, 0xBF],
                Encoding::UTF16LE => vec![0xFF, 0xFE],
                Encoding::UTF16BE => vec![0xFE, 0xFF],
                _ => Vec::new(),
            };
        }
    }

    pub fn set_bom(&mut self, with_bom: bool) {
        self.with_bom = with_bom;
        if with_bom {
            self.bom_bytes = match self.encoding {
                Encoding::UTF8 => vec![0xEF, 0xBB, 0xBF],
                Encoding::UTF16LE => vec![0xFF, 0xFE],
                Encoding::UTF16BE => vec![0xFE, 0xFF],
                _ => Vec::new(),
            };
        } else {
            self.bom_bytes.clear();
        }
    }

    pub fn detect_encoding(bytes: &[u8]) -> Result<(Encoding, bool), EncodingError> {
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return Ok((Encoding::UTF8, true));
        }
        if bytes.starts_with(&[0xFF, 0xFE]) {
            return Ok((Encoding::UTF16LE, true));
        }
        if bytes.starts_with(&[0xFE, 0xFF]) {
            return Ok((Encoding::UTF16BE, true));
        }

        if std::str::from_utf8(bytes).is_ok() {
            return Ok((Encoding::UTF8, false));
        }

        if bytes.iter().all(|&b| b < 128) {
            return Ok((Encoding::ASCII, false));
        }

        Ok((Encoding::LATIN1, false))
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<String, EncodingError> {
        let (data, _) = self.skip_bom(bytes);

        match &self.encoding {
            Encoding::UTF8 => {
                String::from_utf8(data.to_vec())
                    .map_err(|e| EncodingError::InvalidSequence(e.to_string()))
            }
            Encoding::UTF16 | Encoding::UTF16LE => {
                self.decode_utf16le(data)
            }
            Encoding::UTF16BE => {
                self.decode_utf16be(data)
            }
            Encoding::LATIN1 => {
                Ok(data.iter().map(|&b| b as char).collect())
            }
            Encoding::WINDOWS1252 => {
                self.decode_windows1252(data)
            }
            Encoding::ASCII => {
                if data.iter().all(|&b| b < 128) {
                    Ok(String::from_utf8(data.to_vec()).unwrap())
                } else {
                    Err(EncodingError::InvalidSequence("Caractères non-ASCII détectés".to_string()))
                }
            }
        }
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u8>, EncodingError> {
        let mut result = Vec::new();

        if self.with_bom {
            result.extend_from_slice(&self.bom_bytes);
        }

        match &self.encoding {
            Encoding::UTF8 => {
                result.extend_from_slice(text.as_bytes());
            }
            Encoding::UTF16 | Encoding::UTF16LE => {
                for c in text.encode_utf16() {
                    result.extend_from_slice(&c.to_le_bytes());
                }
            }
            Encoding::UTF16BE => {
                for c in text.encode_utf16() {
                    result.extend_from_slice(&c.to_be_bytes());
                }
            }
            Encoding::LATIN1 => {
                for ch in text.chars() {
                    let code = ch as u32;
                    if code <= 255 {
                        result.push(code as u8);
                    } else {
                        return Err(EncodingError::ConversionFailed(
                            format!("Caractère '{}' non représentable en Latin1", ch)
                        ));
                    }
                }
            }
            Encoding::WINDOWS1252 => {
                result.extend(self.encode_windows1252(text)?);
            }
            Encoding::ASCII => {
                for ch in text.chars() {
                    if ch.is_ascii() {
                        result.push(ch as u8);
                    } else {
                        return Err(EncodingError::ConversionFailed(
                            format!("Caractère '{}' non représentable en ASCII", ch)
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    fn skip_bom(&self, bytes: &[u8]) -> (Cow<[u8]>, bool) {
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            (Cow::Borrowed(&bytes[3..]), true)
        } else if bytes.starts_with(&[0xFF, 0xFE]) {
            (Cow::Borrowed(&bytes[2..]), true)
        } else if bytes.starts_with(&[0xFE, 0xFF]) {
            (Cow::Borrowed(&bytes[2..]), true)
        } else {
            (Cow::Borrowed(bytes), false)
        }
    }

    fn decode_utf16le(&self, bytes: &[u8]) -> Result<String, EncodingError> {
        if bytes.len() % 2 != 0 {
            return Err(EncodingError::InvalidSequence("Longueur impaire pour UTF-16".to_string()));
        }

        let mut utf16_chars = Vec::new();
        for chunk in bytes.chunks_exact(2) {
            let val = u16::from_le_bytes([chunk[0], chunk[1]]);
            utf16_chars.push(val);
        }

        String::from_utf16(&utf16_chars)
            .map_err(|e| EncodingError::InvalidSequence(e.to_string()))
    }

    fn decode_utf16be(&self, bytes: &[u8]) -> Result<String, EncodingError> {
        if bytes.len() % 2 != 0 {
            return Err(EncodingError::InvalidSequence("Longueur impaire pour UTF-16".to_string()));
        }

        let mut utf16_chars = Vec::new();
        for chunk in bytes.chunks_exact(2) {
            let val = u16::from_be_bytes([chunk[0], chunk[1]]);
            utf16_chars.push(val);
        }

        String::from_utf16(&utf16_chars)
            .map_err(|e| EncodingError::InvalidSequence(e.to_string()))
    }

    fn decode_windows1252(&self, bytes: &[u8]) -> Result<String, EncodingError> {
        let mut result = String::new();
        
        for &byte in bytes {
            let ch = match byte {
                0x80 => '€',
                0x82 => '‚',
                0x83 => 'ƒ',
                0x84 => '„',
                0x85 => '…',
                0x86 => '†',
                0x87 => '‡',
                0x88 => 'ˆ',
                0x89 => '‰',
                0x8A => 'Š',
                0x8B => '‹',
                0x8C => 'Œ',
                0x8E => 'Ž',
                0x91 => ''',
                0x92 => ''',
                0x93 => '"',
                0x94 => '"',
                0x95 => '•',
                0x96 => '–',
                0x97 => '—',
                0x98 => '˜',
                0x99 => '™',
                0x9A => 'š',
                0x9B => '›',
                0x9C => 'œ',
                0x9E => 'ž',
                0x9F => 'Ÿ',
                _ => byte as char,
            };
            result.push(ch);
        }
        
        Ok(result)
    }

    fn encode_windows1252(&self, text: &str) -> Result<Vec<u8>, EncodingError> {
        let mut result = Vec::new();
        
        for ch in text.chars() {
            let byte = match ch {
                '€' => 0x80,
                '‚' => 0x82,
                'ƒ' => 0x83,
                '„' => 0x84,
                '…' => 0x85,
                '†' => 0x86,
                '‡' => 0x87,
                'ˆ' => 0x88,
                '‰' => 0x89,
                'Š' => 0x8A,
                '‹' => 0x8B,
                'Œ' => 0x8C,
                'Ž' => 0x8E,
                ''' => 0x91,
                ''' => 0x92,
                '"' => 0x93,
                '"' => 0x94,
                '•' => 0x95,
                '–' => 0x96,
                '—' => 0x97,
                '˜' => 0x98,
                '™' => 0x99,
                'š' => 0x9A,
                '›' => 0x9B,
                'œ' => 0x9C,
                'ž' => 0x9E,
                'Ÿ' => 0x9F,
                c if (c as u32) <= 255 => c as u8,
                c => return Err(EncodingError::ConversionFailed(
                    format!("Caractère '{}' non représentable en Windows-1252", c)
                )),
            };
            result.push(byte);
        }
        
        Ok(result)
    }
}

impl Default for EncodingHandler {
    fn default() -> Self {
        Self::new(Encoding::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_encoding() {
        let handler = EncodingHandler::new(Encoding::UTF8);
        let text = "Hello, 世界!";
        
        let encoded = handler.encode(text).unwrap();
        let decoded = handler.decode(&encoded).unwrap();
        
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_bom_detection() {
        let utf8_with_bom = vec![0xEF, 0xBB, 0xBF, b'H', b'i'];
        let (encoding, has_bom) = EncodingHandler::detect_encoding(&utf8_with_bom).unwrap();
        
        assert_eq!(encoding, Encoding::UTF8);
        assert!(has_bom);
    }

    #[test]
    fn test_ascii_encoding() {
        let handler = EncodingHandler::new(Encoding::ASCII);
        let text = "Hello World!";
        
        let encoded = handler.encode(text).unwrap();
        let decoded = handler.decode(&encoded).unwrap();
        
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_invalid_ascii() {
        let handler = EncodingHandler::new(Encoding::ASCII);
        let text = "Héllo!";
        
        let result = handler.encode(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_latin1_encoding() {
        let handler = EncodingHandler::new(Encoding::LATIN1);
        let text = "Café";
        
        let encoded = handler.encode(text).unwrap();
        let decoded = handler.decode(&encoded).unwrap();
        
        assert_eq!(text, decoded);
    }
}