/// piece.rs
/// Defines the Piece struct used in the Piece Table data structure for text editing.
use std::ops::Range;
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub buffer_id: usize,
    pub range: Range<usize>,
}

impl Piece {
    pub fn new(buffer_id: usize, range: Range<usize>) -> Self {
        Self { buffer_id, range }
    }

    pub fn with_length(buffer_id: usize, start: usize, length: usize) -> Self {
        Self {
            buffer_id,
            range: start..(start + length),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.range.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.range.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.range.end
    }

    pub fn contains(&self, index: usize) -> bool {
        self.range.contains(&index)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.buffer_id == other.buffer_id && 
        self.range.start < other.range.end && 
        other.range.start < self.range.end
    }

    pub fn can_merge(&self, other: &Self) -> bool {
        self.buffer_id == other.buffer_id && 
        (self.range.end == other.range.start || other.range.end == self.range.start)
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        if !self.can_merge(other) {
            return None;
        }
        
        let start = self.range.start.min(other.range.start);
        let end = self.range.end.max(other.range.end);
        
        Some(Self {
            buffer_id: self.buffer_id,
            range: start..end,
        })
    }

    pub fn split_at(&self, index: usize) -> (Self, Self) {
        assert!(index <= self.len(), "Index {} out of bounds for piece of length {}", index, self.len());
        
        let left = Self {
            buffer_id: self.buffer_id,
            range: self.range.start..(self.range.start + index),
        };
        
        let right = Self {
            buffer_id: self.buffer_id,
            range: (self.range.start + index)..self.range.end,
        };
        
        (left, right)
    }

    pub fn split_from(&self, index: usize) -> Self {
        assert!(index <= self.len(), "Index {} out of bounds for piece of length {}", index, self.len());
        Self {
            buffer_id: self.buffer_id,
            range: (self.range.start + index)..self.range.end,
        }
    }

    pub fn split_to(&self, index: usize) -> Self {
        assert!(index <= self.len(), "Index {} out of bounds for piece of length {}", index, self.len());
        Self {
            buffer_id: self.buffer_id,
            range: self.range.start..(self.range.start + index),
        }
    }

    pub fn shift(&self, offset: isize) -> Option<Self> {
        let new_start = self.range.start as isize + offset;
        let new_end = self.range.end as isize + offset;
        
        if new_start < 0 || new_end < 0 {
            return None;
        }
        
        Some(Self {
            buffer_id: self.buffer_id,
            range: (new_start as usize)..(new_end as usize),
        })
    }

    pub fn shift_unchecked(&self, offset: isize) -> Self {
        let new_start = (self.range.start as isize + offset) as usize;
        let new_end = (self.range.end as isize + offset) as usize;
        Self {
            buffer_id: self.buffer_id,
            range: new_start..new_end,
        }
    }

    pub fn truncate(&self, new_length: usize) -> Self {
        let actual_length = new_length.min(self.len());
        Self {
            buffer_id: self.buffer_id,
            range: self.range.start..(self.range.start + actual_length),
        }
    }

    pub fn extend(&self, additional_length: usize) -> Self {
        Self {
            buffer_id: self.buffer_id,
            range: self.range.start..(self.range.end + additional_length),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Piece(buffer_id: {}, range: {:?}, len: {})", 
               self.buffer_id, self.range, self.len())
    }
}

impl Ord for Piece {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.buffer_id.cmp(&other.buffer_id) {
            Ordering::Equal => self.range.start.cmp(&other.range.start),
            other => other,
        }
    }
}

impl PartialOrd for Piece {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            buffer_id: 0,
            range: 0..0,
        }
    }
}

impl From<(usize, Range<usize>)> for Piece {
    fn from((buffer_id, range): (usize, Range<usize>)) -> Self {
        Self { buffer_id, range }
    }
}

impl From<(usize, usize, usize)> for Piece {
    fn from((buffer_id, start, end): (usize, usize, usize)) -> Self {
        Self { 
            buffer_id, 
            range: start..end 
        }
    }
}