/// node.rs --- Node structure for rope data structure in text buffer

use crate::core::buffer::content::streaming::{StreamReader, StreamWriter};
use crate::core::buffer::content::validation::validate_content;
use crate::core::buffer::content::{encoding::Encoding, line_ending::LineEnding};
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Node {
    pub content: Rc<RefCell<String>>,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub children: Vec<Node>,
    pub length: usize,
}

impl Node {
    pub fn new(content: String, encoding: Encoding, line_ending: LineEnding) -> Self {
        let length = content.len();
        Node {
            content: Rc::new(RefCell::new(content)),
            encoding,
            line_ending,
            children: Vec::new(),
            length,
        }
    }

    pub fn with_capacity(content: String, encoding: Encoding, line_ending: LineEnding, capacity: usize) -> Self {
        let length = content.len();
        Node {
            content: Rc::new(RefCell::new(content)),
            encoding,
            line_ending,
            children: Vec::with_capacity(capacity),
            length,
        }
    }

    pub fn add_child(&mut self, child: Node) {
        self.length += child.total_length();
        self.children.push(child);
    }

    pub fn insert_child(&mut self, index: usize, child: Node) {
        if index <= self.children.len() {
            self.length += child.total_length();
            self.children.insert(index, child);
        }
    }

    pub fn remove_child(&mut self, index: usize) -> Option<Node> {
        if index < self.children.len() {
            let child = self.children.remove(index);
            self.length -= child.total_length();
            Some(child)
        } else {
            None
        }
    }

    pub fn get_content(&self) -> String {
        self.content.borrow().clone()
    }

    pub fn get_content_ref(&self) -> std::cell::Ref<String> {
        self.content.borrow()
    }

