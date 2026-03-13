//! Replace step implementation
//!
//! The core replacement algorithm that handles complex document changes.

use crate::step::{Step, StepError, StepResult, Mappable, Invertible};
use crate::map::StepMap;
use model::Node;
use model::node::Mark;
use serde::{Serialize, Deserialize};

/// A slice of document content
///
/// This represents content that can be inserted or used as replacement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slice {
    /// The content of the slice
    pub content: Vec<Node>,
    /// The open depth at the start
    pub open_start: usize,
    /// The open depth at the end
    pub open_end: usize,
}

impl Slice {
    /// Create an empty slice
    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
            open_start: 0,
            open_end: 0,
        }
    }

    /// Create a slice from text
    pub fn from_text(text: &str) -> Self {
        Self {
            content: vec![Node::new_text(text)],
            open_start: 0,
            open_end: 0,
        }
    }

    /// Create a slice from nodes
    pub fn from_nodes(nodes: Vec<Node>) -> Self {
        Self {
            content: nodes,
            open_start: 0,
            open_end: 0,
        }
    }

    /// Create a slice with open depths
    pub fn with_open(nodes: Vec<Node>, open_start: usize, open_end: usize) -> Self {
        Self {
            content: nodes,
            open_start,
            open_end,
        }
    }

    /// Get the size of this slice
    pub fn size(&self) -> usize {
        self.content.iter()
            .map(|n| n.content_size().size)
            .sum()
    }

    /// Get the content size
    pub fn content_size(&self) -> usize {
        self.content.iter()
            .map(|n| n.content_size().content_size)
            .sum()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.content.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get the first node
    pub fn first(&self) -> Option<&Node> {
        self.content.first()
    }

    /// Get the last node
    pub fn last(&self) -> Option<&Node> {
        self.content.last()
    }
}

/// Replace step - handles insertion, deletion, and replacement
///
/// This is the most commonly used step type that supports:
/// - Insertion: from == to
/// - Deletion: slice is empty
/// - Replacement: from < to with non-empty slice
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceStep {
    /// Start position of the replacement
    pub from: usize,
    /// End position of the replacement
    pub to: usize,
    /// The slice to insert
    pub slice: Slice,
    /// Structure-only flag (don't replace text)
    pub structure_only: bool,
}

/// Split the current paragraph at a cursor position.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitBlockStep {
    /// Cursor position in flat text coordinates.
    pub pos: usize,
}

impl SplitBlockStep {
    /// Create a new split block step.
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }
}

/// Join the current paragraph with its previous sibling.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinBackwardStep {
    /// Cursor position at the start of the paragraph to join.
    pub pos: usize,
}

impl JoinBackwardStep {
    /// Create a new join backward step.
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }
}

impl ReplaceStep {
    /// Create a new replace step
    pub fn new(from: usize, to: usize, slice: Slice) -> Self {
        Self {
            from,
            to,
            slice,
            structure_only: false,
        }
    }

    /// Create an insert step
    pub fn insert(pos: usize, slice: Slice) -> Self {
        Self::new(pos, pos, slice)
    }

    /// Create a delete step
    pub fn delete(from: usize, to: usize) -> Self {
        Self::new(from, to, Slice::empty())
    }

    /// Create a replace step
    pub fn replace(from: usize, to: usize, slice: Slice) -> Self {
        Self::new(from, to, slice)
    }

    /// Set structure-only mode
    pub fn with_structure_only(mut self, structure_only: bool) -> Self {
        self.structure_only = structure_only;
        self
    }

}

impl Step for ReplaceStep {
    fn apply(&self, doc: &Node) -> Result<StepResult, StepError> {
        // Validate positions
        let doc_len = doc.text_content().len();
        if self.from > self.to || self.to > doc_len {
            return Err(StepError::PositionOutOfRange(self.from));
        }

        let doc_text = doc.text_content();
        let insert_text: String = self.slice.content.iter().map(Node::text_content).collect();
        let mut next_text = String::with_capacity(doc_text.len() - (self.to - self.from) + insert_text.len());
        next_text.push_str(&doc_text[..self.from]);
        next_text.push_str(&insert_text);
        next_text.push_str(&doc_text[self.to..]);

        let new_doc = doc.with_text_content(next_text);
        let slice = if self.slice.is_empty() { None } else { Some(self.slice.clone()) };

        Ok(StepResult::new(new_doc, self.from, self.to, slice))
    }
}

