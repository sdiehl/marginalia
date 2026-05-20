use crate::trivia::TriviaKind;

#[derive(Clone, Copy, Debug)]
pub struct TriviaPiece<'a> {
    pub kind: TriviaKind,
    pub text: &'a str,
}

pub trait Classify {
    fn trivia(&self) -> Option<TriviaPiece<'_>>;
}
