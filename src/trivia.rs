#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trivia {
    Line(String),
    Block(String),
    BlankLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Line,
    Block,
}

impl Trivia {
    #[must_use]
    pub fn from_kind(kind: TriviaKind, text: &str) -> Self {
        match kind {
            TriviaKind::Line => Self::Line(text.to_owned()),
            TriviaKind::Block => Self::Block(text.to_owned()),
        }
    }

    #[must_use]
    pub const fn is_blank(&self) -> bool {
        matches!(self, Self::BlankLine)
    }

    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Line(s) | Self::Block(s) => Some(s),
            Self::BlankLine => None,
        }
    }
}
