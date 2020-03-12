use crate::web::{RequestProcessor, StandardRequestProcessor};

/// creates a RequestProcessor
pub fn create_request_proc() -> impl RequestProcessor {
    StandardRequestProcessor::new(vec![],vec![])
}