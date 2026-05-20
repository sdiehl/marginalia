use std::collections::BTreeMap;

use crate::{Span, Trivia};

pub trait HasSpan {
    fn span(&self) -> Span;
}

impl HasSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

/// Comments attached to a single anchor span.
///
/// `leading` comments precede the anchor (each rendered on its own line).
/// `trailing` comments follow the anchor on the *same source line* — the
/// attacher only populates this slot when the comment was on the same line
/// as the anchor's last token. Comments separated by a line break instead
/// become leading on the next anchor, or dangling if no next anchor exists.
#[derive(Clone, Debug, Default)]
pub struct Comments {
    pub leading: Vec<Trivia>,
    pub trailing: Vec<Trivia>,
}

impl Comments {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leading.is_empty() && self.trailing.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommentMap {
    by_span: BTreeMap<Span, Comments>,
    dangling: Vec<Trivia>,
}

impl CommentMap {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            by_span: BTreeMap::new(),
            dangling: Vec::new(),
        }
    }

    pub fn entry(&mut self, span: Span) -> &mut Comments {
        self.by_span.entry(span).or_default()
    }

    #[must_use]
    pub fn get(&self, span: Span) -> Option<&Comments> {
        self.by_span.get(&span)
    }

    #[must_use]
    pub fn leading(&self, span: Span) -> &[Trivia] {
        self.by_span
            .get(&span)
            .map_or(&[][..], |c| c.leading.as_slice())
    }

    #[must_use]
    pub fn trailing(&self, span: Span) -> &[Trivia] {
        self.by_span
            .get(&span)
            .map_or(&[][..], |c| c.trailing.as_slice())
    }

    #[must_use]
    pub fn dangling(&self) -> &[Trivia] {
        &self.dangling
    }

    pub fn push_dangling(&mut self, t: Trivia) {
        self.dangling.push(t);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_span.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_span.is_empty() && self.dangling.is_empty()
    }
}
