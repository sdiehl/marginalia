use std::collections::BTreeMap;

use crate::{trivia::BuiltinKind, Span, Trivia};

/// An AST node that can anchor comments.
pub trait HasSpan {
    /// The source range this node covers.
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
/// `trailing` comments follow the anchor on the *same source line*: the
/// attacher only populates this slot when the comment was on the same line
/// as the anchor's last token. Comments separated by a line break instead
/// become leading on the next anchor, or dangling if no next anchor exists.
#[derive(Clone, Debug)]
pub struct Comments<K = BuiltinKind> {
    /// Comments rendered before the anchor, one per line.
    pub leading: Vec<Trivia<K>>,
    /// Comments rendered after the anchor on the same line.
    pub trailing: Vec<Trivia<K>>,
}

impl<K> Default for Comments<K> {
    fn default() -> Self {
        Self {
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }
}

impl<K> Comments<K> {
    /// True when neither slot holds anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leading.is_empty() && self.trailing.is_empty()
    }

    /// Total number of comments in both slots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.leading.len() + self.trailing.len()
    }
}

/// Comments indexed by the anchor span they were attached to, plus the
/// dangling ones that had no anchor.
#[derive(Clone, Debug)]
pub struct CommentMap<K = BuiltinKind> {
    by_span: BTreeMap<Span, Comments<K>>,
    dangling: Vec<Trivia<K>>,
}

impl<K> Default for CommentMap<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> CommentMap<K> {
    /// An empty map.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            by_span: BTreeMap::new(),
            dangling: Vec::new(),
        }
    }

    /// The comment slots for `span`, inserting empty ones if absent.
    pub fn entry(&mut self, span: Span) -> &mut Comments<K> {
        self.by_span.entry(span).or_default()
    }

    /// The comment slots for `span`, if any were attached.
    #[must_use]
    pub fn get(&self, span: Span) -> Option<&Comments<K>> {
        self.by_span.get(&span)
    }

    /// Comments preceding `span`, empty if none.
    #[must_use]
    pub fn leading(&self, span: Span) -> &[Trivia<K>] {
        self.by_span
            .get(&span)
            .map_or(&[][..], |c| c.leading.as_slice())
    }

    /// Comments following `span` on the same line, empty if none.
    #[must_use]
    pub fn trailing(&self, span: Span) -> &[Trivia<K>] {
        self.by_span
            .get(&span)
            .map_or(&[][..], |c| c.trailing.as_slice())
    }

    /// Comments that found no anchor. The renderer appends these at the end of
    /// the document when
    /// [`RenderOpts::emit_dangling`](crate::pretty::RenderOpts)
    /// is set, which is what keeps a trailing comment from being dropped.
    #[must_use]
    pub fn dangling(&self) -> &[Trivia<K>] {
        &self.dangling
    }

    /// Record a comment with no anchor.
    pub fn push_dangling(&mut self, t: Trivia<K>) {
        self.dangling.push(t);
    }

    /// Number of anchor spans carrying comments.
    ///
    /// This counts anchors, not comments, and excludes dangling trivia, so a
    /// map with `len() == 0` may still not be [`is_empty`](Self::is_empty).
    /// Use [`comment_len`](Self::comment_len) to count comments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_span.len()
    }

    /// Number of dangling comments.
    #[must_use]
    pub fn dangling_len(&self) -> usize {
        self.dangling.len()
    }

    /// Total number of comments held, attached and dangling.
    #[must_use]
    pub fn comment_len(&self) -> usize {
        self.by_span.values().map(Comments::len).sum::<usize>() + self.dangling.len()
    }

    /// True when the map holds nothing at all, attached or dangling.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_span.is_empty() && self.dangling.is_empty()
    }

    /// Anchor spans and their comments, in span order.
    pub fn iter(&self) -> Anchors<'_, K> {
        self.by_span.iter().map(deref_span as DerefSpan<'_, K>)
    }
}

impl<'a, K> IntoIterator for &'a CommentMap<K> {
    type Item = (Span, &'a Comments<K>);
    type IntoIter = Anchors<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

type DerefSpan<'a, K> = fn((&'a Span, &'a Comments<K>)) -> (Span, &'a Comments<K>);

fn deref_span<'a, K>((span, comments): (&'a Span, &'a Comments<K>)) -> (Span, &'a Comments<K>) {
    (*span, comments)
}

/// Iterator over a [`CommentMap`]'s anchor spans and their comments, in span
/// order. Returned by [`CommentMap::iter`].
pub type Anchors<'a, K = BuiltinKind> =
    std::iter::Map<std::collections::btree_map::Iter<'a, Span, Comments<K>>, DerefSpan<'a, K>>;
