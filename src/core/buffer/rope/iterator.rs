// iterator.rs
// This file is for iterators over ropes.
use crate::core::buffer::rope::node::{Node, NodeRef};
use crate::core::buffer::chunk::Chunk;
use std::cell::RefCell;
use std::cmp::{min, max};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::iter::{Iterator, DoubleEndedIterator, ExactSizeIterator, FusedIterator};
use std::ops::Range;
use std::rc::Rc;

/// An iterator over a rope structure.
#[derive(Clone)]
pub struct RopeIterator {
    stack: Vec<(NodeRef, usize)>,
    reverse_stack: Vec<(NodeRef, usize)>,
    current_chunk: Option<Chunk>,
    current_index: usize,
    total_length: usize,
    traversed_length: usize,
    range: Option<Range<usize>>,
    is_forward: bool,
    exhausted: bool,
}

/// Direction for rope traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection {
    Forward,
    Backward,
}

/// Iterator over rope bytes
pub struct RopeByteIterator {
    rope_iter: RopeIterator,
}

/// Iterator over rope lines
pub struct RopeLineIterator {
    rope_iter: RopeIterator,
    current_line: String,
}

/// Iterator over rope chunks
pub struct RopeChunkIterator {
    stack: Vec<(NodeRef, usize)>,
    range: Option<Range<usize>>,
    traversed_length: usize,
}

impl Debug for RopeIterator {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("RopeIterator")
            .field("stack", &self.stack)
            .field("current_chunk", &self.current_chunk)
            .field("current_index", &self.current_index)
            .field("total_length", &self.total_length)
            .field("traversed_length", &self.traversed_length)
            .field("range", &self.range)
            .field("is_forward", &self.is_forward)
            .field("exhausted", &self.exhausted)
            .finish()
    }
}

impl RopeIterator {
    pub fn new(root: NodeRef, range: Option<Range<usize>>) -> Self {
        let total_length = root.borrow().len();
        let mut iter = Self {
            stack: vec![(root.clone(), 0)],
            reverse_stack: vec![(root, 0)],
            current_chunk: None,
            current_index: 0,
            total_length,
            traversed_length: 0,
            range: range.clone(),
            is_forward: true,
            exhausted: false,
        };
        
        if let Some(ref range) = range {
            iter.seek_to_position(range.start);
        } else {
            iter.advance_to_next_chunk();
        }
        
        iter
    }

    pub fn with_direction(root: NodeRef, range: Option<Range<usize>>, direction: TraversalDirection) -> Self {
        let mut iter = Self::new(root, range);
        match direction {
            TraversalDirection::Forward => iter.set_forward(),
            TraversalDirection::Backward => iter.set_backward(),
        }
        iter
    }

    pub fn set_forward(&mut self) {
        self.is_forward = true;
    }

    pub fn set_backward(&mut self) {
        self.is_forward = false;
    }

    pub fn direction(&self) -> TraversalDirection {
        if self.is_forward {
            TraversalDirection::Forward
        } else {
            TraversalDirection::Backward
        }
    }

    pub fn remaining_length(&self) -> usize {
        if let Some(ref range) = self.range {
            range.end.saturating_sub(self.traversed_length)
        } else {
            self.total_length.saturating_sub(self.traversed_length)
        }
    }

    pub fn position(&self) -> usize {
        self.traversed_length
    }

    pub fn seek_to_position(&mut self, pos: usize) -> bool {
        if pos > self.total_length {
            return false;
        }

        self.reset();
        self.traversed_length = 0;

        while self.traversed_length < pos && !self.exhausted {
            if let Some(chunk) = &self.current_chunk {
                let chunk_len = chunk.len();
                let remaining_to_pos = pos - self.traversed_length;
                
                if remaining_to_pos < chunk_len - self.current_index {
                    self.current_index += remaining_to_pos;
                    self.traversed_length += remaining_to_pos;
                    return true;
                } else {
                    let advance_amount = chunk_len - self.current_index;
                    self.traversed_length += advance_amount;
                    self.advance_to_next_chunk();
                }
            } else {
                break;
            }
        }

        self.traversed_length == pos
    }

