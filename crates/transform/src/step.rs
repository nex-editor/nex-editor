//! Step definitions and execution
//!
//! Defines atomic operations that can be applied to documents.

use model::Node;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::replace::Slice;
use crate::map::StepMap;
use crate::replace::ReplaceStep;

/// Errors from step execution
#[derive(Debug, Error, PartialEq)]
pub enum StepError {
    #[error("Invalid step for document")]
    InvalidStep,

    #[error("Position out of range: {0}")]
    PositionOutOfRange(usize),

    #[error("Step failed: {0}")]
    Failed(String),
}

/// Result of applying a step
#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    /// The updated document
    pub doc: Node,
    /// The position before the replaced range
    pub from: usize,
    /// The position after the replaced range
    pub to: usize,
    /// The slice that was inserted
    pub slice: Option<Slice>,
}

impl StepResult {
    /// Create a new step result
    pub fn new(doc: Node, from: usize, to: usize, slice: Option<Slice>) -> Self {
        Self { doc, from, to, slice }
    }

    /// Create a result with no slice (deletion)
    pub fn delete(doc: Node, from: usize, to: usize) -> Self {
        Self::new(doc, from, to, None)
    }

    /// Create a result with a slice (insertion/replacement)
    pub fn insert(doc: Node, pos: usize, slice: Slice) -> Self {
        Self::new(doc, pos, pos, Some(slice))
    }

    /// Get the size of the replacement
    pub fn replaced_size(&self) -> usize {
        self.to - self.from
    }

    /// Get the size of the inserted content
    pub fn inserted_size(&self) -> usize {
        self.slice.as_ref().map(|s| s.size()).unwrap_or(0)
    }
}

/// Trait for types that can be mapped
pub trait Mappable {
    /// Map this through a step map
    fn map(&self, map: &StepMap) -> Self;
}

/// Trait for steps that can be inverted
pub trait Invertible {
    /// Create an inverse step
    fn invert(&self, doc: &Node) -> Self;
}

/// Trait for all steps
pub trait Step: Send + Sync {
    /// Apply this step to a document
    fn apply(&self, doc: &Node) -> Result<StepResult, StepError>;

    /// Get the steps that this step is composed of
    fn get_steps(&self) -> Vec<&dyn Step>
    where
        Self: Sized,
    {
        vec![self]
    }

    /// Get mutable references to steps
    fn get_steps_mut(&mut self) -> Vec<&mut dyn Step>
    where
        Self: Sized,
    {
        vec![self]
    }
}

/// Represents the type of a step for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Replace step
    Replace,
    /// Add mark step
    AddMark,
    /// Remove mark step
    RemoveMark,
}

/// A step with its type for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepWrapper {
    /// The type of step
    pub step_type: StepType,
    /// Step-specific data
    pub data: serde_json::Value,
}

impl StepWrapper {
    /// Create a new wrapper
    pub fn new(step_type: StepType, data: serde_json::Value) -> Self {
        Self { step_type, data }
    }
}

/// Builder for common steps
#[derive(Debug, Clone)]
pub struct StepBuilder;

impl StepBuilder {
    /// Create an insert step
    pub fn insert(pos: usize, text: &str) -> ReplaceStep {
        let slice = Slice::from_text(text);
        ReplaceStep::insert(pos, slice)
    }

    /// Create a delete step
    pub fn delete(from: usize, to: usize) -> ReplaceStep {
        ReplaceStep::delete(from, to)
    }

    /// Create a replace step
    pub fn replace(from: usize, to: usize, slice: Slice) -> ReplaceStep {
        ReplaceStep::replace(from, to, slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_result_basics() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello")]);

        // Simulate a delete result
        let result = StepResult::delete(doc, 0, 5);
        assert_eq!(result.replaced_size(), 5);
        assert!(result.slice.is_none());
    }

    #[test]
    fn test_step_result_insert() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello")]);
        let slice = Slice::from_text(" World");

        let result = StepResult::insert(doc, 5, slice);
        assert_eq!(result.replaced_size(), 0);
        assert_eq!(result.inserted_size(), 6);
    }
}