impl Mappable for ReplaceStep {
    fn map(&self, map: &StepMap) -> Self {
        let from_result = map.map(self.from);
        let to_result = map.map(self.to);

        let new_from = from_result.pos;
        let new_to = if to_result.deleted || from_result.replaced {
            new_from
        } else {
            to_result.pos
        };

        Self::new(new_from, new_to, self.slice.clone()).with_structure_only(self.structure_only)
    }
}

impl Invertible for ReplaceStep {
    fn invert(&self, doc: &Node) -> Self {
        let deleted_text = &doc.text_content()[self.from..self.to];
        let deleted_slice = if deleted_text.is_empty() {
            Slice::empty()
        } else {
            Slice::from_text(deleted_text)
        };

        Self::new(self.from, self.from + self.slice.size(), deleted_slice)
    }
}

impl Step for SplitBlockStep {
    fn apply(&self, doc: &Node) -> Result<StepResult, StepError> {
        let mut paragraphs = doc.paragraph_texts();
        let (index, offset) = doc
            .paragraph_at(self.pos)
            .ok_or(StepError::PositionOutOfRange(self.pos))?;
        let current = paragraphs
            .get(index)
            .cloned()
            .ok_or(StepError::PositionOutOfRange(self.pos))?;

        let left = current[..offset].to_string();
        let right = current[offset..].to_string();

        paragraphs.splice(index..=index, [left, right]);

        Ok(StepResult::new(
            Node::from_paragraph_texts(paragraphs),
            self.pos,
            self.pos,
            None,
        ))
    }
}

impl Step for JoinBackwardStep {
    fn apply(&self, doc: &Node) -> Result<StepResult, StepError> {
        let mut paragraphs = doc.paragraph_texts();
        let mut start = 0;
        let mut index = None;

        for (i, paragraph) in paragraphs.iter().enumerate() {
            if i > 0 && self.pos == start {
                index = Some(i);
                break;
            }
            start += paragraph.len();
        }

        let index = index.ok_or_else(|| {
            StepError::Failed("join_backward requires a cursor at the start of a paragraph".to_string())
        })?;

        if index == 0 {
            return Err(StepError::Failed("join_backward requires a previous paragraph".to_string()));
        }

        let current = paragraphs.remove(index);
        let previous = &mut paragraphs[index - 1];
        previous.push_str(&current);

        Ok(StepResult::new(
            Node::from_paragraph_texts(paragraphs),
            self.pos,
            self.pos,
            None,
        ))
    }
}

impl Step for SetMarksStep {
    fn apply(&self, doc: &Node) -> Result<StepResult, StepError> {
        let doc_len = doc.text_content().len();
        if self.from > self.to || self.to > doc_len {
            return Err(StepError::PositionOutOfRange(self.to));
        }

        let new_doc = doc.with_mark_range(self.from, self.to, &self.add, &self.remove);
        Ok(StepResult::new(new_doc, self.from, self.to, None))
    }
}

/// Lift step - moves content out of a parent
#[derive(Debug, Clone, PartialEq)]
pub struct LiftStep {
    /// Start of range
    pub from: usize,
    /// End of range
    pub to: usize,
    /// Target depth
    pub depth: usize,
}

impl LiftStep {
    /// Create a new lift step
    pub fn new(from: usize, to: usize, depth: usize) -> Self {
        Self { from, to, depth }
    }
}

impl Step for LiftStep {
    fn apply(&self, _doc: &Node) -> Result<StepResult, StepError> {
        // Simplified implementation
        Err(StepError::Failed("LiftStep not fully implemented".to_string()))
    }
}

/// Wrap step - wraps content in a container
#[derive(Debug, Clone, PartialEq)]
pub struct WrapStep {
    /// Start of range
    pub from: usize,
    /// End of range
    pub to: usize,
    /// Wrapper type
    pub wrapper_type: String,
    /// Wrapper attributes
    pub attrs: Option<serde_json::Value>,
}

impl WrapStep {
    /// Create a new wrap step
    pub fn new(from: usize, to: usize, wrapper_type: impl Into<String>, attrs: Option<serde_json::Value>) -> Self {
        Self {
            from,
            to,
            wrapper_type: wrapper_type.into(),
            attrs,
        }
    }
}

