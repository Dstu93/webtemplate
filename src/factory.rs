use crate::web::{RequestProcessor, StandardRequestProcessor};

pub fn create_request_proc() -> impl RequestProcessor {
    StandardRequestProcessor
}