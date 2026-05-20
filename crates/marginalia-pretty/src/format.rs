use marginalia::Span;

use crate::doc::{concat, trivia, Doc, Side};

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
