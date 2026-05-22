use std::marker::PhantomData;

use crate::{
    classify::Classify,
    span::Span,
    table::{TriviaEvent, TriviaTable},
    trivia::{BuiltinKind, Trivia},
};

pub struct TriviaLexer<I, T, E, K = BuiltinKind> {
    inner: I,
    source: String,
    table: TriviaTable<K>,
    cursor: usize,
    _marker: PhantomData<fn() -> Result<T, E>>,
}

impl<I, T, E, K> TriviaLexer<I, T, E, K>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Classify<K>,
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

    pub fn table(&self) -> &TriviaTable<K> {
        &self.table
    }

    pub fn into_table(self) -> TriviaTable<K> {
        self.table
    }

    pub fn into_parts(self) -> (String, TriviaTable<K>) {
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

impl<I, T, E, K> Iterator for TriviaLexer<I, T, E, K>
where
    I: Iterator<Item = Result<(usize, T, usize), E>>,
    T: Classify<K>,
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
                            trivia: Trivia::Comment {
                                kind: piece.kind,
                                text: piece.text.to_owned(),
                            },
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

impl<I, T, E, K> std::fmt::Debug for TriviaLexer<I, T, E, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriviaLexer")
            .field("events", &self.table.len())
            .field("cursor", &self.cursor)
            .finish_non_exhaustive()
    }
}