    pub fn set_content(&mut self, new_content: String) -> Result<(), String> {
        match validate_content(&new_content, self.encoding) {
            Ok(_) => {
                let old_length = self.length;
                let new_length = new_content.len();
                *self.content.borrow_mut() = new_content;
                self.length = new_length;
                self.update_parent_length(new_length as i64 - old_length as i64);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn append_content(&mut self, additional_content: &str) -> Result<(), String> {
        let combined = format!("{}{}", self.get_content(), additional_content);
        self.set_content(combined)
    }

    pub fn stream_read(&self, range: Range<usize>) -> String {
        let reader = StreamReader::new(self.content.clone());
        reader.read(range)
    }

    pub fn stream_write(&mut self, range: Range<usize>, new_content: String) -> Result<(), String> {
        let mut writer = StreamWriter::new(self.content.clone());
        match writer.write(range, new_content) {
            Ok(_) => {
                self.length = self.content.borrow().len();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn total_length(&self) -> usize {
        self.length + self.children.iter().map(|child| child.total_length()).sum::<usize>()
    }

    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|child| child.depth()).max().unwrap_or(0)
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0 && self.children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn traverse<F>(&self, action: &F) where F: Fn(&Node) {
        action(self);
        for child in &self.children {
            child.traverse(action);
        }
    }

    pub fn traverse_mut<F>(&mut self, action: &F) where F: Fn(&mut Node) {
        action(self);
        for child in &mut self.children {
            child.traverse_mut(action);
        }
    }

    pub fn find_node(&self, index: usize) -> Option<&Node> {
        if index >= self.total_length() {
            return None;
        }
        if index < self.length {
            return Some(self);
        }
        let mut current_index = self.length;
        for child in &self.children {
            let child_length = child.total_length();
            if index < current_index + child_length {
                return child.find_node(index - current_index);
            }
            current_index += child_length;
        }
        None
    }

    pub fn find_node_mut(&mut self, index: usize) -> Option<&mut Node> {
        if index >= self.total_length() {
            return None;
        }
        if index < self.length {
            return Some(self);
        }
        let mut current_index = self.length;
        for child in &mut self.children {
            let child_length = child.total_length();
            if index < current_index + child_length {
                return child.find_node_mut(index - current_index);
            }
            current_index += child_length;
        }
        None
    }

    pub fn split(&mut self, index: usize) -> Option<Node> {
        if index == 0 || index >= self.length {
            return None;
        }
        let content = self.content.borrow().clone();
        let (left, right) = content.split_at(index);
        
        if let Ok(_) = self.set_content(left.to_string()) {
            Some(Node::new(right.to_string(), self.encoding, self.line_ending))
        } else {
            None
        }
    }

    pub fn merge(&mut self, other: Node) -> Result<(), String> {
        if self.encoding != other.encoding || self.line_ending != other.line_ending {
            return Err("Cannot merge nodes with different encoding or line endings".to_string());
        }
        
        let combined = format!("{}{}", self.get_content(), other.get_content());
        self.set_content(combined)?;
        
        for child in other.children {
            self.add_child(child);
        }
        
        Ok(())
    }

    pub fn balance(&mut self) {
        if self.children.len() <= 2 {
            return;
        }
        
        for child in &mut self.children {
            child.balance();
        }
        
        if self.children.len() > 4 {
            let mid = self.children.len() / 2;
            let right_children = self.children.split_off(mid);
            
            let mut right_node = Node::new(String::new(), self.encoding, self.line_ending);
            right_node.children = right_children;
            right_node.update_length();
            
            self.update_length();
            self.children.push(right_node);
        }
    }

    pub fn compact(&mut self) {
        self.children.retain(|child| !child.is_empty());
        
        if self.children.len() == 1 && self.length == 0 {
            let child = self.children.remove(0);
            *self = child;
        }
    }

    fn update_length(&mut self) {
        self.length = self.content.borrow().len();
    }

    fn update_parent_length(&mut self, delta: i64) {
        if delta != 0 {
            self.length = (self.length as i64 + delta) as usize;
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new(String::new(), Encoding::UTF8, LineEnding::LF)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_content() == other.get_content() 
            && self.encoding == other.encoding 
            && self.line_ending == other.line_ending
            && self.children == other.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("Hello, world!".to_string(), Encoding::UTF8, LineEnding::LF);
        assert_eq!(node.get_content(), "Hello, world!");
        assert_eq!(node.length, 13);
        assert!(node.is_leaf());
    }

    #[test]
    fn test_child_operations() {
        let mut parent = Node::new("Parent".to_string(), Encoding::UTF8, LineEnding::LF);
        let child = Node::new("Child".to_string(), Encoding::UTF8, LineEnding::LF);
        
        parent.add_child(child);
        assert_eq!(parent.child_count(), 1);
        assert_eq!(parent.total_length(), 11);
        
        let removed = parent.remove_child(0);
        assert!(removed.is_some());
        assert_eq!(parent.child_count(), 0);
    }

    #[test]
    fn test_split_merge() {
        let mut node = Node::new("Hello, world!".to_string(), Encoding::UTF8, LineEnding::LF);
        let split_node = node.split(7);
        
        assert!(split_node.is_some());
        assert_eq!(node.get_content(), "Hello, ");
        assert_eq!(split_node.unwrap().get_content(), "world!");
    }

    #[test]
    fn test_find_node() {
        let mut parent = Node::new("Parent".to_string(), Encoding::UTF8, LineEnding::LF);
        let child = Node::new("Child".to_string(), Encoding::UTF8, LineEnding::LF);
        
        parent.add_child(child);
        
        let found = parent.find_node(3);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_content(), "Parent");
        
        let found_child = parent.find_node(8);
        assert!(found_child.is_some());
        assert_eq!(found_child.unwrap().get_content(), "Child");
    }

    #[test]
    fn test_balance() {
        let mut node = Node::with_capacity("Root".to_string(), Encoding::UTF8, LineEnding::LF, 10);
        for i in 0..6 {
            let child = Node::new(format!("Child{}", i), Encoding::UTF8, LineEnding::LF);
            node.add_child(child);
        }
        
        node.balance();
        assert!(node.child_count() <= 4);
    }
}

// -- Made by still-eau (id discord: stilau_) --