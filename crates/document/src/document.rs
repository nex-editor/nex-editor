use loro::{LoroDoc};

pub struct Document {
    doc: LoroDoc,
}

impl Document {
    /// Create a new document
    pub fn new() -> Self {
        Self {
            doc: LoroDoc::new(),
        }
    }

    /// Get the underlying LoroDoc
    pub fn loro_doc(&self) -> &LoroDoc {
        &self.doc
    }

    /// Get a mutable reference to the underlying LoroDoc
    pub fn loro_doc_mut(&mut self) -> &mut LoroDoc {
        &mut self.doc
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}