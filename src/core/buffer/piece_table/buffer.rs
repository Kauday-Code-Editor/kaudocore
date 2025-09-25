/// buffer.rs
/// Defines the Buffer struct which uses a Piece Table for efficient text editing.
use std::ops::Range;
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::core::buffer::piece_table::piece::Piece;

#[derive(Clone, Debug)]
pub struct Buffer {
    pieces: Vec<Arc<Piece>>,
    total_length: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            total_length: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pieces: Vec::with_capacity(capacity),
            total_length: 0,
        }
    }
    
    pub fn from_pieces(pieces: Vec<Piece>) -> Self {
        let total_length = pieces.iter().map(|p| p.len()).sum();
        Self {
            pieces: pieces.into_iter().map(Arc::new).collect(),
            total_length,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.total_length
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total_length == 0
    }

    pub fn pieces(&self) -> &[Arc<Piece>] {
        &self.pieces
    }

    pub fn pieces_count(&self) -> usize {
        self.pieces.len()
    }

    pub fn add_piece(&mut self, piece: Piece) {
        self.total_length += piece.len();
        self.pieces.push(Arc::new(piece));
    }

    pub fn insert_piece(&mut self, index: usize, piece: Piece) -> Result<(), &'static str> {
        if index > self.pieces.len() {
            return Err("Index out of bounds");
        }
        self.total_length += piece.len();
        self.pieces.insert(index, Arc::new(piece));
        Ok(())
    }

    pub fn extend_pieces<I>(&mut self, pieces: I) 
    where 
        I: IntoIterator<Item = Piece>
    {
        for piece in pieces {
            self.total_length += piece.len();
            self.pieces.push(Arc::new(piece));
        }
    }

    pub fn clear(&mut self) {
        self.pieces.clear();
        self.total_length = 0;
    }

    pub fn get_piece(&self, index: usize) -> Option<&Arc<Piece>> {
        self.pieces.get(index)
    }

    pub fn remove_piece(&mut self, index: usize) -> Option<Arc<Piece>> {
        if index >= self.pieces.len() {
            return None;
        }
        let piece = self.pieces.remove(index);
        self.total_length -= piece.len();
        Some(piece)
    }

    pub fn swap_pieces(&mut self, a: usize, b: usize) -> Result<(), &'static str> {
        if a >= self.pieces.len() || b >= self.pieces.len() {
            return Err("Index out of bounds");
        }
        self.pieces.swap(a, b);
        Ok(())
    }

    pub fn truncate_pieces(&mut self, len: usize) {
        if len < self.pieces.len() {
            let removed_length: usize = self.pieces[len..].iter().map(|p| p.len()).sum();
            self.pieces.truncate(len);
            self.total_length -= removed_length;
        }
    }

    pub fn iter_pieces(&self) -> std::slice::Iter<Arc<Piece>> {
        self.pieces.iter()
    }

    pub fn shrink_to_fit(&mut self) {
        self.pieces.shrink_to_fit();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.pieces.reserve(additional);
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer[pieces: {}, length: {}]", self.pieces.len(), self.total_length)
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self.total_length == other.total_length && self.pieces == other.pieces
    }
}

impl Eq for Buffer {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferId(usize);

impl BufferId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn id(&self) -> usize {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    pub fn add(&mut self, value: usize) {
        self.0 = self.0.saturating_add(value);
    }

    pub fn sub(&mut self, value: usize) {
        self.0 = self.0.saturating_sub(value);
    }
}

impl Default for BufferId {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Hash for BufferId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialOrd for BufferId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BufferId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl From<usize> for BufferId {
    fn from(id: usize) -> Self {
        Self::new(id)
    }
}

impl fmt::Display for BufferId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BufferId({})", self.0)
    }
}