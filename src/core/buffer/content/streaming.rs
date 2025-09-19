/// streaming.rs
/// File containing streaming functions for buffer content.
use std::io::{self, Read, Write, BufReader, BufWriter};
use crate::core::buffer::content::encoding::{Encoding, EncodingHandler, EncodingError};
use crate::core::buffer::content::validation::{validate_utf8, ValidationError};
use crate::core::buffer::content::line_ending::LineEnding;
use std::result::Result;
use std::str;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum StreamingError {
    IoError(String),
    EncodingError(EncodingError),
    ValidationError(ValidationError),
    InvalidEncoding(String),
    BufferOverflow(usize),
    UnexpectedEof,
}

impl fmt::Display for StreamingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StreamingError::IoError(e) => write!(f, "I/O error: {}", e),
            StreamingError::EncodingError(e) => write!(f, "Encoding error: {}", e),
            StreamingError::ValidationError(e) => write!(f, "Validation error: {}", e),
            StreamingError::InvalidEncoding(enc) => write!(f, "Invalid encoding: {}", enc),
            StreamingError::BufferOverflow(size) => write!(f, "Buffer overflow: {} bytes", size),
            StreamingError::UnexpectedEof => write!(f, "Unexpected end of file"),
        }
    }
}

impl Error for StreamingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StreamingError::EncodingError(e) => Some(e),
            StreamingError::ValidationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for StreamingError {
    fn from(error: io::Error) -> Self {
        StreamingError::IoError(error.to_string())
    }
}

impl From<EncodingError> for StreamingError {
    fn from(error: EncodingError) -> Self {
        StreamingError::EncodingError(error)
    }
}

impl From<ValidationError> for StreamingError {
    fn from(error: ValidationError) -> Self {
        StreamingError::ValidationError(error)
    }
}

pub struct StreamReader<R: Read> {
    reader: BufReader<R>,
    encoding_handler: EncodingHandler,
    buffer: Vec<u8>,
    chunk_size: usize,
    validate_content: bool,
}

impl<R: Read> StreamReader<R> {
    pub fn new(reader: R, encoding: Encoding) -> Self {
        Self {
            reader: BufReader::new(reader),
            encoding_handler: EncodingHandler::new(encoding),
            buffer: Vec::new(),
            chunk_size: 8192,
            validate_content: true,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_content = validate;
        self
    }

    pub fn read_chunk(&mut self) -> Result<Option<String>, StreamingError> {
        let mut chunk = vec![0u8; self.chunk_size];
        let bytes_read = self.reader.read(&mut chunk)?;
        
        if bytes_read == 0 {
            return Ok(None);
        }

        chunk.truncate(bytes_read);
        
        if self.validate_content {
            validate_utf8(&chunk)?;
        }

        let text = self.encoding_handler.decode(&chunk)?;
        Ok(Some(text))
    }

    pub fn read_line(&mut self) -> Result<Option<String>, StreamingError> {
        let mut line_bytes = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            match self.reader.read(&mut byte)? {
                0 => {
                    if line_bytes.is_empty() {
                        return Ok(None);
                    }
                    break;
                }
                1 => {
                    line_bytes.push(byte[0]);
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }

        if self.validate_content {
            validate_utf8(&line_bytes)?;
        }

        let text = self.encoding_handler.decode(&line_bytes)?;
        Ok(Some(text))
    }

    pub fn read_all(&mut self) -> Result<String, StreamingError> {
        let mut buffer = Vec::new();
        self.reader.read_to_end(&mut buffer)?;

        if self.validate_content {
            validate_utf8(&buffer)?;
        }

        let text = self.encoding_handler.decode(&buffer)?;
        Ok(text)
    }

    pub fn encoding(&self) -> &Encoding {
        self.encoding_handler.encoding()
    }

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding_handler.set_encoding(encoding);
    }
}

pub struct StreamWriter<W: Write> {
    writer: BufWriter<W>,
    encoding_handler: EncodingHandler,
    line_ending: LineEnding,
    auto_flush: bool,
}

impl<W: Write> StreamWriter<W> {
    pub fn new(writer: W, encoding: Encoding) -> Self {
        Self {
            writer: BufWriter::new(writer),
            encoding_handler: EncodingHandler::new(encoding),
            line_ending: LineEnding::default(),
            auto_flush: false,
        }
    }

    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    pub fn with_auto_flush(mut self, auto_flush: bool) -> Self {
        self.auto_flush = auto_flush;
        self
    }

