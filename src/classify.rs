use crate::trivia::BuiltinKind;

/// A trivia piece classified by the user's token type.
///
/// `K` is the user's kind enum, defaulting to [`BuiltinKind`] for the
/// line/block-only common case.
#[derive(Clone, Copy, Debug)]
pub struct TriviaPiece<'a, K = BuiltinKind> {
    pub kind: K,
    pub text: &'a str,
}

/// Tokens that may carry trivia implement [`Classify`] so the lexer can split
/// them off into a side table.
///
/// `K` defaults to [`BuiltinKind`]; downstream crates supply their own kind
/// enum when they need to distinguish more comment categories.
pub trait Classify<K = BuiltinKind> {
    fn trivia(&self) -> Option<TriviaPiece<'_, K>>;
}
