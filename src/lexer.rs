use std::marker::PhantomData;

use crate::{
    classify::Classify,
    span::Span,
    table::{TriviaEvent, TriviaTable},
    trivia::{BuiltinKind, Trivia},
};

/// A lexer adapter that diverts trivia into a side table.
///
/// Wraps any `Iterator<Item = Result<(usize, T, usize), E>>` (the shape
/// `lalrpop` expects) and yields only the tokens for which
/// [`Classify::trivia`] returns `None`. Everything else, plus the blank lines
/// between tokens, lands in a [`TriviaTable`] for [`crate::attach`] to place.
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
    /// Wrap `inner`, keeping `source` so blank lines between tokens can be
    /// detected.
    pub fn new(inner: I, source: impl Into<String>) -> Self {
        Self {
            inner,
            source: source.into(),
            table: TriviaTable::new(),
            cursor: 0,
            _marker: PhantomData,
        }
    }

    /// The trivia recorded so far.
    pub fn table(&self) -> &TriviaTable<K> {
        &self.table
    }

    /// Consume the lexer for its trivia table, the usual move once parsing is
    /// done.
    #[must_use]
    pub fn into_table(self) -> TriviaTable<K> {
        self.table
    }

    /// Consume the lexer for both the source it was given and its trivia
    /// table.
    #[must_use]
    pub fn into_parts(self) -> (String, TriviaTable<K>) {
        (self.source, self.table)
    }

    /// Record a `BlankLine` event if the gap since the last token spans two or
    /// more newlines.
    fn record_blank_lines(&mut self, up_to: usize) {
        let gap = self
            .source
            .as_bytes()
            .get(self.cursor..up_to)
            .unwrap_or_default();
        let newlines = gap.iter().filter(|&&b| b == b'\n').take(2).count();
        if newlines == 2 {
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
