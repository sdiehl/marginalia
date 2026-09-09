use crate::{
    span::Span,
    trivia::{BuiltinKind, Trivia},
};

/// One piece of trivia together with the source range it occupied.
#[derive(Clone, Debug)]
pub struct TriviaEvent<K = BuiltinKind> {
    /// Source range the trivia occupied.
    pub span: Span,
    /// The trivia itself.
    pub trivia: Trivia<K>,
}

/// The side table of trivia collected by a [`crate::TriviaLexer`].
///
/// Events arrive in source order when the table is filled by the lexer, and
/// the range queries exploit that to narrow their scan. Pushing out of order
/// is allowed and stays correct: the table notices and falls back to a full
/// scan.
#[derive(Clone, Debug)]
pub struct TriviaTable<K = BuiltinKind> {
    events: Vec<TriviaEvent<K>>,
    /// True while `events` is non-decreasing in `span.start`.
    sorted: bool,
}

impl<K> Default for TriviaTable<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> TriviaTable<K> {
    /// An empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            events: Vec::new(),
            sorted: true,
        }
    }

    /// Append an event.
    pub fn push(&mut self, event: TriviaEvent<K>) {
        if let Some(last) = self.events.last() {
            self.sorted &= last.span.start <= event.span.start;
        }
        self.events.push(event);
    }

    /// Every event, in insertion order.
    #[must_use]
    pub fn events(&self) -> &[TriviaEvent<K>] {
        &self.events
    }

    /// Every event, in insertion order.
    pub fn iter(&self) -> std::slice::Iter<'_, TriviaEvent<K>> {
        self.events.iter()
    }

    /// Events fully contained in `lo..hi`.
    pub fn between(&self, lo: usize, hi: usize) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.window(lo, Some(hi))
            .iter()
            .filter(move |e| e.span.start >= lo && e.span.end <= hi)
    }

    /// Events fully contained in `span`.
    pub fn events_in(&self, span: Span) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.between(span.start, span.end)
    }

    /// Events beginning at or after `pos`.
    pub fn after(&self, pos: usize) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.window(pos, None)
            .iter()
            .filter(move |e| e.span.start >= pos)
    }

    /// The slice that can contain events starting in `lo..=hi`.
    ///
    /// Only a narrowing hint: callers still filter, so an unsorted table
    /// simply gets the whole slice back.
    fn window(&self, lo: usize, hi: Option<usize>) -> &[TriviaEvent<K>] {
        if !self.sorted {
            return &self.events;
        }
        let from = self.events.partition_point(|e| e.span.start < lo);
        let rest = &self.events[from..];
        match hi {
            Some(hi) => &rest[..rest.partition_point(|e| e.span.start <= hi)],
            None => rest,
        }
    }

    /// Number of events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True when no events were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl<K> Extend<TriviaEvent<K>> for TriviaTable<K> {
    fn extend<I: IntoIterator<Item = TriviaEvent<K>>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        self.events.reserve(iter.size_hint().0);
        for event in iter {
            self.push(event);
        }
    }
}

impl<K> FromIterator<TriviaEvent<K>> for TriviaTable<K> {
    fn from_iter<I: IntoIterator<Item = TriviaEvent<K>>>(iter: I) -> Self {
        let mut table = Self::new();
        table.extend(iter);
        table
    }
}

impl<'a, K> IntoIterator for &'a TriviaTable<K> {
    type Item = &'a TriviaEvent<K>;
    type IntoIter = std::slice::Iter<'a, TriviaEvent<K>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
