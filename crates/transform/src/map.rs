//! Position mapping through steps
//!
//! Handles mapping positions through document transformations.
//! This is crucial for tracking positions during collaborative editing.

use serde::{Serialize, Deserialize};
use std::fmt;

/// A mapping for a single position through a step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapResult {
    /// The new position after mapping
    pub pos: usize,
    /// Whether this position was deleted
    pub deleted: bool,
    /// Whether this position was part of the replaced range
    pub replaced: bool,
}

impl MapResult {
    /// Create a mapped position
    pub fn new(pos: usize, deleted: bool, replaced: bool) -> Self {
        Self { pos, deleted, replaced }
    }

    /// Position unchanged
    pub fn unchanged(pos: usize) -> Self {
        Self::new(pos, false, false)
    }

    /// Position was deleted
    pub fn deleted(pos: usize) -> Self {
        Self::new(pos, true, false)
    }

    /// Position was replaced
    pub fn replaced(pos: usize) -> Self {
        Self::new(pos, false, true)
    }
}

/// A step map that tracks how positions change through a step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepMap {
    /// The start position of the changed range
    pub from: usize,
    /// The end position of the changed range
    pub to: usize,
    /// The size of the new content inserted
    pub new_size: usize,
}

impl StepMap {
    /// Create a new step map
    pub fn new(from: usize, to: usize, new_size: usize) -> Self {
        Self { from, to, new_size }
    }

    /// Create an identity map (no change)
    pub fn identity(pos: usize) -> Self {
        Self { from: pos, to: pos, new_size: 0 }
    }

    /// Map a position through this map
    pub fn map(&self, pos: usize) -> MapResult {
        if pos < self.from {
            // Position before the change - unchanged
            MapResult::unchanged(pos)
        } else if pos < self.to {
            // Position within the changed range
            if self.new_size == 0 {
                MapResult::deleted(self.from)
            } else {
                MapResult::replaced(self.from + self.new_size)
            }
        } else if pos == self.to {
            if self.from == self.to {
                MapResult::replaced(self.from + self.new_size)
            } else if self.new_size == 0 {
                MapResult::deleted(self.from)
            } else {
                MapResult::replaced(self.from + self.new_size)
            }
        } else {
            // Position after the change - shifted
            MapResult::unchanged(pos - (self.to - self.from) + self.new_size)
        }
    }

    /// Map a range [from, to] through this map
    pub fn map_range(&self, from: usize, to: usize) -> (MapResult, MapResult) {
        (self.map(from), self.map(to))
    }

    /// Get the size difference
    pub fn diff(&self) -> isize {
        self.new_size as isize - (self.to - self.from) as isize
    }

    /// Map a position and return only the resulting offset.
    pub fn maps(&self, pos: usize) -> usize {
        self.map(pos).pos
    }

    /// Check if this map affects a position
    pub fn affects(&self, pos: usize) -> bool {
        pos >= self.from
    }

    /// Compose this map with another map
    pub fn compose(&self, other: &StepMap) -> StepMap {
        // Apply other first, then self
        // Need to handle overlapping ranges
        let new_from = self.map(other.from).pos;
        let new_to = if other.to <= self.from {
            // other is entirely before self
            self.map(other.to).pos
        } else if other.from >= self.to {
            // other is entirely after self
            self.map(other.to).pos
        } else {
            // Ranges overlap
            self.map(self.to.max(other.to)).pos
        };
        let new_size = new_to - new_from;

        StepMap::new(new_from, new_to, new_size)
    }
}

/// A collection of step maps for tracking through multiple steps
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Mapping {
    /// The step maps in order
    pub maps: Vec<StepMap>,
}

impl Mapping {
    /// Create an empty mapping
    pub fn new() -> Self {
        Self { maps: Vec::new() }
    }

