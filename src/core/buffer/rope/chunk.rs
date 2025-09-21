/// chunk.rs
/// This file defines the `Chunk` struct, which represents a segment of text in a rope data structure.
use std::ops::Range;
use std::str::Chars;
use std::fmt;
use std::cmp::min;
use std::iter::FromIterator;
use std::slice::Iter;
use std::string::String;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    text: String,
    line_endings: Vec<usize>,
}

impl Chunk {
    pub fn new(text: String) -> Self {
        let line_endings = text.match_indices('\n').map(|(idx, _)| idx).collect();
        Chunk { text, line_endings }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn line_endings(&self) -> &[usize] {
        &self.line_endings
    }

    pub fn slice(&self, range: Range<usize>) -> Option<Chunk> {
        if range.start > range.end || range.end > self.len() {
            return None;
        }
        
        // Ensure we slice at valid UTF-8 boundaries
        if !self.text.is_char_boundary(range.start) || !self.text.is_char_boundary(range.end) {
            return None;
        }
        
        Some(Chunk::new(self.text[range].to_string()))
    }

    #[inline]
    pub fn iter(&self) -> Chars {
        self.text.chars()
    }

    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_endings.len() + 1
    }

    pub fn line_at(&self, line: usize) -> Option<&str> {
        if line >= self.line_count() {
            return None;
        }
        let start = if line == 0 { 0 } else { self.line_endings[line - 1] + 1 };
        let end = if line < self.line_endings.len() {
            self.line_endings[line]
        } else {
            self.len()
        };
        Some(&self.text[start..end])
    }

    pub fn concat(&self, other: &Chunk) -> Chunk {
        let mut new_text = String::with_capacity(self.text.len() + other.text.len());
        new_text.push_str(&self.text);
        new_text.push_str(&other.text);
        Chunk::new(new_text)
    }

    pub fn split_at(&self, index: usize) -> Option<(Chunk, Chunk)> {
        if index > self.len() || !self.text.is_char_boundary(index) {
            return None;
        }
        let left = Chunk::new(self.text[0..index].to_string());
        let right = Chunk::new(self.text[index..].to_string());
        Some((left, right))
    }

    pub fn replace_range(&self, range: Range<usize>, replacement: &str) -> Option<Chunk> {
        if range.start > range.end || range.end > self.len() {
            return None;
        }
        
        // Ensure we replace at valid UTF-8 boundaries
        if !self.text.is_char_boundary(range.start) || !self.text.is_char_boundary(range.end) {
            return None;
        }
        
        let mut new_text = String::with_capacity(
            self.len() - (range.end - range.start) + replacement.len()
        );
        new_text.push_str(&self.text[0..range.start]);
        new_text.push_str(replacement);
        new_text.push_str(&self.text[range.end..]);
        Some(Chunk::new(new_text))
    }

    pub fn byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        if byte_idx > self.len() {
            return None;
        }
        Some(self.text[..byte_idx].chars().count())
    }

    pub fn char_to_byte(&self, char_idx: usize) -> Option<usize> {
        let mut byte_idx = 0;
        for (i, ch) in self.text.char_indices() {
            if char_idx == 0 {
                return Some(byte_idx);
            }
            if char_idx == 1 {
                return Some(i);
            }
            byte_idx = i;
            if char_idx <= 1 {
                break;
            }
        }
        if char_idx == self.text.chars().count() {
            Some(self.len())
        } else {
            None
        }
    }

    pub fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn find(&self, pattern: &str) -> Option<usize> {
        self.text.find(pattern)
    }

    pub fn rfind(&self, pattern: &str) -> Option<usize> {
        self.text.rfind(pattern)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.text.starts_with(prefix)
    }

    pub fn ends_with(&self, suffix: &str) -> bool {
        self.text.ends_with(suffix)
    }

    pub fn trim(&self) -> Chunk {
        Chunk::new(self.text.trim().to_string())
    }

    pub fn trim_start(&self) -> Chunk {
        Chunk::new(self.text.trim_start().to_string())
    }

    pub fn trim_end(&self) -> Chunk {
        Chunk::new(self.text.trim_end().to_string())
    }

    pub fn contains(&self, pattern: &str) -> bool {
        self.text.contains(pattern)
    }

    pub fn insert(&self, idx: usize, ch: char) -> Option<Chunk> {
        if idx > self.len() || !self.text.is_char_boundary(idx) {
            return None;
        }
        let mut new_text = String::with_capacity(self.len() + ch.len_utf8());
        new_text.push_str(&self.text[..idx]);
        new_text.push(ch);
        new_text.push_str(&self.text[idx..]);
        Some(Chunk::new(new_text))
    }

    pub fn insert_str(&self, idx: usize, string: &str) -> Option<Chunk> {
        if idx > self.len() || !self.text.is_char_boundary(idx) {
            return None;
        }
        let mut new_text = String::with_capacity(self.len() + string.len());
        new_text.push_str(&self.text[..idx]);
        new_text.push_str(string);
        new_text.push_str(&self.text[idx..]);
        Some(Chunk::new(new_text))
    }

    pub fn remove(&self, idx: usize) -> Option<(Chunk, char)> {
        if idx >= self.len() || !self.text.is_char_boundary(idx) {
            return None;
        }
        
        let mut chars = self.text[idx..].chars();
        let removed_char = chars.next()?;
        let char_end = idx + removed_char.len_utf8();
        
        let mut new_text = String::with_capacity(self.len() - removed_char.len_utf8());
        new_text.push_str(&self.text[..idx]);
        new_text.push_str(&self.text[char_end..]);
        
        Some((Chunk::new(new_text), removed_char))
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for Chunk {
    fn from(s: &str) -> Self {
        Chunk::new(s.to_string())
    }
}

impl From<String> for Chunk {
    fn from(s: String) -> Self {
        Chunk::new(s)
    }
}

impl FromIterator<char> for Chunk {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let text: String = iter.into_iter().collect();
        Chunk::new(text)
    }
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = char;
    type IntoIter = Chars<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.text.chars()
    }
}

