/// descriptor.rs
/// Defines the descriptor for pieces in the Piece Table data structure.
use std::ops::Range;
use std::fmt::{self, Formatter};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::core::buffer::piece_table::piece::Piece;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PieceDescriptor {
    pub piece: Arc<Piece>,
    pub global_start: usize,
}

impl PieceDescriptor {
    pub fn new(piece: Arc<Piece>, global_start: usize) -> Self {
        Self { piece, global_start }
    }

    pub fn from_piece(piece: Piece, global_start: usize) -> Self {
        Self { 
            piece: Arc::new(piece),
            global_start 
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.piece.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.piece.is_empty()
    }

    #[inline]
    pub fn global_end(&self) -> usize {
        self.global_start + self.len()
    }

    pub fn global_range(&self) -> Range<usize> {
        self.global_start..self.global_end()
    }

    pub fn contains_global(&self, index: usize) -> bool {
        index >= self.global_start && index < self.global_end()
    }

    pub fn contains_global_range(&self, range: &Range<usize>) -> bool {
        range.start >= self.global_start && range.end <= self.global_end()
    }

    pub fn intersects_global_range(&self, range: &Range<usize>) -> bool {
        !(range.end <= self.global_start || range.start >= self.global_end())
    }

    #[inline]
    pub fn local_range(&self) -> Range<usize> {
        self.piece.range.clone()
    }

    #[inline]
    pub fn local_start(&self) -> usize {
        self.piece.start()
    }

    #[inline]
    pub fn local_end(&self) -> usize {
        self.piece.end()
    }

    #[inline]
    pub fn buffer_id(&self) -> usize {
        self.piece.buffer_id
    }

    pub fn global_to_local(&self, global_index: usize) -> Option<usize> {
        if self.contains_global(global_index) {
            Some(self.local_start() + (global_index - self.global_start))
        } else {
            None
        }
    }

    pub fn local_to_global(&self, local_index: usize) -> Option<usize> {
        if self.piece.contains(local_index) {
            Some(self.global_start + (local_index - self.local_start()))
        } else {
            None
        }
    }

    pub fn can_merge(&self, other: &Self) -> bool {
        self.piece.can_merge(&other.piece) &&
        (self.global_end() == other.global_start || other.global_end() == self.global_start)
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        if !self.can_merge(other) {
            return None;
        }

        let merged_piece = self.piece.merge(&other.piece)?;
        let new_global_start = self.global_start.min(other.global_start);
        
        Some(Self::new(Arc::new(merged_piece), new_global_start))
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.piece.overlaps(&other.piece) &&
        self.global_start < other.global_end() && 
        other.global_start < self.global_end()
    }

    pub fn split_at(&self, global_index: usize) -> Option<(Self, Self)> {
        if !self.contains_global(global_index) || self.is_empty() {
            return None;
        }
        
        let local_index = global_index - self.global_start;
        if local_index == 0 || local_index == self.len() {
            return None;
        }

        let (left_piece, right_piece) = self.piece.split_at(local_index);
        
        Some((
            Self::new(Arc::new(left_piece), self.global_start),
            Self::new(Arc::new(right_piece), global_index)
        ))
    }

    pub fn split_range(&self, global_range: Range<usize>) -> Option<(Option<Self>, Self, Option<Self>)> {
        if global_range.is_empty() || !self.intersects_global_range(&global_range) {
            return None;
        }

        let range_start = global_range.start.max(self.global_start);
        let range_end = global_range.end.min(self.global_end());

        let mut left = None;
        let mut middle = self.clone();
        let mut right = None;

        if range_start > self.global_start {
            let (l, r) = self.split_at(range_start)?;
            left = Some(l);
            middle = r;
        }

        if range_end < middle.global_end() {
            let (l, r) = middle.split_at(range_end)?;
            middle = l;
            right = Some(r);
        }

        Some((left, middle, right))
    }

    pub fn shift_global(&self, offset: isize) -> Option<Self> {
        let new_global_start = self.global_start as isize + offset;
        if new_global_start < 0 {
            return None;
        }

        Some(Self::new(self.piece.clone(), new_global_start as usize))
    }

    pub fn shift_global_unchecked(&self, offset: isize) -> Self {
        let new_global_start = (self.global_start as isize + offset) as usize;
        Self::new(self.piece.clone(), new_global_start)
    }

    pub fn truncate(&self, new_length: usize) -> Self {
        let truncated_piece = self.piece.truncate(new_length);
        Self::new(Arc::new(truncated_piece), self.global_start)
    }

    pub fn extend(&self, additional_length: usize) -> Self {
        let extended_piece = self.piece.extend(additional_length);
        Self::new(Arc::new(extended_piece), self.global_start)
    }

    pub fn with_global_start(&self, new_global_start: usize) -> Self {
        Self::new(self.piece.clone(), new_global_start)
    }

    pub fn is_adjacent_to(&self, other: &Self) -> bool {
        self.global_end() == other.global_start || other.global_end() == self.global_start
    }

    pub fn distance_to(&self, other: &Self) -> usize {
        if self.global_end() <= other.global_start {
            other.global_start - self.global_end()
        } else if other.global_end() <= self.global_start {
            self.global_start - other.global_end()
        } else {
            0
        }
    }
}

impl fmt::Display for PieceDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PieceDescriptor(global: {}-{}, buffer: {}, local: {:?}, len: {})", 
               self.global_start, 
               self.global_end(), 
               self.piece.buffer_id,
               self.piece.range,
               self.len())
    }
}

impl Ord for PieceDescriptor {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.global_start.cmp(&other.global_start) {
            Ordering::Equal => self.piece.cmp(&other.piece),
            other => other,
        }
    }
}

impl PartialOrd for PieceDescriptor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for PieceDescriptor {
    fn default() -> Self {
        Self {
            piece: Arc::new(Piece::default()),
            global_start: 0,
        }
    }
}

impl From<(Piece, usize)> for PieceDescriptor {
    fn from((piece, global_start): (Piece, usize)) -> Self {
        Self::from_piece(piece, global_start)
    }
}

impl From<(Arc<Piece>, usize)> for PieceDescriptor {
    fn from((piece, global_start): (Arc<Piece>, usize)) -> Self {
        Self::new(piece, global_start)
    }
}