    pub fn with_bom(mut self) -> Self {
        self.encoding_handler = self.encoding_handler.with_bom();
        self
    }

    pub fn write_text(&mut self, text: &str) -> Result<(), StreamingError> {
        let converted_text = if self.line_ending != LineEnding::default() {
            let detected = LineEnding::detect(text).unwrap_or_default();
            detected.convert_text(text, self.line_ending)
        } else {
            text.to_string()
        };

        let bytes = self.encoding_handler.encode(&converted_text)?;
        self.writer.write_all(&bytes)?;
        
        if self.auto_flush {
            self.writer.flush()?;
        }
        
        Ok(())
    }

    pub fn write_line(&mut self, text: &str) -> Result<(), StreamingError> {
        let line_with_ending = format!("{}{}", text, self.line_ending.as_str());
        self.write_text(&line_with_ending)
    }

    pub fn write_lines<I>(&mut self, lines: I) -> Result<(), StreamingError>
    where
        I: IntoIterator<Item = String>,
    {
        for line in lines {
            self.write_line(&line)?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), StreamingError> {
        self.writer.flush()?;
        Ok(())
    }

    pub fn encoding(&self) -> &Encoding {
        self.encoding_handler.encoding()
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding_handler.set_encoding(encoding);
    }

    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        self.line_ending = line_ending;
    }
}

impl<W: Write> Drop for StreamWriter<W> {
    fn drop(&mut self) {
        let _ = self.writer.flush();
    }
}

pub fn stream_convert<R: Read, W: Write>(
    reader: R,
    writer: W,
    from_encoding: Encoding,
    to_encoding: Encoding,
    from_line_ending: LineEnding,
    to_line_ending: LineEnding,
) -> Result<usize, StreamingError> {
    let mut stream_reader = StreamReader::new(reader, from_encoding);
    let mut stream_writer = StreamWriter::new(writer, to_encoding)
        .with_line_ending(to_line_ending)
        .with_auto_flush(true);

    let mut total_bytes = 0;

    while let Some(chunk) = stream_reader.read_chunk()? {
        let converted_chunk = if from_line_ending != to_line_ending {
            from_line_ending.convert_text(&chunk, to_line_ending)
        } else {
            chunk
        };

        stream_writer.write_text(&converted_chunk)?;
        total_bytes += converted_chunk.len();
    }

    stream_writer.flush()?;
    Ok(total_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_stream_reader_utf8() {
        let data = "Hello\nWorld\n".as_bytes();
        let cursor = Cursor::new(data);
        let mut reader = StreamReader::new(cursor, Encoding::UTF8);

        let result = reader.read_all().unwrap();
        assert_eq!(result, "Hello\nWorld\n");
    }

    #[test]
    fn test_stream_writer_utf8() {
        let mut buffer = Vec::new();
        {
            let mut writer = StreamWriter::new(&mut buffer, Encoding::UTF8)
                .with_line_ending(LineEnding::LF);
            writer.write_line("Hello").unwrap();
            writer.write_line("World").unwrap();
        }

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "Hello\nWorld\n");
    }

    #[test]
    fn test_stream_convert() {
        let input = "Hello\r\nWorld\r\n".as_bytes();
        let mut output = Vec::new();

        let bytes_written = stream_convert(
            Cursor::new(input),
            &mut output,
            Encoding::UTF8,
            Encoding::UTF8,
            LineEnding::CRLF,
            LineEnding::LF,
        ).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "Hello\nWorld\n");
        assert!(bytes_written > 0);
    }

    #[test]
    fn test_stream_reader_chunks() {
        let data = "Hello World Test".as_bytes();
        let cursor = Cursor::new(data);
        let mut reader = StreamReader::new(cursor, Encoding::UTF8)
            .with_chunk_size(5);

        let mut chunks = Vec::new();
        while let Some(chunk) = reader.read_chunk().unwrap() {
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());
        let combined: String = chunks.into_iter().collect();
        assert_eq!(combined, "Hello World Test");
    }
}

// --Made by still-eau (Id discord: stilau_)