impl<'a> IntoIterator for &'a mut Chunk {
    type Item = char;
    type IntoIter = Chars<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.text.chars()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk::new(String::new())
    }
}

impl AsRef<str> for Chunk {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

impl std::borrow::Borrow<str> for Chunk {
    fn borrow(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new("Hello\nworld\n".to_string());
        assert_eq!(chunk.len(), 12);
        assert_eq!(chunk.text(), "Hello\nworld\n");
        assert_eq!(chunk.line_endings(), &[5, 11]);
        assert_eq!(chunk.line_count(), 3);
    }

    #[test]
    fn test_empty_chunk() {
        let chunk = Chunk::default();
        assert!(chunk.is_empty());
        assert_eq!(chunk.len(), 0);
        assert_eq!(chunk.line_count(), 1);
        assert_eq!(chunk.line_endings().len(), 0);
    }

    #[test]
    fn test_single_line() {
        let chunk = Chunk::new("Hello world".to_string());
        assert_eq!(chunk.line_count(), 1);
        assert_eq!(chunk.line_at(0), Some("Hello world"));
        assert_eq!(chunk.line_at(1), None);
    }

    #[test]
    fn test_multiple_lines() {
        let chunk = Chunk::new("line1\nline2\nline3".to_string());
        assert_eq!(chunk.line_count(), 3);
        assert_eq!(chunk.line_at(0), Some("line1"));
        assert_eq!(chunk.line_at(1), Some("line2"));
        assert_eq!(chunk.line_at(2), Some("line3"));
        assert_eq!(chunk.line_at(3), None);
    }

    #[test]
    fn test_slice() {
        let chunk = Chunk::new("Hello world".to_string());
        let slice = chunk.slice(0..5).unwrap();
        assert_eq!(slice.text(), "Hello");
        
        let slice = chunk.slice(6..11).unwrap();
        assert_eq!(slice.text(), "world");
        
        assert!(chunk.slice(20..25).is_none());
        assert!(chunk.slice(5..4).is_none());
    }

    #[test]
    fn test_slice_utf8() {
        let chunk = Chunk::new("Héllo wörld 🌍".to_string());
        
        // Valid slice at character boundaries
        let slice = chunk.slice(0..2).unwrap();
        assert_eq!(slice.text(), "Hé");
        
        // Invalid slice not at character boundary should return None
        let emoji_pos = chunk.text().rfind('🌍').unwrap();
        assert!(chunk.slice(emoji_pos + 1..emoji_pos + 2).is_none());
    }

    #[test]
    fn test_split_at() {
        let chunk = Chunk::new("Hello world".to_string());
        let (left, right) = chunk.split_at(5).unwrap();
        assert_eq!(left.text(), "Hello");
        assert_eq!(right.text(), " world");
        
        let (left, right) = chunk.split_at(0).unwrap();
        assert_eq!(left.text(), "");
        assert_eq!(right.text(), "Hello world");
        
        assert!(chunk.split_at(20).is_none());
    }

    #[test]
    fn test_split_at_utf8() {
        let chunk = Chunk::new("Héllo 🌍".to_string());
        
        // Valid split at character boundary
        let (left, right) = chunk.split_at(6).unwrap();
        assert_eq!(left.text(), "Héllo ");
        assert_eq!(right.text(), "🌍");
        
        // Invalid split not at character boundary
        assert!(chunk.split_at(2).is_none()); // Inside é
    }