impl Step for WrapStep {
    fn apply(&self, _doc: &Node) -> Result<StepResult, StepError> {
        // Simplified implementation
        Err(StepError::Failed("WrapStep not fully implemented".to_string()))
    }
}

/// Set marks step - adds or removes marks from a range
#[derive(Debug, Clone, PartialEq)]
pub struct SetMarksStep {
    /// Start of range
    pub from: usize,
    /// End of range
    pub to: usize,
    /// Marks to add
    pub add: Vec<Mark>,
    /// Marks to remove
    pub remove: Vec<String>,
}

impl SetMarksStep {
    /// Create a new set marks step
    pub fn new(from: usize, to: usize, add: Vec<Mark>, remove: Vec<String>) -> Self {
        Self { from, to, add, remove }
    }

    /// Create an add mark step
    pub fn add_mark(from: usize, to: usize, mark: Mark) -> Self {
        Self::new(from, to, vec![mark], Vec::new())
    }

    /// Create a remove mark step
    pub fn remove_mark(from: usize, to: usize, mark_type: &str) -> Self {
        Self::new(from, to, Vec::new(), vec![mark_type.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_basics() {
        let slice = Slice::from_text("Hello");
        assert_eq!(slice.size(), 5);
        assert_eq!(slice.content_size(), 5);
        assert_eq!(slice.node_count(), 1);
        assert!(!slice.is_empty());
    }

    #[test]
    fn test_slice_empty() {
        let slice = Slice::empty();
        assert!(slice.is_empty());
        assert_eq!(slice.size(), 0);
    }

    #[test]
    fn test_slice_from_nodes() {
        let nodes = vec![
            Node::new_text("Hello"),
            Node::new_text(" "),
            Node::new_text("World"),
        ];
        let slice = Slice::from_nodes(nodes);
        assert_eq!(slice.node_count(), 3);
    }

    #[test]
    fn test_replace_step_insert() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello")]);
        let step = ReplaceStep::insert(5, Slice::from_text(" World"));

        let result = step.apply(&doc).expect("insert should succeed");
        assert_eq!(result.doc.text_content(), "Hello World");
    }

    #[test]
    fn test_replace_step_delete() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello World")]);
        let step = ReplaceStep::delete(5, 11);

        let result = step.apply(&doc).expect("delete should succeed");
        assert_eq!(result.doc.text_content(), "Hello");
    }

    #[test]
    fn test_replace_step_replace() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello World")]);
        let step = ReplaceStep::replace(0, 5, Slice::from_text("Hi"));

        let result = step.apply(&doc).expect("replace should succeed");
        assert_eq!(result.doc.text_content(), "Hi World");
    }

    #[test]
    fn test_replace_step_map() {
        let step = ReplaceStep::delete(10, 15);
        let map = StepMap::new(5, 5, 3); // Insert 3 at position 5

        let mapped = step.map(&map);
        // Position 10 is after the insertion point (5), so shifts by 3
        assert_eq!(mapped.from, 13);
        assert_eq!(mapped.to, 18);
    }

    #[test]
    fn test_split_block_step() {
        let doc = Node::from_paragraph_texts(vec!["HelloWorld".to_string()]);
        let step = SplitBlockStep::new(5);

        let result = step.apply(&doc).expect("split should succeed");
        assert_eq!(result.doc.paragraph_texts(), vec!["Hello".to_string(), "World".to_string()]);
    }

    #[test]
    fn test_join_backward_step() {
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string(), "World".to_string()]);
        let step = JoinBackwardStep::new(5);

        let result = step.apply(&doc).expect("join should succeed");
        assert_eq!(result.doc.paragraph_texts(), vec!["HelloWorld".to_string()]);
    }

    #[test]
    fn test_lift_step() {
        let step = LiftStep::new(0, 10, 1);
        // Should not panic, but returns error for unimplemented
        let result = step.apply(&Node::new_text("doc"));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_marks_step() {
        let step = SetMarksStep::add_mark(0, 5, model::node::Mark {
            type_name: "bold".to_string(),
            attrs: None,
        });

        let result = step.apply(&Node::new_text("Hello")).expect("mark step should apply");
        match &result.doc.content {
            model::NodeContent::Text(tc) => assert_eq!(tc.marks[0].type_name, "bold"),
            model::NodeContent::Nodes(_) => panic!("expected text node"),
        }
    }
}
