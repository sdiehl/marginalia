use std::marker::PhantomData;

use crate::{
    classify::Classify,
    span::Span,
    table::{TriviaEvent, TriviaTable},
    trivia::Trivia,
};

pub struct TriviaLexer<I, T, E> {
    inner: I,
    source: String,
    table: TriviaTable,
    cursor: usize,
    _marker: PhantomData<fn() -> Result<T, E>>,
}

impl<I, T, E> TriviaLexer<I, T, E>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Classify,
{
    pub fn new(inner: I, source: impl Into<String>) -> Self {
        Self {
            inner,
            source: source.into(),
            table: TriviaTable::new(),
            cursor: 0,
            _marker: PhantomData,
        }
    }

    pub fn table(&self) -> &TriviaTable {
        &self.table
    }

    pub fn into_table(self) -> TriviaTable {
        self.table
    }

    pub fn into_parts(self) -> (String, TriviaTable) {
        (self.source, self.table)
    }

    fn record_blank_lines(&mut self, up_to: usize) {
        let region = self.source.get(self.cursor..up_to).unwrap_or("");
        let newlines = region.bytes().filter(|&b| b == b'\n').count();
        if newlines >= 2 {
            self.table.push(TriviaEvent {
                span: Span::new(self.cursor, up_to),
                trivia: Trivia::BlankLine,
            });
        }
    }
}

impl<I, T, E> Iterator for TriviaLexer<I, T, E>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Classify,
{
    type Item = Result<(usize, T, usize), E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok((lo, tok, hi)) => {
                    if let Some(piece) = tok.trivia() {
                        self.record_blank_lines(lo);
                        self.table.push(TriviaEvent {
                            span: Span::new(lo, hi),
                            trivia: Trivia::from_kind(piece.kind, piece.text),
                        });
                        self.cursor = hi;
                        continue;
                    }
                    self.record_blank_lines(lo);
                    self.cursor = hi;
                    return Some(Ok((lo, tok, hi)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl<I, T, E> std::fmt::Debug for TriviaLexer<I, T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriviaLexer")
            .field("events", &self.table.len())
            .field("cursor", &self.cursor)
            .finish_non_exhaustive()
    }
}