    #[test]
    fn test_concat() {
        let chunk1 = Chunk::new("Hello ".to_string());
        let chunk2 = Chunk::new("world!".to_string());
        let result = chunk1.concat(&chunk2);
        assert_eq!(result.text(), "Hello world!");
    }

    #[test]
    fn test_replace_range() {
        let chunk = Chunk::new("Hello world".to_string());
        let result = chunk.replace_range(6..11, "Rust").unwrap();
        assert_eq!(result.text(), "Hello Rust");
        
        let result = chunk.replace_range(0..5, "Hi").unwrap();
        assert_eq!(result.text(), "Hi world");
        
        assert!(chunk.replace_range(20..25, "test").is_none());
        assert!(chunk.replace_range(5..4, "test").is_none());
    }

    #[test]
    fn test_byte_char_conversion() {
        let chunk = Chunk::new("Héllo 🌍".to_string());
        
        assert_eq!(chunk.char_len(), 8);
        assert_eq!(chunk.len(), 11); // bytes
        
        // Test conversions
        assert_eq!(chunk.byte_to_char(0), Some(0));
        assert_eq!(chunk.byte_to_char(1), Some(0)); // Inside é
        assert_eq!(chunk.byte_to_char(2), Some(1));
        
        assert_eq!(chunk.char_to_byte(0), Some(0));
        assert_eq!(chunk.char_to_byte(1), Some(2)); // After é
        assert_eq!(chunk.char_to_byte(8), Some(11)); // End of string
    }

    #[test]
    fn test_find_operations() {
        let chunk = Chunk::new("Hello world, hello rust".to_string());
        
        assert_eq!(chunk.find("world"), Some(6));
        assert_eq!(chunk.find("hello"), Some(13)); // case sensitive
        assert_eq!(chunk.find("xyz"), None);
        
        assert_eq!(chunk.rfind("hello"), Some(13));
        assert_eq!(chunk.rfind("Hello"), Some(0));
        
        assert!(chunk.contains("world"));
        assert!(!chunk.contains("World"));
        
        assert!(chunk.starts_with("Hello"));
        assert!(!chunk.starts_with("hello"));
        
        assert!(chunk.ends_with("rust"));
        assert!(!chunk.ends_with("Rust"));
    }

    #[test]
    fn test_trim_operations() {
        let chunk = Chunk::new("  Hello world  ".to_string());
        
        assert_eq!(chunk.trim().text(), "Hello world");
        assert_eq!(chunk.trim_start().text(), "Hello world  ");
        assert_eq!(chunk.trim_end().text(), "  Hello world");
        
        let no_spaces = Chunk::new("Hello".to_string());
        assert_eq!(no_spaces.trim().text(), "Hello");
    }

    #[test]
    fn test_insert_operations() {
        let chunk = Chunk::new("Hello world".to_string());
        
        let result = chunk.insert(5, ',').unwrap();
        assert_eq!(result.text(), "Hello, world");
        
        let result = chunk.insert_str(5, ", beautiful").unwrap();
        assert_eq!(result.text(), "Hello, beautiful world");
        
        // Test boundary conditions
        let result = chunk.insert(0, 'H').unwrap();
        assert_eq!(result.text(), "HHello world");
        
        let result = chunk.insert(chunk.len(), '!').unwrap();
        assert_eq!(result.text(), "Hello world!");
        
        // Test invalid positions
        assert!(chunk.insert(chunk.len() + 1, 'x').is_none());
    }

    #[test]
    fn test_insert_utf8() {
        let chunk = Chunk::new("Hello world".to_string());
        
        let result = chunk.insert(5, '🌍').unwrap();
        assert_eq!(result.text(), "Hello🌍 world");
        
        let result = chunk.insert_str(5, " 🌍").unwrap();
        assert_eq!(result.text(), "Hello 🌍 world");
    }

    #[test]
    fn test_remove_operations() {
        let chunk = Chunk::new("Hello world".to_string());
        
        let (result, removed) = chunk.remove(5).unwrap();
        assert_eq!(result.text(), "Helloworld");
        assert_eq!(removed, ' ');
        
        let (result, removed) = chunk.remove(0).unwrap();
        assert_eq!(result.text(), "ello world");
        assert_eq!(removed, 'H');
        
        // Test with UTF-8
        let utf8_chunk = Chunk::new("Hé🌍llo".to_string());
        let (result, removed) = utf8_chunk.remove(2).unwrap();
        assert_eq!(result.text(), "Héllo");
        assert_eq!(removed, '🌍');
        
        // Test invalid positions
        assert!(chunk.remove(chunk.len()).is_none());
    }