    pub fn peek(&self) -> Option<char> {
        self.current_char()
    }

    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        let mut temp_iter = self.clone();
        for _ in 0..n {
            temp_iter.next()?;
        }
        temp_iter.current_char()
    }

    pub fn collect_string(&mut self, max_chars: Option<usize>) -> String {
        let mut result = String::new();
        let limit = max_chars.unwrap_or(usize::MAX);
        
        for (i, ch) in self.enumerate() {
            if i >= limit {
                break;
            }
            result.push(ch);
        }
        
        result
    }

    pub fn skip_while<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut skipped = 0;
        while let Some(ch) = self.current_char() {
            if !predicate(ch) {
                break;
            }
            self.advance_char();
            skipped += 1;
        }
        skipped
    }

    pub fn take_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if !predicate(ch) {
                break;
            }
            result.push(ch);
            self.advance_char();
        }
        result
    }

    pub fn bytes(self) -> RopeByteIterator {
        RopeByteIterator { rope_iter: self }
    }

    pub fn lines(self) -> RopeLineIterator {
        RopeLineIterator {
            rope_iter: self,
            current_line: String::new(),
        }
    }

    pub fn chunks(root: NodeRef, range: Option<Range<usize>>) -> RopeChunkIterator {
        RopeChunkIterator {
            stack: vec![(root, 0)],
            range,
            traversed_length: 0,
        }
    }

    fn reset(&mut self) {
        self.stack.clear();
        self.current_chunk = None;
        self.current_index = 0;
        self.traversed_length = 0;
        self.exhausted = false;
    }

    fn advance_to_next_chunk(&mut self) {
        while let Some((node_ref, child_index)) = self.stack.pop() {
            let node = node_ref.borrow();
            
            if node.is_leaf() {
                self.current_chunk = Some(node.get_chunk().clone());
                self.current_index = 0;
                return;
            }
            
            if child_index < node.children.len() {
                self.stack.push((node_ref.clone(), child_index + 1));
                self.stack.push((node.children[child_index].clone(), 0));
            }
        }
        
        self.current_chunk = None;
        self.current_index = 0;
        self.exhausted = true;
    }

    fn advance_to_prev_chunk(&mut self) {
        while let Some((node_ref, child_index)) = self.reverse_stack.pop() {
            let node = node_ref.borrow();
            
            if node.is_leaf() {
                self.current_chunk = Some(node.get_chunk().clone());
                if let Some(chunk) = &self.current_chunk {
                    self.current_index = chunk.len().saturating_sub(1);
                }
                return;
            }
            
            if child_index > 0 {
                self.reverse_stack.push((node_ref.clone(), child_index - 1));
                if let Some(child) = node.children.get(child_index - 1) {
                    let child_node = child.borrow();
                    let last_index = if child_node.is_leaf() { 0 } else { child_node.children.len() };
                    self.reverse_stack.push((child.clone(), last_index));
                }
            }
        }
        
        self.current_chunk = None;
        self.current_index = 0;
        self.exhausted = true;
    }

    fn within_range(&self) -> bool {
        match &self.range {
            Some(range) => self.traversed_length >= range.start && self.traversed_length < range.end,
            None => true,
        }
    }

    fn advance_char(&mut self) {
        if self.exhausted {
            return;
        }

        if let Some(chunk) = &self.current_chunk {
            if self.is_forward {
                self.current_index += 1;
                self.traversed_length += 1;
                
                if self.current_index >= chunk.len() {
                    self.advance_to_next_chunk();
                }
            } else {
                if self.current_index == 0 {
                    self.advance_to_prev_chunk();
                } else {
                    self.current_index -= 1;
                    self.traversed_length = self.traversed_length.saturating_sub(1);
                }
            }
        }

        if let Some(ref range) = self.range {
            if (self.is_forward && self.traversed_length >= range.end) ||
               (!self.is_forward && self.traversed_length < range.start) {
                self.exhausted = true;
            }
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.exhausted || !self.within_range() {
            return None;
        }
        
        self.current_chunk
            .as_ref()?
            .char_at(self.current_index)
    }
}

impl Iterator for RopeIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.current_char()?;
        self.advance_char();
        Some(ch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining_length();
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for RopeIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        
        self.set_backward();
        let ch = self.current_char()?;
        self.advance_char();
        Some(ch)
    }
}

impl ExactSizeIterator for RopeIterator {
    fn len(&self) -> usize {
        self.remaining_length()
    }
}

impl FusedIterator for RopeIterator {}

impl Iterator for RopeByteIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.rope_iter.next().map(|ch| {
            let mut buf = [0; 4];
            let bytes = ch.encode_utf8(&mut buf).as_bytes();
            bytes[0]
        })
    }
}

impl Iterator for RopeLineIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rope_iter.exhausted {
            return None;
        }

        self.current_line.clear();
        
        while let Some(ch) = self.rope_iter.next() {
            if ch == '\n' {
                break;
            }
            self.current_line.push(ch);
        }

        if self.current_line.is_empty() && self.rope_iter.exhausted {
            None
        } else {
            Some(self.current_line.clone())
        }
    }
}

impl Iterator for RopeChunkIterator {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node_ref, child_index)) = self.stack.pop() {
            let node = node_ref.borrow();
            
            if node.is_leaf() {
                let chunk = node.get_chunk().clone();
                if let Some(ref range) = self.range {
                    if self.traversed_length >= range.end {
                        return None;
                    }
                }
                self.traversed_length += chunk.len();
                return Some(chunk);
            }
            
            if child_index < node.children.len() {
                self.stack.push((node_ref.clone(), child_index + 1));
                self.stack.push((node.children[child_index].clone(), 0));
            }
        }
        
        None
    }
}