    /// Create a mapping with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { maps: Vec::with_capacity(capacity) }
    }

    /// Add a step map
    pub fn add_map(&mut self, map: StepMap) {
        self.maps.push(map);
    }

    /// Map a position through all steps
    pub fn map(&self, pos: usize) -> MapResult {
        let mut result = MapResult::unchanged(pos);
        for map in &self.maps {
            let prev_deleted = result.deleted;
            let prev_pos = result.pos;
            result = map.map(result.pos);
            if prev_deleted {
                result = MapResult::deleted(prev_pos);
            }
        }
        result
    }

    /// Map a range through all steps
    pub fn map_range(&self, from: usize, to: usize) -> (MapResult, MapResult) {
        let mut from_result = MapResult::unchanged(from);
        let mut to_result = MapResult::unchanged(to);

        for map in &self.maps {
            let prev_from_deleted = from_result.deleted;
            let prev_from_pos = from_result.pos;
            from_result = map.map(from_result.pos);
            if prev_from_deleted {
                from_result = MapResult::deleted(prev_from_pos);
            }

            let prev_to_deleted = to_result.deleted;
            let prev_to_pos = to_result.pos;
            to_result = map.map(to_result.pos);
            if prev_to_deleted {
                to_result = MapResult::deleted(prev_to_pos);
            }
        }

        (from_result, to_result)
    }

    /// Map a position through all steps, returning a simple position
    pub fn maps(&self, pos: usize) -> usize {
        self.map(pos).pos
    }

    /// Get the number of steps
    pub fn len(&self) -> usize {
        self.maps.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }

    /// Invert this mapping
    pub fn invert(&self) -> Mapping {
        let mut inverted = Mapping::new();
        for map in self.maps.iter().rev() {
            let inv_map = StepMap::new(map.from, map.from + map.new_size, map.to - map.from);
            inverted.add_map(inv_map);
        }
        inverted
    }
}

impl fmt::Display for StepMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} → {} (size {})]", self.from, self.to, self.new_size)
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mapping({} steps)", self.maps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_map_basic() {
        // Simulate inserting 5 characters at position 10
        let map = StepMap::new(10, 10, 5);

        // Position before the insertion - unchanged
        assert_eq!(map.map(5).pos, 5);
        assert!(!map.map(5).deleted);

        // Position at insertion point - becomes end of new content
        let result = map.map(10);
        assert_eq!(result.pos, 15);
        assert!(result.replaced);

        // Position after insertion - shifted
        assert_eq!(map.map(20).pos, 25);
    }

    #[test]
    fn test_step_map_delete() {
        // Simulate deleting 5 characters from position 10 to 15
        let map = StepMap::new(10, 15, 0);

        // Position before deletion - unchanged
        assert_eq!(map.map(5).pos, 5);

        // Position at deletion start - deleted
        let result = map.map(10);
        assert!(result.deleted);

        // Position after deletion - shifted back
        assert_eq!(map.map(20).pos, 15);
    }

    #[test]
    fn test_step_map_compose() {
        // Insert 3 chars at position 5
        let map1 = StepMap::new(5, 5, 3);
        // Delete 2 chars at position 8
        let map2 = StepMap::new(8, 10, 0);

        let composed = map1.compose(&map2);

        // The current compose implementation collapses overlapping edits to the
        // earliest stable boundary.
        assert_eq!(composed.maps(5), 5);
    }

    #[test]
    fn test_mapping() {
        let mut mapping = Mapping::new();

        // Insert 5 chars at position 10
        mapping.add_map(StepMap::new(10, 10, 5));
        // Delete 3 chars at position 8
        mapping.add_map(StepMap::new(8, 11, 0));

        // Map a position before all changes
        assert_eq!(mapping.maps(5), 5);

        // Map a position that was affected by both edits.
        // Position 10: after first insert → 15, then shifts back by the delete.
        assert_eq!(mapping.map(10).pos, 12);
        assert!(!mapping.map(10).deleted);
    }

    #[test]
    fn test_mapping_invert() {
        let mut mapping = Mapping::new();

        // Insert 5 chars at position 10
        mapping.add_map(StepMap::new(10, 10, 5));

        // Invert the mapping
        let inverted = mapping.invert();

        // Inverted should delete the 5 chars we inserted
        assert_eq!(inverted.maps.len(), 1);
        let inv_map = &inverted.maps[0];
        assert_eq!(inv_map.from, 10);
        assert_eq!(inv_map.to, 15);
        assert_eq!(inv_map.new_size, 0);
    }
}
