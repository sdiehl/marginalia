use marginalia::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriviaSlot {
    pub span: Span,
    pub side: Side,
}

#[derive(Clone, Debug)]
pub enum Doc {
    Nil,
    Text(String),
    Line,
    SoftLine,
    HardLine,
    Indent(isize, Box<Doc>),
    Group(Box<Doc>),
    Concat(Vec<Doc>),
    Trivia(TriviaSlot),
}

#[must_use]
pub const fn nil() -> Doc {
    Doc::Nil
}

#[must_use]
pub fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}

#[must_use]
pub const fn line() -> Doc {
    Doc::Line
}

#[must_use]
pub const fn softline() -> Doc {
    Doc::SoftLine
}

#[must_use]
pub const fn hardline() -> Doc {
    Doc::HardLine
}

#[must_use]
pub fn indent(n: isize, d: Doc) -> Doc {
    Doc::Indent(n, Box::new(d))
}

#[must_use]
pub fn group(d: Doc) -> Doc {
    Doc::Group(Box::new(d))
}

#[must_use]
pub fn concat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    Doc::Concat(parts.into_iter().collect())
}

#[must_use]
pub const fn trivia(span: Span, side: Side) -> Doc {
    Doc::Trivia(TriviaSlot { span, side })
}

impl Doc {
    #[must_use]
    pub fn append(self, other: Doc) -> Doc {
        match (self, other) {
            (Doc::Nil, x) | (x, Doc::Nil) => x,
            (Doc::Concat(mut a), Doc::Concat(b)) => {
                a.extend(b);
                Doc::Concat(a)
            }
            (Doc::Concat(mut a), b) => {
                a.push(b);
                Doc::Concat(a)
            }
            (a, Doc::Concat(mut b)) => {
                b.insert(0, a);
                Doc::Concat(b)
            }
            (a, b) => Doc::Concat(vec![a, b]),
        }
    }
}
