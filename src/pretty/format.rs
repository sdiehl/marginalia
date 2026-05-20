use super::doc::{concat, trivia, Doc, Side};
use crate::Span;

pub trait Format {
    fn doc(&self) -> Doc;
}

#[must_use]
pub fn with_trivia(span: Span, body: Doc) -> Doc {
    concat([
        trivia(span, Side::Leading),
        body,
        trivia(span, Side::Trailing),
    ])
}
