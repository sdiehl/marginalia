use std::{fmt, ops::Range};

/// A half-open byte range `start..end` into the source text.
///
/// Spans are the identity used throughout the crate: [`crate::TriviaTable`]
/// records one per trivia event, [`crate::attach`] anchors comments to node
/// spans, and [`crate::pretty::with_trivia`] names the slot to fill. The
/// derived `Ord` sorts by `(start, end)`, which puts an enclosing span before
/// the nodes it contains.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// Byte offset of the first character.
    pub start: usize,
    /// Byte offset one past the last character.
    pub end: usize,
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Self { start, end }
    }
}

impl From<Range<usize>> for Span {
    fn from(r: Range<usize>) -> Self {
        Self {
            start: r.start,
            end: r.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(s: Span) -> Self {
        s.start..s.end
    }
}

impl Span {
    /// Construct a span covering `start..end`.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Construct the zero-length span at `at`.
    #[must_use]
    pub const fn empty(at: usize) -> Self {
        Self { start: at, end: at }
    }

    /// Length in bytes, saturating at zero for an inverted span.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// True when the span covers no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start >= self.end
    }

    /// True when `pos` falls inside the span (`start <= pos < end`).
    #[must_use]
    pub const fn contains(self, pos: usize) -> bool {
        self.start <= pos && pos < self.end
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// Shorthand for [`Span::new`].
#[must_use]
pub const fn span(start: usize, end: usize) -> Span {
    Span::new(start, end)
}
