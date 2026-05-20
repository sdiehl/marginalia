use super::doc::{concat, trivia, Doc, Side};
use crate::Span;

/// Lower an AST node to a [`Doc`].
///
/// Implement this for each AST node so that pretty-printing reduces to one
/// call per type. Implementations should wrap node bodies with
/// [`with_trivia`] for any span that carries comments, and never emit raw
/// comment text. The [`render`](super::render) pass resolves the trivia
/// slots against a [`CommentMap`](crate::attach::CommentMap).
///
/// `Format` is purely a convention: nothing in the renderer requires it.
/// Free functions returning [`Doc`] work equally well; the trait just gives
/// you a uniform `node.doc()` call site.
pub trait Format {
    fn doc(&self) -> Doc;
}

/// Wrap `body` with leading + trailing trivia slots for `span`.
///
/// Calling `with_trivia` on the same span twice in a single document is
/// safe: the renderer deduplicates trivia slots by `(span, side)` so each
/// comment appears at most once.
#[must_use]
pub fn with_trivia(span: Span, body: Doc) -> Doc {
    concat([
        trivia(span, Side::Leading),
        body,
        trivia(span, Side::Trailing),
    ])
}
