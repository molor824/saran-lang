use std::rc::Rc;

use super::span::*;

#[derive(Debug, Clone)]
pub struct Error {
    message: SpanOf<String>,
    source: Rc<String>,
}
impl Error {
    pub const fn new(message: SpanOf<String>, source: Rc<String>) -> Self {
        Self { message, source }
    }
}
