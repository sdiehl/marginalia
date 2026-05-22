use crate::{
    span::Span,
    trivia::{BuiltinKind, Trivia},
};

#[derive(Clone, Debug)]
pub struct TriviaEvent<K = BuiltinKind> {
    pub span: Span,
    pub trivia: Trivia<K>,
}

#[derive(Clone, Debug)]
pub struct TriviaTable<K = BuiltinKind> {
    events: Vec<TriviaEvent<K>>,
}

impl<K> Default for TriviaTable<K> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<K> TriviaTable<K> {
    #[must_use]
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: TriviaEvent<K>) {
        self.events.push(event);
    }

    #[must_use]
    pub fn events(&self) -> &[TriviaEvent<K>] {
        &self.events
    }

    pub fn between(&self, lo: usize, hi: usize) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.events
            .iter()
            .filter(move |e| e.span.start >= lo && e.span.end <= hi)
    }

    pub fn events_in(&self, span: Span) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.between(span.start, span.end)
    }

    pub fn after(&self, pos: usize) -> impl Iterator<Item = &TriviaEvent<K>> {
        self.events.iter().filter(move |e| e.span.start >= pos)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