    #[test]
    fn test_iterator() {
        let chunk = Chunk::new("abc".to_string());
        let chars: Vec<char> = chunk.iter().collect();
        assert_eq!(chars, vec!['a', 'b', 'c']);
        
        let chars: Vec<char> = (&chunk).into_iter().collect();
        assert_eq!(chars, vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_from_implementations() {
        let chunk1 = Chunk::from("Hello");
        assert_eq!(chunk1.text(), "Hello");
        
        let chunk2 = Chunk::from("World".to_string());
        assert_eq!(chunk2.text(), "World");
        
        let chars = vec!['H', 'i'];
        let chunk3: Chunk = chars.into_iter().collect();
        assert_eq!(chunk3.text(), "Hi");
    }

    #[test]
    fn test_display() {
        let chunk = Chunk::new("Hello world".to_string());
        assert_eq!(format!("{}", chunk), "Hello world");
    }

    #[test]
    fn test_as_ref_borrow() {
        let chunk = Chunk::new("Hello".to_string());
        
        let s: &str = chunk.as_ref();
        assert_eq!(s, "Hello");
        
        use std::borrow::Borrow;
        let s: &str = chunk.borrow();
        assert_eq!(s, "Hello");
    }

    // Integration tests
    #[test]
    fn test_rope_operations_simulation() {
        // Simulate rope operations with multiple chunks
        let mut chunks = vec![
            Chunk::new("Hello ".to_string()),
            Chunk::new("beautiful ".to_string()),
            Chunk::new("world!".to_string()),
        ];
        
        // Test concatenation of multiple chunks
        let mut result = chunks[0].clone();
        for chunk in &chunks[1..] {
            result = result.concat(chunk);
        }
        assert_eq!(result.text(), "Hello beautiful world!");
        
        // Test splitting and rejoining
        let (left, right) = result.split_at(6).unwrap();
        let middle_chunk = Chunk::new("amazing ".to_string());
        let final_result = left.concat(&middle_chunk).concat(&right);
        assert_eq!(final_result.text(), "Hello amazing beautiful world!");
    }

    #[test]
    fn test_text_editing_workflow() {
        let mut document = Chunk::new("fn main() {\n    println!(\"Hello\");\n}".to_string());
        
        // Insert at specific position
        let insert_pos = document.find("Hello").unwrap();
        document = document.replace_range(insert_pos..insert_pos+5, "World").unwrap();
        assert!(document.contains("println!(\"World\")"));
        
        // Add new line
        let brace_pos = document.rfind('}').unwrap();
        document = document.insert(brace_pos, '\n').unwrap();
        assert_eq!(document.line_count(), 4);
        
        // Verify final structure
        assert!(document.starts_with("fn main()"));
        assert!(document.ends_with("\n}"));
    }

    #[test]
    fn test_large_text_performance() {
        let large_text = "a".repeat(10000) + "\n" + &"b".repeat(10000);
        let chunk = Chunk::new(large_text);
        
        assert_eq!(chunk.len(), 20001);
        assert_eq!(chunk.line_count(), 2);
        assert_eq!(chunk.line_at(0).unwrap().len(), 10000);
        assert_eq!(chunk.line_at(1).unwrap().len(), 10000);
        
        // Test slicing large chunk
        let slice = chunk.slice(5000..15000).unwrap();
        assert_eq!(slice.len(), 10000);
    }

    #[test]
    fn test_unicode_edge_cases() {
        // Test with various Unicode characters
        let unicode_text = "🌍🌎🌏\n한글\nРусский\n中文\némoji: 👨‍💻";
        let chunk = Chunk::new(unicode_text.to_string());
        
        assert_eq!(chunk.line_count(), 5);
        assert!(chunk.line_at(0).unwrap().contains("🌍"));
        assert!(chunk.line_at(1).unwrap().contains("한글"));
        assert!(chunk.line_at(2).unwrap().contains("Русский"));
        assert!(chunk.line_at(3).unwrap().contains("中文"));
        assert!(chunk.line_at(4).unwrap().contains("👨‍💻"));
        
        // Test operations preserve Unicode correctly
        let (left, right) = chunk.split_at(chunk.text().find('\n').unwrap()).unwrap();
        assert_eq!(left.text(), "🌍🌎🌏");
        assert!(right.text().starts_with('\n'));
    }

    #[test]
    fn test_error_conditions() {
        let chunk = Chunk::new("test".to_string());
        
        // Out of bounds operations
        assert!(chunk.slice(10..20).is_none());
        assert!(chunk.split_at(10).is_none());
        assert!(chunk.replace_range(10..20, "x").is_none());
        assert!(chunk.insert(10, 'x').is_none());
        assert!(chunk.remove(10).is_none());
        
        // Invalid ranges
        assert!(chunk.slice(3..2).is_none());
        assert!(chunk.replace_range(3..2, "x").is_none());
        
        // Conversion edge cases
        assert!(chunk.byte_to_char(100).is_none());
        assert!(chunk.char_to_byte(100).is_none());
    }
}

// -- Made by still-eau (id discord: stilau_)