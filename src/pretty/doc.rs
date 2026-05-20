use crate::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    Align(Box<Doc>),
    FlatAlt(Box<Doc>, Box<Doc>),
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
pub fn char(c: char) -> Doc {
    Doc::Text(c.to_string())
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
pub fn align(d: Doc) -> Doc {
    Doc::Align(Box::new(d))
}

#[must_use]
pub fn hang(n: isize, d: Doc) -> Doc {
    align(indent(n, d))
}

#[must_use]
pub fn flat_alt(flat: Doc, broken: Doc) -> Doc {
    Doc::FlatAlt(Box::new(flat), Box::new(broken))
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

#[must_use]
pub fn space() -> Doc {
    text(" ")
}
#[must_use]
pub fn comma() -> Doc {
    text(",")
}
#[must_use]
pub fn semi() -> Doc {
    text(";")
}
#[must_use]
pub fn colon() -> Doc {
    text(":")
}
#[must_use]
pub fn dot() -> Doc {
    text(".")
}
#[must_use]
pub fn equals() -> Doc {
    text("=")
}
#[must_use]
pub fn lparen() -> Doc {
    text("(")
}
#[must_use]
pub fn rparen() -> Doc {
    text(")")
}
#[must_use]
pub fn lbracket() -> Doc {
    text("[")
}
#[must_use]
pub fn rbracket() -> Doc {
    text("]")
}
#[must_use]
pub fn lbrace() -> Doc {
    text("{")
}
#[must_use]
pub fn rbrace() -> Doc {
    text("}")
}
#[must_use]
pub fn langle() -> Doc {
    text("<")
}
#[must_use]
pub fn rangle() -> Doc {
    text(">")
}
#[must_use]
pub fn dquote() -> Doc {
    text("\"")
}
#[must_use]
pub fn squote() -> Doc {
    text("'")
}

#[must_use]
pub fn hcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    concat(parts)
}

#[must_use]
pub fn hsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, space)
}

#[must_use]
pub fn vcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, hardline)
}

#[must_use]
pub fn vsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, line)
}

#[must_use]
pub fn sep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(vsep(parts))
}

#[must_use]
pub fn cat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(interleave(parts, softline))
}

#[must_use]
pub fn punctuate<I: IntoIterator<Item = Doc>>(sep: &Doc, parts: I) -> Vec<Doc> {
    let items: Vec<Doc> = parts.into_iter().collect();
    let last = items.len().saturating_sub(1);
    items
        .into_iter()
        .enumerate()
        .flat_map(|(i, d)| {
            if i == last {
                vec![d]
            } else {
                vec![d, sep.clone()]
            }
        })
        .collect()
}

#[must_use]
pub fn enclose(left: Doc, right: Doc, body: Doc) -> Doc {
    concat([left, body, right])
}

#[must_use]
pub fn parens(d: Doc) -> Doc {
    enclose(lparen(), rparen(), d)
}
#[must_use]
pub fn brackets(d: Doc) -> Doc {
    enclose(lbracket(), rbracket(), d)
}
#[must_use]
pub fn braces(d: Doc) -> Doc {
    enclose(lbrace(), rbrace(), d)
}
#[must_use]
pub fn angles(d: Doc) -> Doc {
    enclose(langle(), rangle(), d)
}
#[must_use]
pub fn dquotes(d: Doc) -> Doc {
    enclose(dquote(), dquote(), d)
}
#[must_use]
pub fn squotes(d: Doc) -> Doc {
    enclose(squote(), squote(), d)
}

#[must_use]
pub fn enclose_sep<I: IntoIterator<Item = Doc>>(left: Doc, right: Doc, sep: &Doc, parts: I) -> Doc {
    enclose(left, right, concat(punctuate(sep, parts)))
}

#[must_use]
pub fn list<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lbracket(), rbracket(), &text(", "), parts)
}

#[must_use]
pub fn tupled<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lparen(), rparen(), &text(", "), parts)
}

fn interleave<I: IntoIterator<Item = Doc>, F: Fn() -> Doc>(parts: I, sep: F) -> Doc {
    let mut out = Vec::new();
    for (i, p) in parts.into_iter().enumerate() {
        if i > 0 {
            out.push(sep());
        }
        out.push(p);
    }
    concat(out)
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

    /// `self <+> other` — concatenate with a single space between.
    #[must_use]
    pub fn space(self, other: Doc) -> Doc {
        self.append(space()).append(other)
    }

    /// `self </> other` — concatenate with `line` between (space when flat,
    /// newline when broken).
    #[must_use]
    pub fn line(self, other: Doc) -> Doc {
        self.append(line()).append(other)
    }

    /// `self <$> other` — concatenate with `hardline` between.
    #[must_use]
    pub fn hardline(self, other: Doc) -> Doc {
        self.append(hardline()).append(other)
    }

    /// `self <//> other` — concatenate with `softline` between (empty when
    /// flat, newline when broken).
    #[must_use]
    pub fn softline(self, other: Doc) -> Doc {
        self.append(softline()).append(other)
    }
}
