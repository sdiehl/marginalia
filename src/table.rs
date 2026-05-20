use crate::{span::Span, trivia::Trivia};

#[derive(Clone, Debug)]
pub struct TriviaEvent {
    pub span: Span,
    pub trivia: Trivia,
}

#[derive(Clone, Debug, Default)]
pub struct TriviaTable {
    events: Vec<TriviaEvent>,
}

impl TriviaTable {
    #[must_use]
    pub const fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: TriviaEvent) {
        self.events.push(event);
    }

    #[must_use]
    pub fn events(&self) -> &[TriviaEvent] {
        &self.events
    }

    pub fn between(&self, lo: usize, hi: usize) -> impl Iterator<Item = &TriviaEvent> {
        self.events
            .iter()
            .filter(move |e| e.span.start >= lo && e.span.end <= hi)
    }

    pub fn events_in(&self, span: Span) -> impl Iterator<Item = &TriviaEvent> {
        self.between(span.start, span.end)
    }

    pub fn after(&self, pos: usize) -> impl Iterator<Item = &TriviaEvent> {
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
