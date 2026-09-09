use crate::trivia::BuiltinKind;

/// A trivia piece classified by the user's token type.
///
/// `K` is the user's kind enum, defaulting to [`BuiltinKind`] for the
/// line/block-only common case.
#[derive(Clone, Copy, Debug)]
pub struct TriviaPiece<'a, K = BuiltinKind> {
    /// The user's classification of this piece.
    pub kind: K,
    /// Verbatim source text, delimiters included.
    pub text: &'a str,
}

/// Tokens that may carry trivia implement [`Classify`] so the lexer can split
/// them off into a side table.
///
/// `K` defaults to [`BuiltinKind`]; downstream crates supply their own kind
/// enum when they need to distinguish more comment categories.
pub trait Classify<K = BuiltinKind> {
    /// `Some` if this token is trivia rather than a semantic token, in which
    /// case [`crate::TriviaLexer`] diverts it into the side table instead of
    /// handing it to the parser.
    fn trivia(&self) -> Option<TriviaPiece<'_, K>>;
}
