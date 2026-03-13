//! State layer - handles editor state
//!
//! Contains EditorState, Selection, and Transaction

pub mod selection;
pub mod state;
pub mod transaction;

pub use selection::{Selection, TextSelection, NodeSelection, SelectionRange};
pub use state::{EditorState, StoredMarks};
pub use transaction::{Transaction, TransactionBuilder};
